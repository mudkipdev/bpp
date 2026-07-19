/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use crate::enums::blocks::{BLOCK_DIRT, BLOCK_GRASS, BLOCK_SAND, BlockType};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Biome {
    None = 0,
    Rainforest = 1,
    Swampland = 2,
    SeasonalForest = 3,
    Forest = 4,
    Savanna = 5,
    Shrubland = 6,
    Taiga = 7,
    Desert = 8,
    Plains = 9,
    IceDesert = 10,
    Tundra = 11,
    Hell = 12,
    Sky = 13,
}

/// @brief Get the Top Block object
///
/// @param biome The biome to get the top/surface block of
/// @return The top/surface block BlockType
pub fn get_top_block(biome: Biome) -> BlockType {
    if biome == Biome::Desert || biome == Biome::IceDesert {
        return BLOCK_SAND;
    }
    BLOCK_GRASS
}

/// @brief Get the Filler Block object
///
/// @param biome The biome to get the filler block of
/// @return The filler block BlockType
pub fn get_filler_block(biome: Biome) -> BlockType {
    if biome == Biome::Desert || biome == Biome::IceDesert {
        return BLOCK_SAND;
    }
    BLOCK_DIRT
}

/// @brief Get the correct biome based on the passed temperature and humidity values
///
/// @param temperature Temperature value
/// @param humidity Humidity/Downfall value
/// @return The appropriate Biome for the passed values
pub fn get_biome(temperature: f32, humidity: f32) -> Biome {
    let humidity = humidity * temperature;
    if temperature < 0.1 {
        return Biome::Tundra;
    }
    if humidity < 0.2 {
        if temperature < 0.5 {
            return Biome::Tundra;
        }
        if temperature < 0.95 {
            return Biome::Savanna;
        }
        return Biome::Desert;
    }
    if humidity > 0.5 && temperature < 0.7 {
        return Biome::Swampland;
    }
    if temperature < 0.5 {
        return Biome::Taiga;
    }
    if temperature < 0.97 {
        if humidity < 0.35 {
            return Biome::Shrubland;
        }
        return Biome::Forest;
    }
    if humidity < 0.45 {
        return Biome::Plains;
    }
    if humidity < 0.9 {
        return Biome::SeasonalForest;
    }
    Biome::Rainforest
}

/// @brief Gets the appropriate biome from the Biome LUT
///
/// @param temperature Temperature value
/// @param humidity Humidity/Downfall value
/// @return The appropriate Biome for the passed values
pub fn get_biome_from_lookup(temperature: f32, humidity: f32) -> Biome {
    let temp = (temperature * 63.0) as i32;
    let humi = (humidity * 63.0) as i32;
    biome_lut()[(temp + humi * 64) as usize]
}

pub const BIOME_LUT_SIZE: usize = 64 * 64;

/// @brief Generates the Biome LUT that is used in b1.7.3
///
fn biome_lut() -> &'static [Biome; BIOME_LUT_SIZE] {
    static BIOME_LUT: OnceLock<[Biome; BIOME_LUT_SIZE]> = OnceLock::new();
    BIOME_LUT.get_or_init(|| {
        let mut lut = [Biome::None; BIOME_LUT_SIZE];
        for temp in 0..64usize {
            for humi in 0..64usize {
                lut[temp + humi * 64] = get_biome(temp as f32 / 63.0, humi as f32 / 63.0);
            }
        }
        lut
    })
}
