/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Mojang Studios (2011)
*/

use crate::helpers::java::java_random::Random;
use crate::numeric_structs::{Int32_2, Int32_3, Vec2, Vec3};
use crate::world::noise::noise_perlin::NoisePerlin;

pub struct NoiseOctavesPerlin {
    octaves: i32,
    generator_collection: Vec<NoisePerlin>,
}

impl NoiseOctavesPerlin {
    pub fn new() -> Self {
        Self { octaves: 0, generator_collection: Vec::new() }
    }

    pub fn with_octaves(poctaves: i32) -> Self {
        let mut generator_collection = Vec::new();
        for _ in 0..poctaves {
            generator_collection.push(NoisePerlin::new());
        }
        Self { octaves: poctaves, generator_collection }
    }

    pub fn with_random(rand: &mut Random, poctaves: i32) -> Self {
        let mut generator_collection = Vec::new();
        for _ in 0..poctaves {
            generator_collection.push(NoisePerlin::with_random(rand));
        }
        Self { octaves: poctaves, generator_collection }
    }

    // func_647_a
    pub fn generate_octaves_scalar(&mut self, offset: Vec2) -> f64 {
        let mut value = 0.0;
        let mut scale = 1.0;
        for i in 0..self.octaves as usize {
            value += self.generator_collection[i].generate_noise_vec2(offset * scale) / scale;
            scale /= 2.0;
        }
        value
    }

    // generateNoiseOctaves
    pub fn generate_octaves(&mut self, noise_field: &mut Vec<f64>, coordinate: Vec3, size: Int32_3, p_scale: Vec3) {
        if noise_field.is_empty() {
            noise_field.resize((size.x * size.y * size.z) as usize, 0.0);
        } else {
            for value in noise_field.iter_mut() {
                *value = 0.0;
            }
        }

        let mut multiplier = 1.0;
        for octave in 0..self.octaves as usize {
            self.generator_collection[octave].generate_noise(noise_field, coordinate, size, p_scale * multiplier, multiplier);
            multiplier /= 2.0;
        }
    }

    // func_4103_a
    pub fn generate_octaves_2d(&mut self, noise_field: &mut Vec<f64>, offset: Int32_2, size: Int32_2, scale: Vec2, _unused: f64) {
        self.generate_octaves(
            noise_field,
            Vec3::new(offset.x as f64, 10.0, *offset.z() as f64),
            Int32_3::new(size.x, 1, *size.z()),
            Vec3::new(scale.x, 1.0, *scale.z()),
        );
    }
}

impl Default for NoiseOctavesPerlin {
    fn default() -> Self {
        Self::new()
    }
}
