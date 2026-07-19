/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Mojang Studios (2011)
*/

// A recreation of the the Infdev 20100227-1433 Perlin noise function
use crate::helpers::java::java_math::double_to_int32;
use crate::helpers::java::java_random::Random;
use crate::numeric_structs::{Int32_2, VEC3_ZERO, Vec2, Vec3};

/// @brief A faithful reimplementation of the Beta-era simplex noise generator, often used for Biome generation
pub struct NoiseSimplex {
    pub permutations: [i32; 512],
    pub coordinate: Vec3,
    gradients: [[i32; 3]; 12],
    skewing: f64,
    unskewing: f64,
}

impl NoiseSimplex {
    pub fn new() -> Self {
        let mut rand = Random::new();
        let mut noise = Self::blank();
        noise.init_perm_table(&mut rand);
        noise
    }

    pub fn with_random(rand: &mut Random) -> Self {
        let mut noise = Self::blank();
        noise.init_perm_table(rand);
        noise
    }

    fn blank() -> Self {
        Self {
            permutations: [0; 512],
            coordinate: VEC3_ZERO,
            gradients: [
                [1, 1, 0],
                [-1, 1, 0],
                [1, -1, 0],
                [-1, -1, 0],
                [1, 0, 1],
                [-1, 0, 1],
                [1, 0, -1],
                [-1, 0, -1],
                [0, 1, 1],
                [0, -1, 1],
                [0, 1, -1],
                [0, -1, -1],
            ],
            skewing: 0.5 * (3.0f64.sqrt() - 1.0),
            unskewing: (3.0 - 3.0f64.sqrt()) / 6.0,
        }
    }

    fn init_perm_table(&mut self, rand: &mut Random) {
        self.coordinate.x = rand.next_double() * 256.0;
        self.coordinate.y = rand.next_double() * 256.0;
        self.coordinate.z = rand.next_double() * 256.0;

        for i in 0..256usize {
            self.permutations[i] = i as i32;
        }

        for i in 0..256usize {
            let j = (rand.next_int_bound(256 - i as i32) + i as i32) as usize;
            self.permutations.swap(i, j);
            self.permutations[i + 256] = self.permutations[i];
        }
    }

    pub fn generate_noise(&self, values: &mut Vec<f64>, p_offset: Vec2, p_size: Int32_2, p_scale: Vec2, amplitude: f64) {
        let mut index = 0usize;

        for x_i in 0..p_size.x {
            let x_pos = (p_offset.x + x_i as f64) * p_scale.x + self.coordinate.x;

            for y_i in 0..p_size.y {
                let y_pos = (p_offset.y + y_i as f64) * p_scale.y + self.coordinate.y;
                let skew = (x_pos + y_pos) * self.skewing;
                let x0 = wrap(x_pos + skew);
                let y0 = wrap(y_pos + skew);
                let unskewed = (x0 + y0) as f64 * self.unskewing;
                let x0a = x0 as f64 - unskewed;
                let y0a = y0 as f64 - unskewed;
                let x0b = x_pos - x0a;
                let y0b = y_pos - y0a;
                let i: i8;
                let j: i8;
                if x0b > y0b {
                    i = 1;
                    j = 0;
                } else {
                    i = 0;
                    j = 1;
                }

                let x0c = x0b - i as f64 + self.unskewing;
                let y0c = y0b - j as f64 + self.unskewing;
                let x1c = x0b - 1.0 + 2.0 * self.unskewing;
                let y1c = y0b - 1.0 + 2.0 * self.unskewing;
                let x_int = (x0 & 255) as usize;
                let y_int = (y0 & 255) as usize;
                let grad0 = self.permutations[x_int + self.permutations[y_int] as usize] % 12;
                let grad1 = self.permutations[x_int + i as usize + self.permutations[y_int + j as usize] as usize] % 12;
                let grad2 = self.permutations[x_int + 1 + self.permutations[y_int + 1] as usize] % 12;
                let mut term0 = 0.5 - x0b * x0b - y0b * y0b;
                let contrib0 = if term0 < 0.0 {
                    0.0
                } else {
                    term0 *= term0;
                    term0 * term0 * dot_prod(&self.gradients[grad0 as usize], x0b, y0b)
                };

                let mut term1 = 0.5 - x0c * x0c - y0c * y0c;
                let contrib1 = if term1 < 0.0 {
                    0.0
                } else {
                    term1 *= term1;
                    term1 * term1 * dot_prod(&self.gradients[grad1 as usize], x0c, y0c)
                };

                let mut term2 = 0.5 - x1c * x1c - y1c * y1c;
                let contrib2 = if term2 < 0.0 {
                    0.0
                } else {
                    term2 *= term2;
                    term2 * term2 * dot_prod(&self.gradients[grad2 as usize], x1c, y1c)
                };

                values[index] += 70.0 * (contrib0 + contrib1 + contrib2) * amplitude;
                index += 1;
            }
        }
    }
}

impl Default for NoiseSimplex {
    fn default() -> Self {
        Self::new()
    }
}

pub fn wrap(grad: f64) -> i32 {
    if grad > 0.0 { double_to_int32(grad) } else { double_to_int32(grad) - 1 }
}

pub fn dot_prod(grad: &[i32; 3], x: f64, y: f64) -> f64 {
    grad[0] as f64 * x + grad[1] as f64 * y
}
