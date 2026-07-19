/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use bpp_shared::logger::logger::global_logger;
use glfw::Context;
use glfw::fail_on_errors;

pub struct Window {
    glfw: glfw::Glfw,
    handle: glfw::PWindow,
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,

    width: i32,
    height: i32,
}

impl Window {
    pub fn new(width: i32, height: i32, title: &str) -> Self {
        let mut glfw = glfw::init(fail_on_errors!()).expect("Failed to init GLFW");

        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

        let (mut handle, events) = glfw
            .create_window(width as u32, height as u32, title, glfw::WindowMode::Windowed)
            .expect("Failed to create window");

        handle.make_current();

        gl::load_with(|s| handle.get_proc_address(s) as *const _);

        unsafe {
            gl::Viewport(0, 0, width, height);
        }
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
        }
        unsafe {
            gl::Enable(gl::CULL_FACE);
        }

        global_logger().info("Window initialized!\n");
        // Note: user pointer is set by Client, not here

        Window { glfw, handle, events, width, height }
    }

    // Called by Client after setting the user pointer
    pub fn init_callbacks(&mut self) {
        self.handle.set_framebuffer_size_polling(true);
    }

    pub fn should_close(&self) -> bool {
        self.handle.should_close()
    }

    pub fn swap_buffers(&mut self) {
        self.handle.swap_buffers();
    }

    pub fn poll_events(&mut self, input: &mut crate::inputs::Input) {
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::FramebufferSize(w, h) => {
                    self.width = w;
                    self.height = h;
                    unsafe {
                        gl::Viewport(0, 0, w, h);
                    }
                }
                glfw::WindowEvent::Key(key, _, action, _) => {
                    input.key_callback(key as i32, action);
                }
                glfw::WindowEvent::MouseButton(button, action, _) => {
                    input.mouse_button_callback(button as i32, action);
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    input.cursor_callback(x, y);
                }
                _ => {}
            }
        }
    }

    pub fn set_vsync(&mut self, enabled: bool) {
        self.glfw.set_swap_interval(if enabled { glfw::SwapInterval::Sync(1) } else { glfw::SwapInterval::None });
    }

    pub fn set_title(&mut self, title: &str) {
        self.handle.set_title(title);
    }

    pub fn set_cursor_locked(&mut self, locked: bool) {
        self.handle.set_cursor_mode(if locked { glfw::CursorMode::Disabled } else { glfw::CursorMode::Normal });
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn get_aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn get_time(&self) -> f64 {
        self.glfw.get_time()
    }

    // Called by Input to register its own event polling on this window's handle
    pub fn set_key_polling(&mut self, enabled: bool) {
        self.handle.set_key_polling(enabled);
    }

    pub fn set_mouse_button_polling(&mut self, enabled: bool) {
        self.handle.set_mouse_button_polling(enabled);
    }

    pub fn set_cursor_pos_polling(&mut self, enabled: bool) {
        self.handle.set_cursor_pos_polling(enabled);
    }
}
