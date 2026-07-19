/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Mojang Studios (2011)
*/

// A recreation of the the Infdev 20100227-1433 Perlin noise function
use crate::helpers::java::java_math::{double_to_int32, fade, grad2d, grad3d, lerp};
use crate::helpers::java::java_random::Random;
use crate::numeric_structs::{Int3, VEC3_ZERO, Vec2, Vec3};

/// @brief A faithful reimplementation of the Infdev and Beta perlin noise generator
pub struct NoisePerlin {
    pub permutations: [i32; 512],
    pub coordinate: Vec3,
}

impl NoisePerlin {
    pub fn new() -> Self {
        let mut rand = Random::new();
        let mut noise = Self { permutations: [0; 512], coordinate: VEC3_ZERO };
        noise.init_perm_table(&mut rand);
        noise
    }

    /// @brief Construct a new Noise Perlin object
    ///
    /// @param rand The random number generator that should be used
    pub fn with_random(rand: &mut Random) -> Self {
        let mut noise = Self { permutations: [0; 512], coordinate: VEC3_ZERO };
        noise.init_perm_table(rand);
        noise
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

    /// @brief This is a rather standard implementation of "Improved Perlin Noise",
    ///        as described by Ken Perlin in 2002
    ///        This version is mainly used by the infdev generator
    ///        but Beta still implements and uses it for some things,
    ///        namely the nether
    ///
    /// @param pos Coordinate at which to sample the noise
    /// @return Noise value
    fn generate_noise_base(&self, pos: Vec3) -> f64 {
        let mut pos = pos;
        pos.x += self.coordinate.x;
        pos.y += self.coordinate.y;
        pos.z += self.coordinate.z;
        // The farlands are caused by this getting cast to a 32-Bit Integer.
        // Change these int32_t to int64_t to fix the farlands in Infdev
        let mut x_int = double_to_int32(pos.x);
        let mut y_int = double_to_int32(pos.y);
        let mut z_int = double_to_int32(pos.z);
        if pos.x < x_int as f64 {
            x_int -= 1;
        }
        if pos.y < y_int as f64 {
            y_int -= 1;
        }
        if pos.z < z_int as f64 {
            z_int -= 1;
        }

        let mut x_index = (x_int & 255) as usize;
        let mut y_index = (y_int & 255) as usize;
        let z_index = (z_int & 255) as usize;

        pos.x -= x_int as f64;
        pos.y -= y_int as f64;
        pos.z -= z_int as f64;
        let w = fade(pos.x);
        let v = fade(pos.y);
        let u = fade(pos.z);
        let mut perm_xy = (self.permutations[x_index] as usize) + y_index;
        let perm_xyz = (self.permutations[perm_xy] as usize) + z_index;
        // Some of the following code is weird,
        // probably because it got optimized by Java to use
        // fewer variables or Notch did this to be efficient
        perm_xy = (self.permutations[perm_xy + 1] as usize) + z_index;
        x_index = (self.permutations[x_index + 1] as usize) + y_index;
        y_index = (self.permutations[x_index] as usize) + z_index;
        x_index = (self.permutations[x_index + 1] as usize) + z_index;
        lerp(
            u,
            lerp(
                v,
                lerp(
                    w,
                    grad3d(self.permutations[perm_xyz], pos.x, pos.y, pos.z),
                    grad3d(self.permutations[y_index], pos.x - 1.0, pos.y, pos.z),
                ),
                lerp(
                    w,
                    grad3d(self.permutations[perm_xy], pos.x, pos.y - 1.0, pos.z),
                    grad3d(self.permutations[x_index], pos.x - 1.0, pos.y - 1.0, pos.z),
                ),
            ),
            lerp(
                v,
                lerp(
                    w,
                    grad3d(self.permutations[perm_xyz + 1], pos.x, pos.y, pos.z - 1.0),
                    grad3d(self.permutations[y_index + 1], pos.x - 1.0, pos.y, pos.z - 1.0),
                ),
                lerp(
                    w,
                    grad3d(self.permutations[perm_xy + 1], pos.x, pos.y - 1.0, pos.z - 1.0),
                    grad3d(self.permutations[x_index + 1], pos.x - 1.0, pos.y - 1.0, pos.z - 1.0),
                ),
            ),
        )
    }

    pub fn generate_noise_vec2(&self, coord: Vec2) -> f64 {
        self.generate_noise_base(Vec3::new(coord.x, coord.y, 0.0))
    }

    pub fn generate_noise_vec3(&self, coord: Vec3) -> f64 {
        self.generate_noise_base(coord)
    }

    /// @brief The main noise generator employed by the Beta 1.7.3 world generator
    ///
    /// @param noiseField the vector the noise will be written to
    /// @param offset The positional offset within the perlin noise that'll be rendered
    /// @param size The size of the volume that'll be saved the noise field
    /// @param scale The scale of the perlin noise equation
    /// @param amplitude The amplitude multiplier of the perlin noise function
    pub fn generate_noise(&self, noise_field: &mut Vec<f64>, offset: Vec3, size: Int3, scale: Vec3, amplitude: f64) {
        if size.y == 1 {
            let mut index = 0usize;
            let inv_amp = 1.0 / amplitude;

            for x in 0..size.x {
                let mut fx = (offset.x + x as f64) * scale.x + self.coordinate.x;
                let mut ix = double_to_int32(fx);
                if fx < ix as f64 {
                    ix -= 1;
                }
                let px = (ix & 255) as usize;
                fx -= ix as f64;
                let u = fade(fx);

                for z in 0..size.z {
                    let mut fz = (offset.z + z as f64) * scale.z + self.coordinate.z;
                    let mut iz = double_to_int32(fz);
                    if fz < iz as f64 {
                        iz -= 1;
                    }
                    let pz = (iz & 255) as usize;
                    fz -= iz as f64;
                    let w = fade(fz);

                    let a = self.permutations[px] as usize;
                    let aa = (self.permutations[a] as usize) + pz;
                    let b = self.permutations[px + 1] as usize;
                    let ba = (self.permutations[b] as usize) + pz;

                    let x1 = lerp(u, grad2d(self.permutations[aa], fx, fz), grad3d(self.permutations[ba], fx - 1.0, 0.0, fz));

                    let x2 = lerp(
                        u,
                        grad3d(self.permutations[aa + 1], fx, 0.0, fz - 1.0),
                        grad3d(self.permutations[ba + 1], fx - 1.0, 0.0, fz - 1.0),
                    );

                    let result = lerp(w, x1, x2);
                    noise_field[index] += result * inv_amp;
                    index += 1;
                }
            }
        } else {
            let mut index = 0usize;
            let inv_amp = 1.0 / amplitude;
            let mut last_perm_y: i32 = -1;

            let mut lerp_ax = 0.0;
            let mut lerp_bx = 0.0;
            let mut lerp_ay = 0.0;
            let mut lerp_by = 0.0;

            for x in 0..size.x {
                let mut fx = (offset.x + x as f64) * scale.x + self.coordinate.x;
                let mut ix = double_to_int32(fx);
                if fx < ix as f64 {
                    ix -= 1;
                }
                let px = (ix & 255) as usize;
                fx -= ix as f64;
                let u = fade(fx);

                for z in 0..size.z {
                    let mut fz = (offset.z + z as f64) * scale.z + self.coordinate.z;
                    let mut iz = double_to_int32(fz);
                    if fz < iz as f64 {
                        iz -= 1;
                    }
                    let pz = (iz & 255) as usize;
                    fz -= iz as f64;
                    let w = fade(fz);

                    for y in 0..size.y {
                        let mut fy = (offset.y + y as f64) * scale.y + self.coordinate.y;
                        let mut iy = double_to_int32(fy);
                        if fy < iy as f64 {
                            iy -= 1;
                        }
                        let py = iy & 255;
                        fy -= iy as f64;
                        let v = fade(fy);

                        if y == 0 || py != last_perm_y {
                            last_perm_y = py;
                            let py = py as usize;

                            let aa_base = (self.permutations[px] as usize) + py;
                            let aa = (self.permutations[aa_base] as usize) + pz;
                            let ab = (self.permutations[aa_base + 1] as usize) + pz;
                            let ba_base = (self.permutations[px + 1] as usize) + py;
                            let ba = (self.permutations[ba_base] as usize) + pz;
                            let bb = (self.permutations[ba_base + 1] as usize) + pz;

                            lerp_ax = lerp(u, grad3d(self.permutations[aa], fx, fy, fz), grad3d(self.permutations[ba], fx - 1.0, fy, fz));

                            lerp_bx = lerp(
                                u,
                                grad3d(self.permutations[ab], fx, fy - 1.0, fz),
                                grad3d(self.permutations[bb], fx - 1.0, fy - 1.0, fz),
                            );

                            lerp_ay = lerp(
                                u,
                                grad3d(self.permutations[aa + 1], fx, fy, fz - 1.0),
                                grad3d(self.permutations[ba + 1], fx - 1.0, fy, fz - 1.0),
                            );

                            lerp_by = lerp(
                                u,
                                grad3d(self.permutations[ab + 1], fx, fy - 1.0, fz - 1.0),
                                grad3d(self.permutations[bb + 1], fx - 1.0, fy - 1.0, fz - 1.0),
                            );
                        }

                        let i1 = lerp(v, lerp_ax, lerp_bx);
                        let i2 = lerp(v, lerp_ay, lerp_by);
                        let result = lerp(w, i1, i2);

                        noise_field[index] += result * inv_amp;
                        index += 1;
                    }
                }
            }
        }
    }
}

impl Default for NoisePerlin {
    fn default() -> Self {
        Self::new()
    }
}
