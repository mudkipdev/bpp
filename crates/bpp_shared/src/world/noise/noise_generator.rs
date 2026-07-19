/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use crate::helpers::java::java_random::Random;
use crate::numeric_structs::{VEC3_ZERO, Vec3};

/// @brief The base Noise generator object that splits into Perlin and Simplex noise
pub struct NoiseGenerator {
    pub permutations: [i32; 512],
    pub coordinate: Vec3,
}

impl NoiseGenerator {
    pub fn new() -> Self {
        Self { permutations: [0; 512], coordinate: VEC3_ZERO }
    }

    pub fn with_random(_rand: &mut Random) -> Self {
        Self { permutations: [0; 512], coordinate: VEC3_ZERO }
    }
}

impl Default for NoiseGenerator {
    fn default() -> Self {
        Self::new()
    }
}
