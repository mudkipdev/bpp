/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

pub const CHUNK_HEIGHT: i32 = 128;
pub const CHUNK_WIDTH: i32 = 16;
pub const CHUNK_AREA: i32 = CHUNK_WIDTH * CHUNK_WIDTH;
pub const CHUNK_VOLUME: i32 = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_WIDTH;

pub const SUB_CHUNK_SIZE: i32 = 16;
pub const SUB_CHUNK_VOLUME: i32 = SUB_CHUNK_SIZE * SUB_CHUNK_SIZE * SUB_CHUNK_SIZE;

pub const WATER_LEVEL: i32 = 64;
pub const NETHER_LAVA_LEVEL: i32 = 32;
// NOTE: Notch just copy-pasted the code for the overworld generator when making the Nether,
// so some values got wrongfully duplicated in the process.
pub const NETHER_BIOME_LAVA_LEVEL: i32 = 64; // Comes about due to a copy-paste error by notch

pub const PLAYER_EYE_HEIGHT: f64 = 1.62;
