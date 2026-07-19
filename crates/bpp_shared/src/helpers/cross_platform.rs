/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

pub struct Math;

impl Math {
    pub fn min<T: PartialOrd>(a: T, b: T) -> T {
        if a < b { a } else { b }
    }

    pub fn max<T: PartialOrd>(a: T, b: T) -> T {
        if a > b { a } else { b }
    }
}
