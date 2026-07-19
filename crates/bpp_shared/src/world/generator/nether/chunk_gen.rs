/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Mojang Studios (2011)
*/

use crate::constants::{CHUNK_HEIGHT, CHUNK_WIDTH, NETHER_BIOME_LAVA_LEVEL, NETHER_LAVA_LEVEL};
use crate::enums::blocks::{
    BLOCK_AIR, BLOCK_BEDROCK, BLOCK_GRAVEL, BLOCK_LAVA_FLOWING, BLOCK_LAVA_STILL, BLOCK_MUSHROOM_BROWN,
    BLOCK_MUSHROOM_RED, BLOCK_NETHERRACK, BLOCK_SOULSAND,
};
use crate::helpers::java::java_math::{JavaMath, double_to_int32};
use crate::helpers::java::java_random::Random;
use crate::numeric_structs::{Int3, Int32_2, Vec3};
use crate::world::chunk::Chunk;
use crate::world::generator::generator::{Generator, GeneratorBehavior};
use crate::world::generator::shared::cave_gen::CaveGenerator;
use crate::world::generator::shared::feature_gen::{FeatureGenerator, WorldWrapper};
use crate::world::noise::noise_octaves_perlin::NoiseOctavesPerlin;

/// @brief A faithful reimplementation of the Beta 1.7.3 Nether Generator
pub struct NetherGenerator {
    pub base: Generator,

    // Perlin Noise Generators
    pub low_noise_gen: NoiseOctavesPerlin,
    pub high_noise_gen: NoiseOctavesPerlin,
    pub selector_noise_gen: NoiseOctavesPerlin,
    pub sand_gravel_noise_gen: NoiseOctavesPerlin,
    pub stone_noise_gen: NoiseOctavesPerlin,
    pub continentalness_noise_gen: NoiseOctavesPerlin,
    pub depth_noise_gen: NoiseOctavesPerlin,

    // Stored noise Fields
    pub terrain_noise_field: Vec<f64>,
    pub low_noise_field: Vec<f64>,
    pub high_noise_field: Vec<f64>,
    pub selector_noise_field: Vec<f64>,
    pub continentalness_noise_field: Vec<f64>,
    pub depth_noise_field: Vec<f64>,

    pub sand_noise: Vec<f64>,
    pub gravel_noise: Vec<f64>,
    pub stone_noise: Vec<f64>,

    // Cave Gen
    pub caver: CaveGenerator,
}

impl NetherGenerator {
    /// @brief Construct a new Beta 1.7.3 Nether Generator
    ///
    /// @param pSeed The seed of the generated world
    /// @param pWorld The world that the NetherGenerator belongs to
    pub fn new(seed: i64) -> Self {
        // Tell caver it's a nether caver
        let mut base = Generator::new(seed);
        base.rand = Random::with_seed(base.seed);
        // Init Terrain Noise
        let low_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 16);
        let high_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 16);
        let selector_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 8);
        let sand_gravel_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 4);
        let stone_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 4);
        let continentalness_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 10);
        let depth_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 16);

        Self {
            base,
            low_noise_gen,
            high_noise_gen,
            selector_noise_gen,
            sand_gravel_noise_gen,
            stone_noise_gen,
            continentalness_noise_gen,
            depth_noise_gen,
            terrain_noise_field: Vec::new(),
            low_noise_field: Vec::new(),
            high_noise_field: Vec::new(),
            selector_noise_field: Vec::new(),
            continentalness_noise_field: Vec::new(),
            depth_noise_field: Vec::new(),
            sand_noise: Vec::new(),
            gravel_noise: Vec::new(),
            stone_noise: Vec::new(),
            caver: CaveGenerator::new(true),
        }
    }
}

impl GeneratorBehavior for NetherGenerator {
    fn base(&self) -> &Generator {
        &self.base
    }
    fn base_mut(&mut self) -> &mut Generator {
        &mut self.base
    }

    /// @brief Generate a non-populated chunk
    ///
    /// @param chunkPos The x,z coordinate of the chunk
    /// @return std::shared_ptr<Chunk>
    fn generate_chunk(&mut self, chunk: &mut Chunk) {
        self.base
            .rand
            .set_seed((i64::from(chunk.cpos.x)).wrapping_mul(341873128712).wrapping_add((i64::from(*chunk.cpos.z())).wrapping_mul(132897987541)));

        // Allocate empty chunk
        chunk.clear();
        // Generate the Terrain, minus any caves, as just stone
        self.generate_terrain(chunk);
        // Replace some of the stone with Biome-appropriate blocks
        self.replace_blocks_for_biome(chunk);
        // Carve caves
        self.caver.generate_caves_for_chunk(chunk, self.base.seed);
        // Generate heightmap
        chunk.generate_height_map();

        chunk.is_modified = true;
    }

    /// @brief Populates the specified chunk with biome-specific features.
    ///
    /// Direct port of ChunkProviderHell.populate() from Beta 1.7.3.
    /// Section order, rand call counts, and coordinate offsets all
    /// match the Java source exactly.
    fn populate_chunk(&mut self, cpos: Int32_2, world: &mut WorldWrapper) -> bool {
        let block_x = cpos.x * CHUNK_WIDTH;
        let block_z = *cpos.z() * CHUNK_WIDTH;
        // TODO: The nether does not initialize its prng values,
        // meaning that *technically* they're fully up to random chance.
        // It just happens that this random chance is somewhat consistent, apparently.
        // So... figure that out, if possible. Probably just some chunk ordering tomfuckery.
        self.base.rand.set_seed(world.get_seed());
        let x_salt: i64 = self.base.rand.next_long() / 2 * 2 + 1;
        let z_salt: i64 = self.base.rand.next_long() / 2 * 2 + 1;
        // Use unsigned arithmetic to avoid overflow UB
        let x_salt_u = x_salt as u64;
        let z_salt_u = z_salt as u64;
        let x_part = (i64::from(cpos.x) as u64).wrapping_mul(x_salt_u);
        let z_part = (i64::from(*cpos.z()) as u64).wrapping_mul(z_salt_u);
        let combined = x_part.wrapping_add(z_part) ^ (world.get_seed() as u64);

        self.base.rand.set_seed(combined as i64);

        let mut coord = Int3::default();
        // Generate single-block lava streams
        for _attempt in 0..8 {
            coord.x = (f64::from(block_x + self.base.rand.next_int_bound(CHUNK_WIDTH)) + f64::from(CHUNK_WIDTH) * 0.5) as i32;
            coord.y = self.base.rand.next_int_bound(CHUNK_HEIGHT - 8) + 4; // 120
            coord.z = (f64::from(block_z + self.base.rand.next_int_bound(CHUNK_WIDTH)) + f64::from(CHUNK_WIDTH) * 0.5) as i32;
            FeatureGenerator::with_type(BLOCK_LAVA_FLOWING).generate_nether_liquid(world, &mut self.base.rand, coord);
        }

        // Generate fire patch
        let fire_bound = self.base.rand.next_int_bound(10) + 1;
        let max_fire_attempts = self.base.rand.next_int_bound(fire_bound) + 1;
        for _attempt in 0..max_fire_attempts {
            coord.x = (f64::from(block_x + self.base.rand.next_int_bound(CHUNK_WIDTH)) + f64::from(CHUNK_WIDTH) * 0.5) as i32;
            coord.y = self.base.rand.next_int_bound(CHUNK_HEIGHT - 8) + 4; // 120
            coord.z = (f64::from(block_z + self.base.rand.next_int_bound(CHUNK_WIDTH)) + f64::from(CHUNK_WIDTH) * 0.5) as i32;
            FeatureGenerator::with_type(BLOCK_AIR).generate_nether_fire(world, &mut self.base.rand, coord);
        }

        // Generate Glowstone Blob
        let glowstone_bound = self.base.rand.next_int_bound(10) + 1;
        let max_glowstone_attempts = self.base.rand.next_int_bound(glowstone_bound);
        for _attempt in 0..max_glowstone_attempts {
            coord.x = (f64::from(block_x + self.base.rand.next_int_bound(CHUNK_WIDTH)) + f64::from(CHUNK_WIDTH) * 0.5) as i32;
            coord.y = self.base.rand.next_int_bound(CHUNK_HEIGHT - 8) + 4; // 120
            coord.z = (f64::from(block_z + self.base.rand.next_int_bound(CHUNK_WIDTH)) + f64::from(CHUNK_WIDTH) * 0.5) as i32;
            FeatureGenerator::with_type(BLOCK_AIR).generate_nether_glowstone(world, &mut self.base.rand, coord);
        }

        // Generate secondary Glowstone Blob
        for _attempt in 0..10 {
            coord.x = (f64::from(block_x + self.base.rand.next_int_bound(CHUNK_WIDTH)) + f64::from(CHUNK_WIDTH) * 0.5) as i32;
            coord.y = self.base.rand.next_int_bound(CHUNK_HEIGHT);
            coord.z = (f64::from(block_z + self.base.rand.next_int_bound(CHUNK_WIDTH)) + f64::from(CHUNK_WIDTH) * 0.5) as i32;
            FeatureGenerator::with_type(BLOCK_AIR).generate_nether_glowstone(world, &mut self.base.rand, coord);
        }

        // Generate Brown Mushrooms
        if self.base.rand.next_int_bound(1) == 0 {
            coord.x = (f64::from(block_x + self.base.rand.next_int_bound(CHUNK_WIDTH)) + f64::from(CHUNK_WIDTH) * 0.5) as i32;
            coord.y = self.base.rand.next_int_bound(CHUNK_HEIGHT);
            coord.z = (f64::from(block_z + self.base.rand.next_int_bound(CHUNK_WIDTH)) + f64::from(CHUNK_WIDTH) * 0.5) as i32;
            FeatureGenerator::with_type(BLOCK_MUSHROOM_BROWN).generate_flowers(world, &mut self.base.rand, coord);
        }

        // Generate Red Mushrooms
        if self.base.rand.next_int_bound(1) == 0 {
            coord.x = (f64::from(block_x + self.base.rand.next_int_bound(CHUNK_WIDTH)) + f64::from(CHUNK_WIDTH) * 0.5) as i32;
            coord.y = self.base.rand.next_int_bound(CHUNK_HEIGHT);
            coord.z = (f64::from(block_z + self.base.rand.next_int_bound(CHUNK_WIDTH)) + f64::from(CHUNK_WIDTH) * 0.5) as i32;
            FeatureGenerator::with_type(BLOCK_MUSHROOM_RED).generate_flowers(world, &mut self.base.rand, coord);
        }

        true
    }
}

impl NetherGenerator {
    /// @brief Replace some of the stone with Biome-appropriate blocks
    ///
    /// @param chunkPos The x,z coordinate of the chunk
    /// @param c The chunk that should gets its blocks replaced
    fn replace_blocks_for_biome(&mut self, chunk: &mut Chunk) {
        let one_thirty_second = 1.0 / 32.0;
        // Init noise maps
        self.sand_noise.resize(256, 0.0);
        self.gravel_noise.resize(256, 0.0);
        self.stone_noise.resize(256, 0.0);

        // Populate noise maps
        self.sand_gravel_noise_gen.generate_octaves(
            &mut self.sand_noise,
            Vec3::new(f64::from(chunk.cpos.x * CHUNK_WIDTH), f64::from(*chunk.cpos.z() * CHUNK_WIDTH), 0.0),
            Int3::new(16, 16, 1),
            Vec3::new(one_thirty_second, one_thirty_second, 1.0),
        );
        self.sand_gravel_noise_gen.generate_octaves(
            &mut self.gravel_noise,
            Vec3::new(f64::from(chunk.cpos.x * CHUNK_WIDTH), 109.0134, f64::from(*chunk.cpos.z() * CHUNK_WIDTH)),
            Int3::new(16, 1, 16),
            Vec3::new(one_thirty_second, 1.0, one_thirty_second),
        );
        self.stone_noise_gen.generate_octaves(
            &mut self.stone_noise,
            Vec3::new(f64::from(chunk.cpos.x * CHUNK_WIDTH), f64::from(*chunk.cpos.z() * CHUNK_WIDTH), 0.0),
            Int3::new(16, 16, 1),
            Vec3::new(one_thirty_second * 2.0, one_thirty_second * 2.0, one_thirty_second * 2.0),
        );

        // Iterate through entire chunk
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                // This is intentional, to match b1.7.3 behavior!
                let bindex = (x + z * CHUNK_WIDTH) as usize;
                // Get values from noise maps
                let sand_active = self.sand_noise[bindex] + self.base.rand.next_double() * 0.2 > 0.0;
                let gravel_active = self.gravel_noise[bindex] + self.base.rand.next_double() * 0.2 > 3.0;
                let stone_active = double_to_int32(self.stone_noise[bindex] / 3.0 + 3.0 + self.base.rand.next_double() * 0.25);
                let mut stone_depth: i32 = -1;
                // Get biome-appropriate top and filler blocks
                let mut top_block = BLOCK_NETHERRACK;
                let mut filler_block = BLOCK_NETHERRACK;

                // Iterate over column top to bottom
                for y in (0..CHUNK_HEIGHT).rev() {
                    // This is intentional, to match b1.7.3 behavior!
                    let bpos = Int3::new(z, y, x);
                    // Place Bedrock at bottom and top with some randomness
                    if y >= (CHUNK_HEIGHT - 1) - self.base.rand.next_int_bound(5) {
                        chunk.set_block(bpos, BLOCK_BEDROCK);
                        continue;
                    } else if y <= 0 + self.base.rand.next_int_bound(5) {
                        chunk.set_block(bpos, BLOCK_BEDROCK);
                        continue;
                    }

                    let current_block = chunk.get_block(bpos);
                    // Ignore air
                    if current_block == BLOCK_AIR {
                        stone_depth = -1;
                        continue;
                    }

                    // If we counter stone, start replacing it
                    if current_block == BLOCK_NETHERRACK {
                        if stone_depth == -1 {
                            if stone_active <= 0 {
                                top_block = BLOCK_AIR;
                                filler_block = BLOCK_NETHERRACK;
                            } else if y >= NETHER_BIOME_LAVA_LEVEL - 4 && y <= NETHER_BIOME_LAVA_LEVEL + 1 {
                                // If we're close to the water level, apply gravel and sand
                                top_block = BLOCK_NETHERRACK;
                                filler_block = BLOCK_NETHERRACK;

                                if gravel_active {
                                    top_block = BLOCK_GRAVEL;
                                    filler_block = BLOCK_NETHERRACK;
                                }
                                if sand_active {
                                    top_block = BLOCK_SOULSAND;
                                    filler_block = BLOCK_SOULSAND;
                                }
                            }

                            // Add water if we're below lava level
                            if y < NETHER_BIOME_LAVA_LEVEL && top_block == BLOCK_AIR {
                                top_block = BLOCK_LAVA_STILL;
                            }

                            stone_depth = stone_active;
                            // Place filler block if we're under lava
                            chunk.set_block(bpos, if y >= NETHER_BIOME_LAVA_LEVEL - 1 { top_block } else { filler_block });
                        } else if stone_depth > 0 {
                            stone_depth -= 1;
                            chunk.set_block(bpos, filler_block);
                        }
                    }
                }
            }
        }
    }

    /// @brief Generate the Terrain, minus any caves, as just stone
    ///
    /// @param chunkPos The x,z coordinate of the chunk
    /// @param c The chunk that should get its terrain generated
    fn generate_terrain(&mut self, chunk: &mut Chunk) {
        let max = Int3::new(CHUNK_WIDTH / 4 + 1, CHUNK_HEIGHT / 8 + 1, CHUNK_WIDTH / 4 + 1);

        // Generate 4x16x4 low resolution noise map
        let mut terrain_noise_field = std::mem::take(&mut self.terrain_noise_field);
        self.generate_terrain_noise(&mut terrain_noise_field, Int3::new(chunk.cpos.x * 4, 0, *chunk.cpos.z() * 4), max);

        // Terrain noise is interpolated and only sampled every 4 blocks
        for sample_x in 0..4 {
            for sample_z in 0..4 {
                for sample_y in 0..16 {
                    let vertical_lerp_step = 0.125;

                    // Get noise cube corners
                    let mut corner000 =
                        terrain_noise_field[(((sample_x + 0) * max.z + sample_z + 0) * max.y + sample_y + 0) as usize];
                    let mut corner010 =
                        terrain_noise_field[(((sample_x + 0) * max.z + sample_z + 1) * max.y + sample_y + 0) as usize];
                    let mut corner100 =
                        terrain_noise_field[(((sample_x + 1) * max.z + sample_z + 0) * max.y + sample_y + 0) as usize];
                    let mut corner110 =
                        terrain_noise_field[(((sample_x + 1) * max.z + sample_z + 1) * max.y + sample_y + 0) as usize];
                    let corner001 = (terrain_noise_field
                        [(((sample_x + 0) * max.z + sample_z + 0) * max.y + sample_y + 1) as usize]
                        - corner000)
                        * vertical_lerp_step;
                    let corner011 = (terrain_noise_field
                        [(((sample_x + 0) * max.z + sample_z + 1) * max.y + sample_y + 1) as usize]
                        - corner010)
                        * vertical_lerp_step;
                    let corner101 = (terrain_noise_field
                        [(((sample_x + 1) * max.z + sample_z + 0) * max.y + sample_y + 1) as usize]
                        - corner100)
                        * vertical_lerp_step;
                    let corner111 = (terrain_noise_field
                        [(((sample_x + 1) * max.z + sample_z + 1) * max.y + sample_y + 1) as usize]
                        - corner110)
                        * vertical_lerp_step;

                    // Interpolate the 1/4th scale noise
                    for sub_y in 0..8 {
                        let horizontal_lerp_step = 0.25;
                        let mut terrain_x0 = corner000;
                        let mut terrain_x1 = corner010;
                        let terrain_step_x0 = (corner100 - corner000) * horizontal_lerp_step;
                        let terrain_step_x1 = (corner110 - corner010) * horizontal_lerp_step;

                        for sub_x in 0..4 {
                            let mut bpos = Int3::new(sub_x + sample_x * 4, (sample_y * 8) + sub_y, sample_z * 4);
                            let mut terrain_density = terrain_x0;
                            let density_step_z = (terrain_x1 - terrain_x0) * horizontal_lerp_step;

                            for _sub_z in 0..4 {
                                // Here the actual block is determined
                                // Default to air block
                                let mut block_type = BLOCK_AIR;

                                // Place lava in empty space below Nether lava level
                                let y_level = sample_y * 8 + sub_y;
                                if y_level < NETHER_LAVA_LEVEL {
                                    block_type = BLOCK_LAVA_STILL;
                                }

                                // If the terrain density falls below,
                                // replace block with stone
                                if terrain_density > 0.0 {
                                    block_type = BLOCK_NETHERRACK;
                                }

                                chunk.set_block(bpos, block_type);
                                // Prep for next iteration
                                bpos.z += 1;
                                terrain_density += density_step_z;
                            }

                            terrain_x0 += terrain_step_x0;
                            terrain_x1 += terrain_step_x1;
                        }

                        corner000 += corner001;
                        corner010 += corner011;
                        corner100 += corner101;
                        corner110 += corner111;
                    }
                }
            }
        }

        self.terrain_noise_field = terrain_noise_field;
    }

    /// @brief Make terrain noise and updates the terrain map
    ///
    /// @param terrainMap The terrain map that the scaled-down terrain values will be written to
    /// @param chunkPos The x,y,z coordinate of the sub-chunk
    /// @param max Defines the area of the terrainMap
    fn generate_terrain_noise(&mut self, terrain_map: &mut Vec<f64>, cpos: Int3, max: Int3) {
        terrain_map.resize((max.x * max.y * max.z) as usize, 0.0);

        let hori_scale = 684.412;
        let vert_scale = 2053.236;

        {
            let vec_cpos = Vec3::new(f64::from(cpos.x), f64::from(cpos.y), f64::from(cpos.z));
            // We do this to need to generate noise as often
            self.continentalness_noise_gen.generate_octaves(
                &mut self.continentalness_noise_field,
                vec_cpos,
                Int3::new(max.x, 1, max.z),
                Vec3::new(1.0, 0.0, 1.0),
            );
            self.depth_noise_gen.generate_octaves(
                &mut self.depth_noise_field,
                vec_cpos,
                Int3::new(max.x, 1, max.z),
                Vec3::new(100.0, 0.0, 100.0),
            );
            self.selector_noise_gen.generate_octaves(
                &mut self.selector_noise_field,
                vec_cpos,
                max,
                Vec3::new(hori_scale / 80.0, vert_scale / 60.0, hori_scale / 80.0),
            );
            self.low_noise_gen.generate_octaves(&mut self.low_noise_field, vec_cpos, max, Vec3::new(hori_scale, vert_scale, hori_scale));
            self.high_noise_gen.generate_octaves(&mut self.high_noise_field, vec_cpos, max, Vec3::new(hori_scale, vert_scale, hori_scale));
        }
        // Used to iterate 3D noise maps (low, high, selector)
        let mut xyz_index: usize = 0;
        // Used to iterate 2D Noise maps (depth, continentalness)
        let mut xz_index: usize = 0;
        // Reserve stuff
        let mut nether_density_offset = vec![0.0f64; max.y as usize];

        for i_y in 0..max.y {
            nether_density_offset[i_y as usize] = (f64::from(i_y) * JavaMath::PI * 6.0 / f64::from(max.y)).cos() * 2.0;
            let mut di_y = f64::from(i_y);
            if di_y > f64::from(max.y / 2) {
                di_y = f64::from(max.y - 1 - i_y);
            }
            if di_y < 4.0 {
                di_y = 4.0 - di_y;
                nether_density_offset[i_y as usize] -= di_y * di_y * di_y * 10.0;
            }
        }

        for _i_x in 0..max.x {
            for _i_z in 0..max.z {
                // Sample contientalness noise
                let mut continentalness = (self.continentalness_noise_field[xz_index] + 256.0) / 512.0;
                if continentalness > 1.0 {
                    continentalness = 1.0;
                }
                // Sample depth noise
                let mut depth_noise = self.depth_noise_field[xz_index] / 8000.0;
                if depth_noise < 0.0 {
                    depth_noise = -depth_noise;
                }
                depth_noise = depth_noise * 3.0 - 3.0;
                if depth_noise < 0.0 {
                    depth_noise /= 2.0;
                    if depth_noise < -1.0 {
                        depth_noise = -1.0;
                    }
                    depth_noise /= 1.4;
                    depth_noise /= 2.0;
                    continentalness = 0.0;
                } else {
                    if depth_noise > 1.0 {
                        depth_noise = 1.0;
                    }
                    depth_noise /= 6.0;
                }
                continentalness += 0.5;
                depth_noise = depth_noise * f64::from(max.y) / 16.0;
                xz_index += 1;

                for i_y in 0..max.y {
                    // Sample 3D noises
                    let mut terrain_density = 0.0;
                    let density_offset = nether_density_offset[i_y as usize];
                    // Sample low noise
                    let low_noise = self.low_noise_field[xyz_index] / 512.0;
                    // Sample high noise
                    let high_noise = self.high_noise_field[xyz_index] / 512.0;
                    // Sample selector noise
                    let selector_noise = (self.selector_noise_field[xyz_index] / 10.0 + 1.0) / 2.0;
                    if selector_noise < 0.0 {
                        terrain_density = low_noise;
                    } else if selector_noise > 1.0 {
                        terrain_density = high_noise;
                    } else {
                        terrain_density = low_noise + (high_noise - low_noise) * selector_noise;
                    }

                    terrain_density -= density_offset;
                    // Reduce density towards max height
                    if i_y > max.y - 4 {
                        let height_edge_fade = f64::from((i_y - (max.y - 4)) as f32 / 3.0f32);
                        terrain_density = (terrain_density * (1.0 - height_edge_fade)) + (-10.0 * height_edge_fade);
                    }
                    if f64::from(i_y) < 0.0 {
                        let mut height_edge_fade = 0.0 - f64::from(i_y) / 4.0;
                        if height_edge_fade < 0.0 {
                            height_edge_fade = 0.0;
                        }
                        if height_edge_fade > 1.0 {
                            height_edge_fade = 1.0;
                        }
                        terrain_density = (terrain_density * (1.0 - height_edge_fade)) + (-10.0 * height_edge_fade);
                    }

                    terrain_map[xyz_index] = terrain_density;
                    xyz_index += 1;
                }
            }
        }
    }
}
