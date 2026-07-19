/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use crate::inputs::Input;
use crate::window::Window;

// Shared context passed as the GLFW user pointer so both
// Window and Input callbacks can coexist on one pointer
pub struct GlfwContext {
    pub window: *mut Window,
    pub input: *mut Input,
}

impl Default for GlfwContext {
    fn default() -> Self {
        Self { window: std::ptr::null_mut(), input: std::ptr::null_mut() }
    }
}
