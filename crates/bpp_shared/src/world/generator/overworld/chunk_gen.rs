/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Mojang Studios (2011)
*/

use crate::constants::{CHUNK_AREA, CHUNK_HEIGHT, CHUNK_WIDTH, WATER_LEVEL};
use crate::enums::biomes::{Biome, get_filler_block, get_top_block};
use crate::enums::blocks::{
    BLOCK_AIR, BLOCK_BEDROCK, BLOCK_DANDELION, BLOCK_DEADBUSH, BLOCK_DIRT, BLOCK_GRAVEL, BLOCK_ICE, BLOCK_LAVA_FLOWING, BLOCK_LAVA_STILL,
    BLOCK_MUSHROOM_BROWN, BLOCK_MUSHROOM_RED, BLOCK_ORE_COAL, BLOCK_ORE_DIAMOND, BLOCK_ORE_GOLD, BLOCK_ORE_IRON, BLOCK_ORE_LAPIS_LAZULI,
    BLOCK_ORE_REDSTONE_OFF, BLOCK_ROSE, BLOCK_SAND, BLOCK_SANDSTONE, BLOCK_SNOW_LAYER, BLOCK_STONE, BLOCK_TALLGRASS, BLOCK_WATER_FLOWING,
    BLOCK_WATER_STILL,
};
use crate::helpers::java::java_math::double_to_int32;
use crate::helpers::java::java_random::Random;
use crate::numeric_structs::{Int2, Int3, Int32_2, Int32_3, Vec3};
use crate::world::chunk::Chunk;
use crate::world::generator::generator::{Generator, GeneratorBehavior};
use crate::world::generator::overworld::biome_gen::BiomeGenerator;
use crate::world::generator::overworld::tree_gen::{AltTaigaTreeGenerator, BigTreeGenerator, TaigaTreeGenerator, TreeGenerator, TreeGeneratorBehavior};
use crate::world::generator::shared::cave_gen::CaveGenerator;
use crate::world::generator::shared::feature_gen::{FeatureGenerator, WorldWrapper, is_solid};
use crate::world::noise::noise_octaves_perlin::NoiseOctavesPerlin;

/// @brief A faithful reimplementation of the Beta 1.7.3 Overworld Generator
pub struct OverworldGenerator {
    pub base: Generator,

    // Perlin Noise Generators
    low_noise_gen: NoiseOctavesPerlin,
    high_noise_gen: NoiseOctavesPerlin,
    selector_noise_gen: NoiseOctavesPerlin,
    sand_gravel_noise_gen: NoiseOctavesPerlin,
    stone_noise_gen: NoiseOctavesPerlin,
    continentalness_noise_gen: NoiseOctavesPerlin,
    depth_noise_gen: NoiseOctavesPerlin,
    tree_density_noise_gen: NoiseOctavesPerlin,

    // Stored noise Fields
    terrain_noise_field: Vec<f64>,
    low_noise_field: Vec<f64>,
    high_noise_field: Vec<f64>,
    selector_noise_field: Vec<f64>,
    continentalness_noise_field: Vec<f64>,
    depth_noise_field: Vec<f64>,

    sand_noise: Vec<f64>,
    gravel_noise: Vec<f64>,
    stone_noise: Vec<f64>,

    // Biome Vectors
    biome_map: [Biome; CHUNK_AREA as usize],
    temperature: Vec<f64>,
    humidity: Vec<f64>,
    weirdness: Vec<f64>,

    // Cave Gen
    caver: CaveGenerator,
}

impl OverworldGenerator {
    /// @brief Construct a new Beta 1.7.3 Overworld Generator
    ///
    /// @param pSeed The seed of the generated world
    /// @param pWorld The world that the OverworldGenerator belongs to
    pub fn new(p_seed: i64) -> Self {
        let mut base = Generator::new(p_seed);
        base.rand = Random::with_seed(base.seed);
        // Init Terrain Noise
        let low_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 16);
        let high_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 16);
        let selector_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 8);
        let sand_gravel_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 4);
        let stone_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 4);
        let continentalness_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 10);
        let depth_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 16);
        let tree_density_noise_gen = NoiseOctavesPerlin::with_random(&mut base.rand, 8);

        Self {
            base,
            low_noise_gen,
            high_noise_gen,
            selector_noise_gen,
            sand_gravel_noise_gen,
            stone_noise_gen,
            continentalness_noise_gen,
            depth_noise_gen,
            tree_density_noise_gen,
            terrain_noise_field: Vec::new(),
            low_noise_field: Vec::new(),
            high_noise_field: Vec::new(),
            selector_noise_field: Vec::new(),
            continentalness_noise_field: Vec::new(),
            depth_noise_field: Vec::new(),
            sand_noise: Vec::new(),
            gravel_noise: Vec::new(),
            stone_noise: Vec::new(),
            biome_map: [Biome::None; CHUNK_AREA as usize],
            temperature: Vec::new(),
            humidity: Vec::new(),
            weirdness: Vec::new(),
            caver: CaveGenerator::new(false),
        }
    }

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
        let cx = chunk.cpos.x;
        let cz = *chunk.cpos.z();
        self.sand_gravel_noise_gen.generate_octaves(
            &mut self.sand_noise,
            Vec3::new((cx * CHUNK_WIDTH) as f64, (cz * CHUNK_WIDTH) as f64, 0.0),
            Int32_3::new(16, 16, 1),
            Vec3::new(one_thirty_second, one_thirty_second, 1.0),
        );
        self.sand_gravel_noise_gen.generate_octaves(
            &mut self.gravel_noise,
            Vec3::new((cx * CHUNK_WIDTH) as f64, 109.0134, (cz * CHUNK_WIDTH) as f64),
            Int32_3::new(16, 1, 16),
            Vec3::new(one_thirty_second, 1.0, one_thirty_second),
        );
        self.stone_noise_gen.generate_octaves(
            &mut self.stone_noise,
            Vec3::new((cx * CHUNK_WIDTH) as f64, (cz * CHUNK_WIDTH) as f64, 0.0),
            Int32_3::new(16, 16, 1),
            Vec3::new(one_thirty_second * 2.0, one_thirty_second * 2.0, one_thirty_second * 2.0),
        );

        // Iterate through entire chunk
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                // This is intentional, to match b1.7.3 behavior!
                let bindex = (x + z * CHUNK_WIDTH) as usize;
                // Get values from noise maps
                let biome = self.biome_map[bindex];
                let sand_active = self.sand_noise[bindex] + self.base.rand.next_double() * 0.2 > 0.0;
                let gravel_active = self.gravel_noise[bindex] + self.base.rand.next_double() * 0.2 > 3.0;
                let stone_active = double_to_int32(self.stone_noise[bindex] / 3.0 + 3.0 + self.base.rand.next_double() * 0.25);
                let mut stone_depth: i32 = -1;
                // Get biome-appropriate top and filler blocks
                let mut top_block = get_top_block(biome);
                let mut filler_block = get_filler_block(biome);

                // Iterate over column top to bottom
                for y in (0..CHUNK_HEIGHT).rev() {
                    // This is intentional, to match b1.7.3 behavior!
                    let bpos = Int3::new(z, y, x);
                    // Place Bedrock at bottom with some randomness
                    if y <= 0 + self.base.rand.next_int_bound(5) {
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
                    if current_block == BLOCK_STONE {
                        if stone_depth == -1 {
                            if stone_active <= 0 {
                                top_block = BLOCK_AIR;
                                filler_block = BLOCK_STONE;
                            } else if y >= WATER_LEVEL - 4 && y <= WATER_LEVEL + 1 {
                                // If we're close to the water level, apply gravel and sand
                                top_block = get_top_block(biome);
                                filler_block = get_filler_block(biome);

                                if gravel_active {
                                    top_block = BLOCK_AIR;
                                }
                                if gravel_active {
                                    filler_block = BLOCK_GRAVEL;
                                }
                                if sand_active {
                                    top_block = BLOCK_SAND;
                                }
                                if sand_active {
                                    filler_block = BLOCK_SAND;
                                }
                            }

                            // Add water if we're below water level
                            if y < WATER_LEVEL && top_block == BLOCK_AIR {
                                top_block = BLOCK_WATER_STILL;
                            }

                            stone_depth = stone_active;
                            // Place filler block if we're underwater
                            chunk.set_block(bpos, if y >= WATER_LEVEL - 1 { top_block } else { filler_block });
                        } else if stone_depth > 0 {
                            stone_depth -= 1;
                            chunk.set_block(bpos, filler_block);
                            if stone_depth == 0 && filler_block == BLOCK_SAND {
                                stone_depth = self.base.rand.next_int_bound(4);
                                filler_block = BLOCK_SANDSTONE;
                            }
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
        let cx = chunk.cpos.x;
        let cz = *chunk.cpos.z();
        self.generate_terrain_noise(Int3::new(cx * 4, 0, cz * 4), max);

        // Terrain noise is interpolated and only sampled every 4 blocks
        for sample_x in 0..4 {
            for sample_z in 0..4 {
                for sample_y in 0..16 {
                    let vertical_lerp_step = 0.125;

                    // Get noise cube corners
                    let mut corner000 = self.terrain_noise_field[(((sample_x + 0) * max.z + sample_z + 0) * max.y + sample_y + 0) as usize];
                    let mut corner010 = self.terrain_noise_field[(((sample_x + 0) * max.z + sample_z + 1) * max.y + sample_y + 0) as usize];
                    let mut corner100 = self.terrain_noise_field[(((sample_x + 1) * max.z + sample_z + 0) * max.y + sample_y + 0) as usize];
                    let mut corner110 = self.terrain_noise_field[(((sample_x + 1) * max.z + sample_z + 1) * max.y + sample_y + 0) as usize];
                    let corner001 = (self.terrain_noise_field[(((sample_x + 0) * max.z + sample_z + 0) * max.y + sample_y + 1) as usize]
                        - corner000)
                        * vertical_lerp_step;
                    let corner011 = (self.terrain_noise_field[(((sample_x + 0) * max.z + sample_z + 1) * max.y + sample_y + 1) as usize]
                        - corner010)
                        * vertical_lerp_step;
                    let corner101 = (self.terrain_noise_field[(((sample_x + 1) * max.z + sample_z + 0) * max.y + sample_y + 1) as usize]
                        - corner100)
                        * vertical_lerp_step;
                    let corner111 = (self.terrain_noise_field[(((sample_x + 1) * max.z + sample_z + 1) * max.y + sample_y + 1) as usize]
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

                            for sub_z in 0..4 {
                                // Here the actual block is determined
                                // Default to air block
                                let mut block_type = BLOCK_AIR;

                                // If water is too cold, turn into ice
                                let temp = self.temperature[((sample_x * 4 + sub_x) * 16 + sample_z * 4 + sub_z) as usize];
                                let y_level = sample_y * 8 + sub_y;
                                if y_level < WATER_LEVEL {
                                    if temp < 0.5 && y_level >= WATER_LEVEL - 1 {
                                        block_type = BLOCK_ICE;
                                    } else {
                                        block_type = BLOCK_WATER_STILL;
                                    }
                                }

                                // If the terrain density falls below,
                                // replace block with stone
                                if terrain_density > 0.0 {
                                    block_type = BLOCK_STONE;
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
    }

    /// @brief Make terrain noise and updates the terrain map
    ///
    /// @param terrainMap The terrain map that the scaled-down terrain values will be written to
    /// @param chunkPos The x,y,z coordinate of the sub-chunk
    /// @param max Defines the area of the terrainMap
    fn generate_terrain_noise(&mut self, cpos: Int3, max: Int3) {
        self.terrain_noise_field.resize((max.x * max.y * max.z) as usize, 0.0);

        let hori_scale = 684.412;
        let vert_scale = 684.412;

        // We do this to need to generate noise as often
        self.continentalness_noise_gen.generate_octaves_2d(
            &mut self.continentalness_noise_field,
            Int2::new(cpos.x, cpos.z),
            Int2::new(max.x, max.z),
            crate::numeric_structs::Vec2::new(1.121, 1.121),
            0.5,
        );
        self.depth_noise_gen.generate_octaves_2d(
            &mut self.depth_noise_field,
            Int2::new(cpos.x, cpos.z),
            Int2::new(max.x, max.z),
            crate::numeric_structs::Vec2::new(200.0, 200.0),
            0.5,
        );
        self.selector_noise_gen.generate_octaves(
            &mut self.selector_noise_field,
            Vec3::new(cpos.x as f64, cpos.y as f64, cpos.z as f64),
            max,
            Vec3::new(hori_scale / 80.0, vert_scale / 160.0, hori_scale / 80.0),
        );
        self.low_noise_gen.generate_octaves(
            &mut self.low_noise_field,
            Vec3::new(cpos.x as f64, cpos.y as f64, cpos.z as f64),
            max,
            Vec3::new(hori_scale, vert_scale, hori_scale),
        );
        self.high_noise_gen.generate_octaves(
            &mut self.high_noise_field,
            Vec3::new(cpos.x as f64, cpos.y as f64, cpos.z as f64),
            max,
            Vec3::new(hori_scale, vert_scale, hori_scale),
        );
        // Used to iterate 3D noise maps (low, high, selector)
        let mut xyz_index = 0usize;
        // Used to iterate 2D Noise maps (depth, continentalness)
        let mut xz_index = 0usize;
        let scale_fraction = 16 / max.x;

        for i_x in 0..max.x {
            let sample_x = i_x * scale_fraction + scale_fraction / 2;

            for i_z in 0..max.z {
                // Sample 2D noises
                let sample_z = i_z * scale_fraction + scale_fraction / 2;
                // Apply biome-noise-dependent variety
                let sample_index = (sample_x * CHUNK_WIDTH + sample_z) as usize;
                let temp = self.temperature[sample_index];
                let mut humi = self.humidity[sample_index] * temp;
                humi = 1.0 - humi;
                humi *= humi;
                humi *= humi;
                humi = 1.0 - humi;
                // Sample contientalness noise
                let mut continentalness = (self.continentalness_noise_field[xz_index] + 256.0) / 512.0;
                continentalness *= humi;
                if continentalness > 1.0 {
                    continentalness = 1.0;
                }
                // Sample depth noise
                let mut depth_noise = self.depth_noise_field[xz_index] / 8000.0;
                if depth_noise < 0.0 {
                    depth_noise = -depth_noise * 0.3;
                }
                depth_noise = depth_noise * 3.0 - 2.0;
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
                    depth_noise /= 8.0;
                }
                if continentalness < 0.0 {
                    continentalness = 0.0;
                }
                continentalness += 0.5;
                depth_noise = depth_noise * max.y as f64 / 16.0;
                let elevation_offset = max.y as f64 / 2.0 + depth_noise * 4.0;
                xz_index += 1;

                for i_y in 0..max.y {
                    // Sample 3D noises
                    let mut terrain_density;
                    let mut density_offset = (i_y as f64 - elevation_offset) * 12.0 / continentalness;
                    if density_offset < 0.0 {
                        density_offset *= 4.0;
                    }
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
                        let height_edge_fade = (((i_y - (max.y - 4)) as f32) / 3.0f32) as f64;
                        terrain_density = (terrain_density * (1.0 - height_edge_fade)) + (-10.0 * height_edge_fade);
                    }

                    self.terrain_noise_field[xyz_index] = terrain_density;
                    xyz_index += 1;
                }
            }
        }
    }

    /// @brief Probes the biome map at the specified coordinates
    ///
    /// @param worldPos The x,z coordinate of the desired block column
    /// @return The Biome at that column
    fn get_biome_at(&self, world_pos: Int2) -> Biome {
        // biomeMap is always for the chunk whose origin is (cpos.x*16, cpos.z*16).
        // Convert world coords to chunk-local [0,15] and index directly.
        let local_x = ((world_pos.x % CHUNK_WIDTH) + CHUNK_WIDTH) % CHUNK_WIDTH;
        let local_z = ((*world_pos.z() % CHUNK_WIDTH) + CHUNK_WIDTH) % CHUNK_WIDTH;
        self.biome_map[(local_x * CHUNK_WIDTH + local_z) as usize]
    }

    // Exact port of BiomeGenBase.getRandomWorldGenForTrees() and per-biome overrides.
    fn generate_tree_for_biome(world: &mut WorldWrapper, p_rand: &mut Random, pos: Int3, biome: Biome) {
        match biome {
            Biome::Taiga => {
                if p_rand.next_int_bound(3) == 0 {
                    TaigaTreeGenerator::new().generate(world, p_rand, pos, false);
                } else {
                    AltTaigaTreeGenerator::new().generate(world, p_rand, pos, false);
                }
            }
            Biome::Forest => {
                if p_rand.next_int_bound(5) == 0 {
                    TreeGenerator::new().generate(world, p_rand, pos, true);
                } else if p_rand.next_int_bound(3) == 0 {
                    let mut big = BigTreeGenerator::new();
                    big.configure(1.0, 1.0, 1.0);
                    big.generate(world, p_rand, pos, false);
                } else {
                    TreeGenerator::new().generate(world, p_rand, pos, false);
                }
            }
            Biome::Rainforest => {
                if p_rand.next_int_bound(3) == 0 {
                    let mut big = BigTreeGenerator::new();
                    big.configure(1.0, 1.0, 1.0);
                    big.generate(world, p_rand, pos, false);
                } else {
                    TreeGenerator::new().generate(world, p_rand, pos, false);
                }
            }
            _ => {
                if p_rand.next_int_bound(10) == 0 {
                    let mut big = BigTreeGenerator::new();
                    big.configure(1.0, 1.0, 1.0);
                    big.generate(world, p_rand, pos, false);
                } else {
                    TreeGenerator::new().generate(world, p_rand, pos, false);
                }
            }
        }
    }
}

impl GeneratorBehavior for OverworldGenerator {
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
        self.base.rand.set_seed((chunk.cpos.x as i64).wrapping_mul(341873128712) + (*chunk.cpos.z() as i64).wrapping_mul(132897987541));

        // Allocate empty chunk
        chunk.clear();

        // Generate Biomes
        BiomeGenerator::new(self.base.seed).generate_biome_map(
            &mut self.biome_map,
            &mut self.temperature,
            &mut self.humidity,
            &mut self.weirdness,
            Int2::new(chunk.cpos.x * CHUNK_WIDTH, *chunk.cpos.z() * CHUNK_WIDTH),
        );

        // Store the final temperature and humidity in the chunk so PopulateChunk
        // (which runs on a different thread_local OverworldGenerator) can reconstruct the
        // biome map via GetBiomeFromLookup without re-running the noise generators.
        for i in 0..CHUNK_AREA as usize {
            chunk.temperature[i] = self.temperature[i] as f32;
            chunk.humidity[i] = self.humidity[i] as f32;
        }

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
    /// Direct port of ChunkProviderGenerate.populate() from Beta 1.7.3.
    /// Biome is sampled at blockX+16, blockZ+16 from stored chunk climate data.
    /// RNG seeding, section order, rand call counts, and coordinate offsets all
    /// match the Java source exactly.
    fn populate_chunk(&mut self, cpos: Int32_2, world: &mut WorldWrapper) -> bool {
        let block_x = cpos.x * CHUNK_WIDTH;
        let block_z = *cpos.z() * CHUNK_WIDTH;
        let biome = BiomeGenerator::new(self.base.seed).get_biome_at_point(Int2::new(block_x + CHUNK_WIDTH, block_z + CHUNK_WIDTH));
        // Java RNG seeding sequence
        self.base.rand.set_seed(world.get_seed());
        let x_salt = self.base.rand.next_long() / 2 * 2 + 1;
        let z_salt = self.base.rand.next_long() / 2 * 2 + 1;
        // Use unsigned arithmetic to avoid overflow UB
        let x_salt_u = x_salt as u64;
        let z_salt_u = z_salt as u64;
        let x_part = (cpos.x as i64 as u64).wrapping_mul(x_salt_u);
        let z_part = (*cpos.z() as i64 as u64).wrapping_mul(z_salt_u);
        let combined = x_part.wrapping_add(z_part) ^ (world.get_seed() as u64);

        self.base.rand.set_seed(combined as i64);

        let mut coord;

        // Water lakes
        if self.base.rand.next_int_bound(4) == 0 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                self.base.rand.next_int_bound(CHUNK_HEIGHT),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
            );
            FeatureGenerator::new(BLOCK_WATER_STILL).generate_lake(world, &mut self.base.rand, coord);
        }

        // Lava lakes
        if self.base.rand.next_int_bound(8) == 0 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                {
                    let bound = self.base.rand.next_int_bound(120) + 8;
                    self.base.rand.next_int_bound(bound)
                },
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
            );
            if coord.y < WATER_LEVEL || self.base.rand.next_int_bound(10) == 0 {
                FeatureGenerator::new(BLOCK_LAVA_STILL).generate_lake(world, &mut self.base.rand, coord);
            }
        }

        // Dungeons
        for _i in 0..8 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                self.base.rand.next_int_bound(CHUNK_HEIGHT),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
            );
            FeatureGenerator::default().generate_dungeon(world, &mut self.base.rand, coord);
        }

        // Clay
        for _i in 0..10 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH),
                self.base.rand.next_int_bound(CHUNK_HEIGHT),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH),
            );
            FeatureGenerator::default().generate_clay(world, &mut self.base.rand, coord, 32);
        }

        // Dirt blobs
        for _i in 0..20 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH),
                self.base.rand.next_int_bound(CHUNK_HEIGHT),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH),
            );
            FeatureGenerator::new(BLOCK_DIRT).generate_minable(world, &mut self.base.rand, coord, 32);
        }

        // Gravel blobs
        for _i in 0..10 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH),
                self.base.rand.next_int_bound(CHUNK_HEIGHT),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH),
            );
            FeatureGenerator::new(BLOCK_GRAVEL).generate_minable(world, &mut self.base.rand, coord, 32);
        }

        // Coal Ore blobs
        for _i in 0..20 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH),
                self.base.rand.next_int_bound(CHUNK_HEIGHT),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH),
            );
            FeatureGenerator::new(BLOCK_ORE_COAL).generate_minable(world, &mut self.base.rand, coord, 16);
        }

        // Iron Ore blobs
        for _i in 0..20 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH),
                self.base.rand.next_int_bound(CHUNK_HEIGHT / 2),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH),
            );
            FeatureGenerator::new(BLOCK_ORE_IRON).generate_minable(world, &mut self.base.rand, coord, 8);
        }

        // Gold Ore blobs
        for _i in 0..2 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH),
                self.base.rand.next_int_bound(CHUNK_HEIGHT / 4),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH),
            );
            FeatureGenerator::new(BLOCK_ORE_GOLD).generate_minable(world, &mut self.base.rand, coord, 8);
        }

        // Redstone Ore blobs
        for _i in 0..8 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH),
                self.base.rand.next_int_bound(CHUNK_HEIGHT / 8),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH),
            );
            FeatureGenerator::new(BLOCK_ORE_REDSTONE_OFF).generate_minable(world, &mut self.base.rand, coord, 7);
        }

        // Diamond Ore blobs
        {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH),
                self.base.rand.next_int_bound(CHUNK_HEIGHT / 8),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH),
            );
            FeatureGenerator::new(BLOCK_ORE_DIAMOND).generate_minable(world, &mut self.base.rand, coord, 7);
        }

        // Lapis lazuli Ore blobs
        {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH),
                self.base.rand.next_int_bound(CHUNK_HEIGHT / 8) + self.base.rand.next_int_bound(CHUNK_HEIGHT / 8),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH),
            );
            FeatureGenerator::new(BLOCK_ORE_LAPIS_LAZULI).generate_minable(world, &mut self.base.rand, coord, 6);
        }

        // Tree count
        let noise_val = self.tree_density_noise_gen.generate_octaves_scalar(crate::numeric_structs::Vec2::new(
            block_x as f64 * 0.5,
            block_z as f64 * 0.5,
        ));
        let base_tree_count = double_to_int32((noise_val / 8.0 + self.base.rand.next_double() * 4.0 + 4.0) / 3.0);
        let mut tree_count = 0;
        if self.base.rand.next_int_bound(10) == 0 {
            tree_count += 1;
        }

        // Biome tree adjustments
        match biome {
            Biome::Forest | Biome::Rainforest | Biome::Taiga => tree_count += base_tree_count + 5,
            Biome::SeasonalForest => tree_count += base_tree_count + 2,
            Biome::Desert | Biome::Tundra | Biome::Plains => tree_count -= 20,
            Biome::None | Biome::Swampland | Biome::Savanna | Biome::Shrubland | Biome::IceDesert | Biome::Hell | Biome::Sky => {}
        }

        for _i in 0..tree_count {
            let tx = block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8;
            let tz = block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8;
            let ty = world.get_height_value(tx, tz);
            coord = Int3::new(tx, ty, tz);
            Self::generate_tree_for_biome(world, &mut self.base.rand, coord, biome);
        }

        // Dandelion patches
        {
            let count = match biome {
                Biome::Forest => 2,
                Biome::SeasonalForest => 4,
                Biome::Taiga => 2,
                Biome::Plains => 3,
                _ => 0,
            };
            for _i in 0..count {
                coord = Int3::new(
                    block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                    self.base.rand.next_int_bound(CHUNK_HEIGHT),
                    block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                );
                FeatureGenerator::new(BLOCK_DANDELION).generate_flowers(world, &mut self.base.rand, coord);
            }
        }

        // Tall grass / fern patches
        {
            let count = match biome {
                Biome::Forest => 2,
                Biome::Rainforest => 10,
                Biome::SeasonalForest => 2,
                Biome::Taiga => 1,
                Biome::Plains => 10,
                _ => 0,
            };
            for _i in 0..count {
                let mut grass_meta: i8 = 1;
                if biome == Biome::Rainforest && self.base.rand.next_int_bound(3) != 0 {
                    grass_meta = 2; // fern
                }
                coord = Int3::new(
                    block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                    self.base.rand.next_int_bound(CHUNK_HEIGHT),
                    block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                );
                FeatureGenerator::with_meta(BLOCK_TALLGRASS, grass_meta).generate_tallgrass(world, &mut self.base.rand, coord);
            }
        }

        // Deadbush patches
        {
            let count = if biome == Biome::Desert { 2 } else { 0 };
            for _i in 0..count {
                coord = Int3::new(
                    block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                    self.base.rand.next_int_bound(CHUNK_HEIGHT),
                    block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                );
                FeatureGenerator::new(BLOCK_DEADBUSH).generate_deadbush(world, &mut self.base.rand, coord);
            }
        }

        // Rose patches
        if self.base.rand.next_int_bound(2) == 0 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                self.base.rand.next_int_bound(CHUNK_HEIGHT),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
            );
            FeatureGenerator::new(BLOCK_ROSE).generate_flowers(world, &mut self.base.rand, coord);
        }

        // Brown mushroom patches
        if self.base.rand.next_int_bound(4) == 0 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                self.base.rand.next_int_bound(CHUNK_HEIGHT),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
            );
            FeatureGenerator::new(BLOCK_MUSHROOM_BROWN).generate_flowers(world, &mut self.base.rand, coord);
        }

        // Red mushroom patches
        if self.base.rand.next_int_bound(8) == 0 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                self.base.rand.next_int_bound(CHUNK_HEIGHT),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
            );
            FeatureGenerator::new(BLOCK_MUSHROOM_RED).generate_flowers(world, &mut self.base.rand, coord);
        }

        // Sugar cane
        for _i in 0..10 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                self.base.rand.next_int_bound(CHUNK_HEIGHT),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
            );
            FeatureGenerator::default().generate_sugarcane(world, &mut self.base.rand, coord);
        }

        // Pumpkin patches
        if self.base.rand.next_int_bound(32) == 0 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                self.base.rand.next_int_bound(CHUNK_HEIGHT),
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
            );
            FeatureGenerator::default().generate_pumpkins(world, &mut self.base.rand, coord);
        }

        // Cacti
        {
            let count = if biome == Biome::Desert { 10 } else { 0 };
            for _i in 0..count {
                coord = Int3::new(
                    block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                    self.base.rand.next_int_bound(CHUNK_HEIGHT),
                    block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                );
                FeatureGenerator::default().generate_cacti(world, &mut self.base.rand, coord);
            }
        }

        // Water springs
        for _i in 0..50 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                {
                    let bound = self.base.rand.next_int_bound(120) + 8;
                    self.base.rand.next_int_bound(bound)
                },
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
            );
            FeatureGenerator::new(BLOCK_WATER_FLOWING).generate_liquid(world, &mut self.base.rand, coord);
        }

        // Lava springs
        for _i in 0..20 {
            coord = Int3::new(
                block_x + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
                {
                    let inner = self.base.rand.next_int_bound(112) + 8;
                    let middle = self.base.rand.next_int_bound(inner) + 8;
                    self.base.rand.next_int_bound(middle)
                },
                block_z + self.base.rand.next_int_bound(CHUNK_WIDTH) + 8,
            );
            FeatureGenerator::new(BLOCK_LAVA_FLOWING).generate_liquid(world, &mut self.base.rand, coord);
        }

        // Snow/ice placement for cold biomes
        for x in (block_x + 8)..(block_x + 8 + CHUNK_WIDTH) {
            for z in (block_z + 8)..(block_z + 8 + CHUNK_WIDTH) {
                let top_y = world.find_top_solid_block(x, z);
                let temp = world.get_temperature_at(x, z) - (top_y - 64) as f64 / 64.0 * 0.3;
                if temp < 0.5
                    && top_y > 0
                    && top_y < CHUNK_HEIGHT
                    && world.get_block_id(Int3::new(x, top_y, z)) == BLOCK_AIR
                    && world.get_block_id(Int3::new(x, top_y - 1, z)) != BLOCK_ICE
                    && is_solid(world.get_block_id(Int3::new(x, top_y - 1, z)))
                {
                    world.set_block(Int3::new(x, top_y, z), BLOCK_SNOW_LAYER, 0);
                }
            }
        }
        true
    }
}
