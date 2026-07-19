/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Mojang Studios (2011)
*/

use crate::constants::{CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::enums::blocks::{
    BLOCK_AIR, BLOCK_DIRT, BLOCK_GRASS, BLOCK_LAVA_FLOWING, BLOCK_LAVA_STILL, BLOCK_NETHERRACK, BLOCK_STONE,
    BLOCK_WATER_FLOWING, BLOCK_WATER_STILL,
};
use crate::helpers::java::java_math::{JavaMath, MathHelper};
use crate::helpers::java::java_random::Random;
use crate::numeric_structs::{Int2, Int3, VEC3_ZERO, Vec3};
use crate::world::chunk::Chunk;

/**
 * @brief Used to carve caves into the world
 *
 */
pub struct CaveGenerator {
    rand: Random,
    pub is_nether_cave: bool,
}

impl CaveGenerator {
    const CARVE_EXTENT_LIMIT: i32 = 8;

    pub fn new(is_nether_cave: bool) -> Self {
        Self { rand: Random::new(), is_nether_cave }
    }

    /**
     * @brief Attempts to generate a cave in the current chunk
     *
     * @param chunk The chunk to carve caves into
     * @param seed  The world seed
     */
    pub fn generate_caves_for_chunk(&mut self, chunk: &mut Chunk, seed: i64) {
        let carve_extent = Self::CARVE_EXTENT_LIMIT;
        self.rand.set_seed(seed);
        let x_offset = self.rand.next_long() / 2i64 * 2i64 + 1i64;
        let z_offset = self.rand.next_long() / 2i64 * 2i64 + 1i64;

        // Use unsigned arithmetic to avoid overflow UB
        let x_offset_u = x_offset as u64;
        let z_offset_u = z_offset as u64;
        let seed_u = seed as u64;

        for c_x_offset in (chunk.cpos.x - carve_extent)..=(chunk.cpos.x + carve_extent) {
            for c_z_offset in (*chunk.cpos.z() - carve_extent)..=(*chunk.cpos.z() + carve_extent) {
                let x_part = (c_x_offset as i64 as u64).wrapping_mul(x_offset_u);
                let z_part = (c_z_offset as i64 as u64).wrapping_mul(z_offset_u);
                let combined = x_part.wrapping_add(z_part) ^ seed_u;
                self.rand.set_seed(combined as i64);
                self.generate_caves(chunk, Int2::new(c_x_offset, c_z_offset));
            }
        }
    }

    pub fn generate_caves(&mut self, chunk: &mut Chunk, chunk_offset: Int2) {
        let cave_bound_a = self.rand.next_int_bound(if self.is_nether_cave { 10 } else { 40 });
        let cave_bound_b = self.rand.next_int_bound(cave_bound_a + 1);
        let mut number_of_caves = self.rand.next_int_bound(cave_bound_b + 1);
        if self.rand.next_int_bound(if self.is_nether_cave { 5 } else { 15 }) != 0 {
            number_of_caves = 0;
        }

        for _cave_index in 0..number_of_caves {
            let mut offset = VEC3_ZERO;
            offset.x = f64::from(chunk_offset.x * CHUNK_WIDTH + self.rand.next_int_bound(CHUNK_WIDTH));
            offset.y = if self.is_nether_cave {
                f64::from(self.rand.next_int_bound(128))
            } else {
                let inner = self.rand.next_int_bound(120);
                f64::from(self.rand.next_int_bound(inner + 8))
            };
            offset.z = f64::from(chunk_offset.y * CHUNK_WIDTH + self.rand.next_int_bound(CHUNK_WIDTH));
            let mut number_of_nodes = 1;
            if self.rand.next_int_bound(4) == 0 {
                self.carve_cave(chunk, offset);
                number_of_nodes += self.rand.next_int_bound(4);
            }

            for _node_index in 0..number_of_nodes {
                let carve_yaw = self.rand.next_float() * JavaMath::PI_FLOAT * 2.0;
                let carve_pitch = (self.rand.next_float() - 0.5) * 2.0 / 8.0;
                let tunnel_radius = self.rand.next_float() * 2.0 + self.rand.next_float();
                self.carve_cave_full(
                    chunk,
                    offset,
                    if self.is_nether_cave { tunnel_radius * 2.0 } else { tunnel_radius },
                    carve_yaw,
                    carve_pitch,
                    0,
                    0,
                    1.0,
                );
            }
        }
    }

    pub fn carve_cave(&mut self, chunk: &mut Chunk, offset: Vec3) {
        let tunnel_radius = 1.0 + self.rand.next_float() * 6.0;
        self.carve_cave_full(chunk, offset, tunnel_radius, 0.0, 0.0, -1, -1, 0.5);
    }

    pub fn carve_cave_full(
        &mut self,
        chunk: &mut Chunk,
        mut offset: Vec3,
        tunnel_radius: f32,
        mut carve_yaw: f32,
        mut carve_pitch: f32,
        mut tunnel_step: i32,
        mut tunnel_length: i32,
        vertical_scale: f64,
    ) {
        let chunk_center_x = f64::from(chunk.cpos.x * CHUNK_WIDTH) + CHUNK_WIDTH as f64 * 0.5;
        let chunk_center_z = f64::from(*chunk.cpos.z() * CHUNK_WIDTH) + CHUNK_WIDTH as f64 * 0.5;
        let mut pitch_vel = 0.0f32;
        let mut yaw_vel = 0.0f32;
        let mut rand2 = Random::with_seed(self.rand.next_long());

        if tunnel_length <= 0 {
            let max_tunnel_length = Self::CARVE_EXTENT_LIMIT * CHUNK_WIDTH - CHUNK_WIDTH;
            tunnel_length = max_tunnel_length - rand2.next_int_bound(max_tunnel_length / 4);
        }

        let mut branch_tunnel = false;
        if tunnel_step == -1 {
            tunnel_step = tunnel_length / 2;
            branch_tunnel = true;
        }

        let branch_point = rand2.next_int_bound(tunnel_length / 2) + tunnel_length / 4;

        let tunnel_steepness = rand2.next_int_bound(6) == 0;
        while tunnel_step < tunnel_length {
            let radius_xz = 1.5
                + f64::from(
                    MathHelper::sin(tunnel_step as f32 * JavaMath::PI_FLOAT / tunnel_length as f32) * tunnel_radius * 1.0,
                );
            let radius_y = radius_xz * vertical_scale;
            let p_cos = MathHelper::cos(carve_pitch);
            let p_sin = MathHelper::sin(carve_pitch);
            offset.x += f64::from(MathHelper::cos(carve_yaw) * p_cos);
            offset.y += f64::from(p_sin);
            offset.z += f64::from(MathHelper::sin(carve_yaw) * p_cos);

            carve_pitch *= if tunnel_steepness { 0.92 } else { 0.7 };

            carve_pitch += yaw_vel * 0.1;
            carve_yaw += pitch_vel * 0.1;
            yaw_vel *= 0.9;
            pitch_vel *= 12.0 / 16.0;
            yaw_vel += (rand2.next_float() - rand2.next_float()) * rand2.next_float() * 2.0;
            pitch_vel += (rand2.next_float() - rand2.next_float()) * rand2.next_float() * 4.0;

            if !branch_tunnel && tunnel_step == branch_point && tunnel_radius > 1.0 {
                self.carve_cave_full(
                    chunk,
                    offset,
                    rand2.next_float() * 0.5 + 0.5,
                    carve_yaw - JavaMath::PI_FLOAT * 0.5,
                    carve_pitch / 3.0,
                    tunnel_step,
                    tunnel_length,
                    1.0,
                );
                self.carve_cave_full(
                    chunk,
                    offset,
                    rand2.next_float() * 0.5 + 0.5,
                    carve_yaw + JavaMath::PI_FLOAT * 0.5,
                    carve_pitch / 3.0,
                    tunnel_step,
                    tunnel_length,
                    1.0,
                );
                return;
            }

            if branch_tunnel || rand2.next_int_bound(4) != 0 {
                let dx = offset.x - chunk_center_x;
                let dz = offset.z - chunk_center_z;
                let dist = f64::from(tunnel_length - tunnel_step);
                let limit = f64::from(tunnel_radius + 2.0 + 16.0);
                if (dx * dx + dz * dz - dist * dist) > (limit * limit) {
                    return;
                }

                if offset.x >= chunk_center_x - 16.0 - radius_xz * 2.0
                    && offset.z >= chunk_center_z - 16.0 - radius_xz * 2.0
                    && offset.x <= chunk_center_x + 16.0 + radius_xz * 2.0
                    && offset.z <= chunk_center_z + 16.0 + radius_xz * 2.0
                {
                    let mut x_min = MathHelper::floor_double(offset.x - radius_xz) - chunk.cpos.x * 16 - 1;
                    let mut x_max = MathHelper::floor_double(offset.x + radius_xz) - chunk.cpos.x * 16 + 1;
                    let mut y_min = MathHelper::floor_double(offset.y - radius_y) - 1;
                    let mut y_max = MathHelper::floor_double(offset.y + radius_y) + 1;
                    let mut z_min = MathHelper::floor_double(offset.z - radius_xz) - *chunk.cpos.z() * 16 - 1;
                    let mut z_max = MathHelper::floor_double(offset.z + radius_xz) - *chunk.cpos.z() * 16 + 1;

                    if x_min < 0 {
                        x_min = 0;
                    }
                    if x_max > 16 {
                        x_max = 16;
                    }
                    if y_min < 1 {
                        y_min = 1;
                    }
                    if y_max > 120 {
                        y_max = 120;
                    }
                    if z_min < 0 {
                        z_min = 0;
                    }
                    if z_max > 16 {
                        z_max = 16;
                    }

                    // Check for water before carving
                    let mut fluid_is_present = false;
                    let mut block_x = x_min;
                    while !fluid_is_present && block_x < x_max {
                        let mut block_z = z_min;
                        while !fluid_is_present && block_z < z_max {
                            let mut block_y = y_max + 1;
                            while !fluid_is_present && block_y >= y_min - 1 {
                                if block_y >= 0 && block_y < CHUNK_HEIGHT {
                                    let block_type = chunk.get_block(Int3::new(block_x, block_y, block_z));
                                    // Overworld caver check
                                    if !self.is_nether_cave
                                        && (block_type == BLOCK_WATER_FLOWING || block_type == BLOCK_WATER_STILL)
                                    {
                                        fluid_is_present = true;
                                    }
                                    // Nether caver check
                                    if self.is_nether_cave
                                        && (block_type == BLOCK_LAVA_FLOWING || block_type == BLOCK_LAVA_STILL)
                                    {
                                        fluid_is_present = true;
                                    }
                                    // Skip interior, only check the shell
                                    if block_y != y_min - 1
                                        && block_x != x_min
                                        && block_x != x_max - 1
                                        && block_z != z_min
                                        && block_z != z_max - 1
                                    {
                                        block_y = y_min;
                                    }
                                }
                                block_y -= 1;
                            }
                            block_z += 1;
                        }
                        block_x += 1;
                    }

                    if !fluid_is_present {
                        for block_x in x_min..x_max {
                            let center_dx = (f64::from(block_x + chunk.cpos.x * 16) + 0.5 - offset.x) / radius_xz;

                            for block_z in z_min..z_max {
                                let center_dz = (f64::from(block_z + *chunk.cpos.z() * 16) + 0.5 - offset.z) / radius_xz;

                                if center_dx * center_dx + center_dz * center_dz < 1.0 {
                                    // Doesn't exist in nether caver
                                    let mut is_grass = false;
                                    for block_y in (y_min..y_max).rev() {
                                        let bpos = Int3::new(block_x, block_y + 1, block_z);
                                        let center_dy = (f64::from(block_y) + 0.5 - offset.y) / radius_y;
                                        if center_dy > -0.7
                                            && center_dx * center_dx + center_dy * center_dy + center_dz * center_dz < 1.0
                                        {
                                            let block_type = chunk.get_block(bpos);
                                            // Nether caver behavior
                                            // Dirt and grass check is most likely irrelevant,
                                            // but it still exists in the Vanilla Nether caver
                                            if self.is_nether_cave
                                                && (block_type == BLOCK_NETHERRACK
                                                    || block_type == BLOCK_DIRT
                                                    || block_type == BLOCK_GRASS)
                                            {
                                                chunk.set_block(bpos, BLOCK_AIR);
                                                continue;
                                            }
                                            // Overworld caver behavior
                                            if block_type == BLOCK_GRASS {
                                                is_grass = true;
                                            }
                                            if block_type != BLOCK_STONE && block_type != BLOCK_DIRT && block_type != BLOCK_GRASS
                                            {
                                                continue;
                                            }
                                            if block_y < 10 {
                                                chunk.set_block(bpos, BLOCK_LAVA_FLOWING);
                                                continue;
                                            }
                                            chunk.set_block(bpos, BLOCK_AIR);
                                            if !is_grass {
                                                continue;
                                            }
                                            let below = Int3::new(bpos.x, block_y, bpos.z);
                                            if chunk.get_block(below) == BLOCK_DIRT {
                                                chunk.set_block(below, BLOCK_GRASS);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if branch_tunnel {
                            break;
                        }
                    }
                }
            }

            tunnel_step += 1;
        }
    }
}
