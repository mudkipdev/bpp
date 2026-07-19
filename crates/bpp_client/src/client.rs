/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use bpp_shared::logger::logger::global_logger;
use bpp_shared::world::client_pos::ClientPosition;

use crate::inputs::Input;
use crate::window::Window;

pub struct Client {
    window: Window,
    input: Input,
    single_player_pos: ClientPosition,
    accumulator: f32,
}

impl Client {
    const TICK_DELTA: f32 = 1.0 / 20.0;
    const MAX_TICKS_PER_FRAME: i32 = 10;

    // This window size seems really random but its the size beta uses
    pub fn new() -> Self {
        let mut window = Window::new(854, 480, "Betrock.rs");
        let mut input = Input::default();

        input.init(&mut window);
        window.init_callbacks();

        window.set_cursor_locked(true);
        window.set_vsync(true);

        global_logger().info("Client initialized\n");

        Client {
            window,
            input,
            single_player_pos: ClientPosition::default(),
            accumulator: 0.0,
        }
    }

    fn tick(&mut self) {}

    fn render(&mut self, partial_tick: f32) {
        let _ = partial_tick;
    }

    pub fn run(&mut self) -> i32 {
        let mut last_time = self.window.get_time() as f32;

        while !self.window.should_close() {
            let mut ticks_ran = 0;
            let now = self.window.get_time() as f32;
            let delta = now - last_time;
            last_time = now;
            self.accumulator += delta;

            self.window.poll_events(&mut self.input);

            // Run ticks until caught up, but cap to avoid spiraling on slow frames
            while self.accumulator >= Self::TICK_DELTA && ticks_ran < Self::MAX_TICKS_PER_FRAME {
                self.input.drain_events();
                self.tick();
                self.input.flush();
                self.accumulator -= Self::TICK_DELTA;
                ticks_ran += 1;
            }

            // Discard leftover time if we hit the cap
            if ticks_ran == Self::MAX_TICKS_PER_FRAME {
                self.accumulator = 0.0;
            }

            self.render(self.accumulator / Self::TICK_DELTA);
            self.window.swap_buffers();
        }
        0
    }
}
