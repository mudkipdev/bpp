/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Mojang Studios (2011)
*/

use crate::helpers::java::java_random::Random;
use crate::numeric_structs::{Int32_2, Vec2};
use crate::world::noise::noise_simplex::NoiseSimplex;

pub struct NoiseOctavesSimplex {
    octaves: i32,
    generator_collection: Vec<NoiseSimplex>,
}

impl NoiseOctavesSimplex {
    pub fn new() -> Self {
        Self { octaves: 0, generator_collection: Vec::new() }
    }

    pub fn with_octaves(poctaves: i32) -> Self {
        let mut generator_collection = Vec::new();
        for _ in 0..poctaves {
            generator_collection.push(NoiseSimplex::new());
        }
        Self { octaves: poctaves, generator_collection }
    }

    pub fn with_random(rand: &mut Random, poctaves: i32) -> Self {
        let mut generator_collection = Vec::new();
        for _ in 0..poctaves {
            generator_collection.push(NoiseSimplex::with_random(rand));
        }
        Self { octaves: poctaves, generator_collection }
    }

    pub fn generate_octaves(&mut self, noise_field: &mut Vec<f64>, offset: Int32_2, size: Int32_2, scale: Vec2, lacunarity: f64) {
        self.generate_octaves_vec2(noise_field, Vec2::new(offset.x as f64, offset.y as f64), size, scale, lacunarity);
    }

    pub fn generate_octaves_vec2(&mut self, noise_field: &mut Vec<f64>, offset: Vec2, size: Int32_2, scale: Vec2, lacunarity: f64) {
        self.generate_octaves_full(noise_field, offset, size, scale, lacunarity, 0.5);
    }

    // func_4111_a
    pub fn generate_octaves_full(
        &mut self,
        noise_field: &mut Vec<f64>,
        offset: Vec2,
        size: Int32_2,
        scale: Vec2,
        lacunarity: f64,
        persistence: f64,
    ) {
        let mut scale = scale;
        scale.x /= 1.5;
        scale.y /= 1.5;
        if !noise_field.is_empty() && noise_field.len() as i32 >= size.x * size.y {
            for value in noise_field.iter_mut() {
                *value = 0.0;
            }
        } else {
            noise_field.resize((size.x * size.y) as usize, 0.0);
        }

        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        for octave in 0..self.octaves as usize {
            self.generator_collection[octave].generate_noise(noise_field, offset, size, scale * amplitude, 0.55 / frequency);
            amplitude *= lacunarity;
            frequency *= persistence;
        }
    }
}

impl Default for NoiseOctavesSimplex {
    fn default() -> Self {
        Self::new()
    }
}
