/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Mojang Studios (2011)
*/

use crate::constants::CHUNK_WIDTH;
use crate::enums::biomes::{Biome, get_biome_from_lookup};
use crate::helpers::java::java_random::Random;
use crate::numeric_structs::{Int2, Int32_2, Vec2};
use crate::world::noise::noise_octaves_simplex::NoiseOctavesSimplex;

/// @brief A faithful reimplementation of the Beta 1.7.3 biome generator
pub struct BiomeGenerator {
    // Simplex Noise Generators
    temperature_noise_gen: NoiseOctavesSimplex,
    humidity_noise_gen: NoiseOctavesSimplex,
    weirdness_noise_gen: NoiseOctavesSimplex,
}

impl BiomeGenerator {
    /// @brief Construct a new Beta 1.7.3 Biome
    ///
    /// @param seed The world seed that the biome-generator will use
    pub fn new(seed: i64) -> Self {
        // Init Biome Noise
        let mut rand_temp = Random::with_seed(seed.wrapping_mul(9871));
        let mut rand_hum = Random::with_seed(seed.wrapping_mul(39811));
        let mut rand_weird = Random::with_seed(seed.wrapping_mul(543321));
        Self {
            temperature_noise_gen: NoiseOctavesSimplex::with_random(&mut rand_temp, 4),
            humidity_noise_gen: NoiseOctavesSimplex::with_random(&mut rand_hum, 4),
            weirdness_noise_gen: NoiseOctavesSimplex::with_random(&mut rand_weird, 2),
        }
    }

    /// @brief Generate Biomes based on simplex noise and updates the temperature, humidity and weirdness maps
    ///
    /// @param biomeMap The biome map the final Biome values should be written to
    /// @param temperature The temperature map that'll be used/written to during generation
    /// @param humidity The humidity map that'll be used/written to during generation
    /// @param weirdness The weirdness map that'll be used/written to during generation
    /// @param blockPos The x,z block-space coordindate of the chunk
    /// @param max The size of the area that'll be generated (16x16 by default)
    pub fn generate_biome_map(
        &mut self,
        biome_map: &mut [Biome],
        temperature: &mut Vec<f64>,
        humidity: &mut Vec<f64>,
        weirdness: &mut Vec<f64>,
        block_pos: Int2,
    ) {
        // Get noise values
        let max_area = Int32_2::new(CHUNK_WIDTH, CHUNK_WIDTH);
        self.temperature_noise_gen.generate_octaves(temperature, block_pos, max_area, Vec2::new(0.025f32 as f64, 0.025f32 as f64), 0.25);
        self.humidity_noise_gen.generate_octaves(humidity, block_pos, max_area, Vec2::new(0.05f32 as f64, 0.05f32 as f64), 1.0 / 3.0);
        self.weirdness_noise_gen.generate_octaves(weirdness, block_pos, max_area, Vec2::new(0.25, 0.25), 0.5882352941176471);
        let mut index = 0usize;

        // Iterate over each block column
        for _i_x in 0..CHUNK_WIDTH {
            for _i_z in 0..CHUNK_WIDTH {
                let weird = weirdness[index] * 1.1 + 0.5;
                let mut scale = 0.01;
                let mut limit = 1.0 - scale;
                let mut temp = (temperature[index] * 0.15 + 0.7) * limit + weird * scale;
                scale = 0.002;
                limit = 1.0 - scale;
                let mut humi = (humidity[index] * 0.15 + 0.5) * limit + weird * scale;
                temp = 1.0 - (1.0 - temp) * (1.0 - temp);
                // Limit values to 0.0 - 1.0
                if temp < 0.0 {
                    temp = 0.0;
                }
                if humi < 0.0 {
                    humi = 0.0;
                }
                if temp > 1.0 {
                    temp = 1.0;
                }
                if humi > 1.0 {
                    humi = 1.0;
                }
                // Write the temperature and humidity values back
                temperature[index] = temp;
                humidity[index] = humi;
                // Get the biome from the lookup
                biome_map[index] = get_biome_from_lookup(temp as f32, humi as f32);
                index += 1;
            }
        }
    }

    pub fn get_biome_at_point(&mut self, world_pos: Int2) -> Biome {
        let mut temp = vec![0.0; 1];
        let mut humi = vec![0.0; 1];
        let mut weird = vec![0.0; 1];

        self.temperature_noise_gen.generate_octaves(
            &mut temp,
            Int2::new(world_pos.x, world_pos.y),
            Int32_2::new(1, 1),
            Vec2::new(0.025f32 as f64, 0.025f32 as f64),
            0.25,
        );
        self.humidity_noise_gen.generate_octaves(
            &mut humi,
            Int2::new(world_pos.x, world_pos.y),
            Int32_2::new(1, 1),
            Vec2::new(0.05f32 as f64, 0.05f32 as f64),
            1.0 / 3.0,
        );
        self.weirdness_noise_gen.generate_octaves(
            &mut weird,
            Int2::new(world_pos.x, world_pos.y),
            Int32_2::new(1, 1),
            Vec2::new(0.25, 0.25),
            0.5882352941176471,
        );

        let w = weird[0] * 1.1 + 0.5;
        let mut t = (temp[0] * 0.15 + 0.7) * 0.99 + w * 0.01;
        let mut h = (humi[0] * 0.15 + 0.5) * 0.998 + w * 0.002;
        t = 1.0 - (1.0 - t) * (1.0 - t);
        if t < 0.0 {
            t = 0.0;
        }
        if t > 1.0 {
            t = 1.0;
        }
        if h < 0.0 {
            h = 0.0;
        }
        if h > 1.0 {
            h = 1.0;
        }

        get_biome_from_lookup(t as f32, h as f32)
    }

    /// @brief Generates the temperature map values
    ///
    /// @param temperature The temperature map that'll be used/written to during generation
    /// @param weirdness The weirdness map that'll be used/written to during generation
    /// @param blockPos The x,z block-space coordindate of the chunk
    /// @param max The size of the area that'll be generated (16x16 by default)
    pub fn generate_temperature(&mut self, temperature: &mut Vec<f64>, weirdness: &mut Vec<f64>, block_pos: Int2, max: Int2) {
        if temperature.is_empty() || temperature.len() < (max.x * max.y) as usize {
            temperature.resize((max.x * max.y) as usize, 0.0);
        }

        self.temperature_noise_gen.generate_octaves(temperature, block_pos, max, Vec2::new(0.025f32 as f64, 0.025f32 as f64), 0.25);
        self.weirdness_noise_gen.generate_octaves(weirdness, block_pos, max, Vec2::new(0.25, 0.25), 0.5882352941176471);
        let mut index = 0usize;

        // Iterate over each block column
        for _x in 0..max.x {
            for _z in 0..max.y {
                let weird = weirdness[index] * 1.1 + 0.5;
                let scale = 0.01;
                let limit = 1.0 - scale;
                let mut temp = (temperature[index] * 0.15 + 0.7) * limit + weird * scale;
                temp = 1.0 - (1.0 - temp) * (1.0 - temp);
                // Limit values to 0.0 - 1.0
                if temp < 0.0 {
                    temp = 0.0;
                }
                if temp > 1.0 {
                    temp = 1.0;
                }
                // Write the temperature values back
                temperature[index] = temp;
                index += 1;
            }
        }
    }
}

impl Default for BiomeGenerator {
    fn default() -> Self {
        Self::new(0)
    }
}
