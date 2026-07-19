/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Mojang Studios (2011)
*/

// Acts a breakpoint to aid in debugging
#[macro_export]
macro_rules! debug_break {
    () => {
        ::std::process::abort()
    };
}
