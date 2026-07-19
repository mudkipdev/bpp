/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use crate::blocks::block_properties;
use crate::constants::{CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::enums::blocks::{BLOCK_AIR, BlockType};
use crate::helpers::cross_platform::Math;
use crate::nbt::nbt::Tag;
use crate::numeric_structs::{Int2, Int3, Int32_2, INT32_2_ZERO};
use crate::tile_entities::tile_entity::TileEntityBehavior;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ChunkState {
    Unloaded = 0,
    Generating = 1,
    Loading = 2,
    Generated = 3,
    Populating = 4,
    Populated = 5,
    Unloading = 6,
}

impl ChunkState {
    fn from_u8(value: u8) -> Self {
        match value {
            0 => ChunkState::Unloaded,
            1 => ChunkState::Generating,
            2 => ChunkState::Loading,
            3 => ChunkState::Generated,
            4 => ChunkState::Populating,
            5 => ChunkState::Populated,
            6 => ChunkState::Unloading,
            _ => unreachable!(),
        }
    }
}

pub struct Chunk {
    pub cpos: Int32_2,
    pub in_use: AtomicBool,

    // Flat arrays indexed by (y * CHUNK_WIDTH * CHUNK_WIDTH) + (z * CHUNK_WIDTH) + x
    pub blocks: [BlockType; Chunk::VOLUME],
    pub light_nibble: [u8; Chunk::VOLUME],
    pub nibble_block_meta: [u8; Chunk::META_VOLUME],

    pub state: AtomicU8,
    pub height_map: [u8; (CHUNK_WIDTH * CHUNK_WIDTH) as usize],
    pub temperature: [f32; (CHUNK_WIDTH * CHUNK_WIDTH) as usize],
    pub humidity: [f32; (CHUNK_WIDTH * CHUNK_WIDTH) as usize],

    pub is_terrain_populated: bool,
    pub is_modified: bool,
    pub spawn_chunk: bool,

    // Tile entities
    pub tile_entities: Vec<Arc<Mutex<dyn TileEntityBehavior + Send>>>,

    // Used for loading entities into the world from disk
    pub entity_tags: Vec<Tag>,
}

impl Chunk {
    pub const VOLUME: usize = (CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_WIDTH) as usize;
    pub const META_VOLUME: usize = Self::VOLUME / 2;

    pub fn state_load(&self) -> ChunkState {
        ChunkState::from_u8(self.state.load(Ordering::SeqCst))
    }

    pub fn state_store(&self, state: ChunkState) {
        self.state.store(state as u8, Ordering::SeqCst);
    }

    pub fn block_index(&self, pos: Int3) -> usize {
        ((pos.y * CHUNK_WIDTH * CHUNK_WIDTH) + (pos.z * CHUNK_WIDTH) + pos.x) as usize
    }

    pub fn set_nibble(&self, hi: u8, lo: u8) -> u8 {
        ((hi & 0x0F) << 4) | (lo & 0x0F)
    }
    pub fn get_nibble_low(&self, byte: u8) -> u8 {
        byte & 0x0F
    }
    pub fn get_nibble_high(&self, byte: u8) -> u8 {
        (byte >> 4) & 0x0F
    }

    pub fn get_temperature(&self, pos: Int2) -> f32 {
        self.temperature[((pos.x << 4) | pos.y) as usize]
    }
    pub fn get_humidity(&self, pos: Int2) -> f32 {
        self.humidity[((pos.x << 4) | pos.y) as usize]
    }

    pub fn get_height_value(&self, pos: Int2) -> u8 {
        self.height_map[((pos.y << 4) | pos.x) as usize]
    }
    pub fn set_height_value(&mut self, pos: Int2, val: u8) {
        self.height_map[((pos.y << 4) | pos.x) as usize] = val;
    }
    pub fn can_block_see_sky(&self, pos: Int3) -> bool {
        pos.y >= i32::from(self.get_height_value(Int2::new(pos.x, pos.z)))
    }

    pub fn generate_height_map(&mut self) {
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                self.generate_height_map_column(Int2::new(x, z));
            }
        }
    }

    pub fn generate_height_map_column(&mut self, pos: Int2) {
        for y in (0..CHUNK_HEIGHT).rev() {
            if block_properties::block_properties()[self.get_block(Int3::new(pos.x, y, *pos.z())).0 as u8 as usize]
                .light_opacity
                > 0
            {
                self.set_height_value(pos, (y + 1) as u8);
                return;
            }
        }
        self.set_height_value(pos, 0);
    }

    pub fn generate_skylight_map(&mut self) {
        self.generate_height_map();
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                let height = i32::from(self.get_height_value(Int2::new(x, z)));
                for y in (height..CHUNK_HEIGHT).rev() {
                    self.set_sky_light(Int3::new(x, y, z), 15);
                }
                let mut sky_light: i32 = 15;
                for y in (0..height).rev() {
                    sky_light -= Math::max(
                        1,
                        i32::from(block_properties::block_properties()[self.get_block(Int3::new(x, y, z)).0 as u8 as usize].light_opacity),
                    );
                    sky_light = Math::max(0, sky_light);
                    self.set_sky_light(Int3::new(x, y, z), sky_light as u8);
                }
            }
        }
    }

    pub fn relight_column(&mut self, pos: Int2) {
        self.generate_height_map_column(pos);
        let height = i32::from(self.get_height_value(pos));

        for y in (height..CHUNK_HEIGHT).rev() {
            self.set_sky_light(Int3::new(pos.x, y, *pos.z()), 15);
        }

        // only pull values up
        let mut sky_light: i32 = 15;
        for y in (0..height).rev() {
            sky_light -= Math::max(
                1,
                i32::from(
                    block_properties::block_properties()[self.get_block(Int3::new(pos.x, y, *pos.z())).0 as u8 as usize].light_opacity,
                ),
            );
            sky_light = Math::max(0, sky_light);
            let current = self.get_sky_light(Int3::new(pos.x, y, *pos.z()));
            if i32::from(current) < sky_light {
                self.set_sky_light(Int3::new(pos.x, y, *pos.z()), sky_light as u8);
            }
        }
    }

    pub fn get_block(&self, pos: Int3) -> BlockType {
        self.blocks[self.block_index(pos)]
    }

    pub fn set_block(&mut self, pos: Int3, id: BlockType) {
        let index = self.block_index(pos);
        self.blocks[index] = id;
        self.is_modified = true;
    }

    pub fn get_meta(&self, pos: Int3) -> u8 {
        let idx = self.block_index(pos);
        let byte = self.nibble_block_meta[idx >> 1];
        if idx & 1 != 0 { self.get_nibble_high(byte) } else { self.get_nibble_low(byte) }
    }

    pub fn set_meta(&mut self, pos: Int3, meta: u8) {
        let idx = self.block_index(pos);
        let byte = self.nibble_block_meta[idx >> 1];
        self.nibble_block_meta[idx >> 1] = if idx & 1 != 0 {
            self.set_nibble(meta, self.get_nibble_low(byte))
        } else {
            self.set_nibble(self.get_nibble_high(byte), meta)
        };
        self.is_modified = true;
    }

    pub fn get_block_light(&self, pos: Int3) -> u8 {
        self.get_nibble_low(self.light_nibble[self.block_index(pos)])
    }

    pub fn get_sky_light(&self, pos: Int3) -> u8 {
        self.get_nibble_high(self.light_nibble[self.block_index(pos)])
    }

    pub fn set_block_light(&mut self, pos: Int3, val: u8) {
        let idx = self.block_index(pos);
        let byte = self.light_nibble[idx];
        self.light_nibble[idx] = self.set_nibble(self.get_nibble_high(byte), val);
        self.is_modified = true;
    }

    pub fn set_sky_light(&mut self, pos: Int3, val: u8) {
        let idx = self.block_index(pos);
        let byte = self.light_nibble[idx];
        self.light_nibble[idx] = self.set_nibble(val, self.get_nibble_low(byte));
        self.is_modified = true;
    }

    pub fn get_block_light_value(&self, pos: Int3, sky_subtracted: i32) -> i32 {
        let sky = Math::max(0, i32::from(self.get_sky_light(pos)) - sky_subtracted);
        let block = i32::from(self.get_block_light(pos));
        Math::min(15, Math::max(sky, block))
    }

    pub fn clear(&mut self) {
        self.is_terrain_populated = false;
        self.is_modified = false;
        self.blocks = [BLOCK_AIR; Chunk::VOLUME];
        self.light_nibble = [0; Chunk::VOLUME];
        self.nibble_block_meta = [0; Chunk::META_VOLUME];
        self.height_map = [0; (CHUNK_WIDTH * CHUNK_WIDTH) as usize];
        self.temperature = [0.0; (CHUNK_WIDTH * CHUNK_WIDTH) as usize];
        self.humidity = [0.0; (CHUNK_WIDTH * CHUNK_WIDTH) as usize];
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            cpos: INT32_2_ZERO,
            in_use: AtomicBool::new(false),
            blocks: [BLOCK_AIR; Chunk::VOLUME],
            light_nibble: [0; Chunk::VOLUME],
            nibble_block_meta: [0; Chunk::META_VOLUME],
            state: AtomicU8::new(ChunkState::Unloaded as u8),
            height_map: [0; (CHUNK_WIDTH * CHUNK_WIDTH) as usize],
            temperature: [0.0; (CHUNK_WIDTH * CHUNK_WIDTH) as usize],
            humidity: [0.0; (CHUNK_WIDTH * CHUNK_WIDTH) as usize],
            is_terrain_populated: false,
            is_modified: false,
            spawn_chunk: false,
            tile_entities: Vec::new(),
            entity_tags: Vec::new(),
        }
    }
}
