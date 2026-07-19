/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use crate::window::Window;

pub struct InputEvent {
    pub code: i32,
    pub action: glfw::Action, // GLFW_PRESS, GLFW_RELEASE
    pub is_mouse: bool,
}

pub struct Input {
    // Raw event queue — filled by callbacks, drained each tick
    event_queue: Vec<InputEvent>,

    keys: [bool; 1024],
    pressed: [bool; 1024],
    mouse: [bool; 8],
    mouse_pressed: [bool; 8],

    delta_x: f32,
    delta_y: f32,
    last_x: f32,
    last_y: f32,
    first_mouse: bool,
}

impl Input {
    pub fn init(&mut self, window: &mut Window) {
        window.set_key_polling(true);
        window.set_mouse_button_polling(true);
        window.set_cursor_pos_polling(true);
    }

    // Called at the start of a tick — drains queue into stable state
    pub fn drain_events(&mut self) {
        let local = std::mem::take(&mut self.event_queue);

        for e in local {
            if e.is_mouse {
                if e.action == glfw::Action::Press {
                    self.mouse[e.code as usize] = true;
                    self.mouse_pressed[e.code as usize] = true;
                } else if e.action == glfw::Action::Release {
                    self.mouse[e.code as usize] = false;
                }
            } else if e.action == glfw::Action::Press {
                self.keys[e.code as usize] = true;
                self.pressed[e.code as usize] = true;
            } else if e.action == glfw::Action::Release {
                self.keys[e.code as usize] = false;
            }
        }
    }

    pub fn is_key_held(&self, key: i32) -> bool {
        key >= 0 && key < 1024 && self.keys[key as usize]
    }

    pub fn is_key_pressed(&self, key: i32) -> bool {
        key >= 0 && key < 1024 && self.pressed[key as usize]
    }

    pub fn is_mouse_held(&self, btn: i32) -> bool {
        btn >= 0 && btn < 8 && self.mouse[btn as usize]
    }

    pub fn is_mouse_pressed(&self, btn: i32) -> bool {
        btn >= 0 && btn < 8 && self.mouse_pressed[btn as usize]
    }

    pub fn mouse_delta_x(&self) -> f32 {
        self.delta_x
    }

    pub fn mouse_delta_y(&self) -> f32 {
        self.delta_y
    }

    // Called at the end of a tick — clears one-shot flags
    pub fn flush(&mut self) {
        self.pressed = [false; 1024];
        self.mouse_pressed = [false; 8];
        self.delta_x = 0.0;
        self.delta_y = 0.0;
    }

    pub(crate) fn key_callback(&mut self, key: i32, action: glfw::Action) {
        if key < 0 || key >= 1024 {
            return;
        }
        if action == glfw::Action::Repeat {
            return;
        }
        self.event_queue.push(InputEvent { code: key, action, is_mouse: false });
    }

    pub(crate) fn mouse_button_callback(&mut self, btn: i32, action: glfw::Action) {
        if btn < 0 || btn >= 8 {
            return;
        }
        self.event_queue.push(InputEvent { code: btn, action, is_mouse: true });
    }

    pub(crate) fn cursor_callback(&mut self, x: f64, y: f64) {
        if self.first_mouse {
            self.last_x = x as f32;
            self.last_y = y as f32;
            self.first_mouse = false;
            return;
        }
        self.delta_x += x as f32 - self.last_x;
        self.delta_y += self.last_y - y as f32;
        self.last_x = x as f32;
        self.last_y = y as f32;
    }
}

impl Default for Input {
    fn default() -> Self {
        Self {
            event_queue: Vec::new(),
            keys: [false; 1024],
            pressed: [false; 1024],
            mouse: [false; 8],
            mouse_pressed: [false; 8],
            delta_x: 0.0,
            delta_y: 0.0,
            last_x: 0.0,
            last_y: 0.0,
            first_mouse: true,
        }
    }
}
