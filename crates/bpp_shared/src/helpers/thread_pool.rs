use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread::JoinHandle;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct State {
    queue: VecDeque<Job>,
    in_flight: usize,
    shutdown: bool,
}

struct Inner {
    state: Mutex<State>,
    task_available: Condvar,
    task_done: Condvar,
}

pub struct ThreadPool {
    inner: Arc<Inner>,
    workers: Vec<JoinHandle<()>>,
}

pub struct Task<T> {
    receiver: mpsc::Receiver<T>,
    value: Option<T>,
}

impl<T> Task<T> {
    pub fn is_ready(&mut self) -> bool {
        if self.value.is_none() {
            if let Ok(value) = self.receiver.try_recv() {
                self.value = Some(value);
            }
        }
        self.value.is_some()
    }

    pub fn get(mut self) -> T {
        match self.value.take() {
            Some(value) => value,
            None => self.receiver.recv().expect("ThreadPool task panicked"),
        }
    }
}

impl ThreadPool {
    pub fn new(thread_count: usize) -> Self {
        let inner = Arc::new(Inner {
            state: Mutex::new(State {
                queue: VecDeque::new(),
                in_flight: 0,
                shutdown: false,
            }),
            task_available: Condvar::new(),
            task_done: Condvar::new(),
        });

        let workers = (0..thread_count.max(1))
            .map(|_| {
                let inner = Arc::clone(&inner);
                std::thread::spawn(move || Self::worker(inner))
            })
            .collect();

        Self { inner, workers }
    }

    pub fn detach_task(&self, task: impl FnOnce() + Send + 'static) {
        let mut state = self.inner.state.lock().unwrap();
        state.queue.push_back(Box::new(task));
        drop(state);
        self.inner.task_available.notify_one();
    }

    pub fn submit_task<T: Send + 'static>(
        &self,
        task: impl FnOnce() -> T + Send + 'static,
    ) -> Task<T> {
        let (sender, receiver) = mpsc::channel();
        self.detach_task(move || {
            let _ = sender.send(task());
        });
        Task {
            receiver,
            value: None,
        }
    }

    pub fn wait(&self) {
        let mut state = self.inner.state.lock().unwrap();
        while !state.queue.is_empty() || state.in_flight > 0 {
            state = self.inner.task_done.wait(state).unwrap();
        }
    }

    fn worker(inner: Arc<Inner>) {
        loop {
            let job = {
                let mut state = inner.state.lock().unwrap();
                loop {
                    if let Some(job) = state.queue.pop_front() {
                        state.in_flight += 1;
                        break job;
                    }
                    if state.shutdown {
                        return;
                    }
                    state = inner.task_available.wait(state).unwrap();
                }
            };

            let _ = catch_unwind(AssertUnwindSafe(job));

            let mut state = inner.state.lock().unwrap();
            state.in_flight -= 1;
            if state.in_flight == 0 && state.queue.is_empty() {
                inner.task_done.notify_all();
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.inner.state.lock().unwrap().shutdown = true;
        self.inner.task_available.notify_all();
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}
