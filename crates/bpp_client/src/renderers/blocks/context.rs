/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use bpp_shared::enums::blocks::{BLOCK_AIR, BlockType};
use bpp_shared::numeric_structs::Int3;

// This is a struct that contains a bunch of relevant data a block may need during meshing; especially liquids
// Makes face lookups a LOT faster than having to go through the world object every time
// The relevant caches are built in the sub chunk mesher

pub struct BlockData {
    pub pos: Int3,
    pub block_id: BlockType,
    pub block_meta: u8,
    pub block_light: f32, // Converted value from 0-15 to the range 0.0-1.0 using the specified dimension's brightness curve
}

impl Default for BlockData {
    fn default() -> Self {
        Self { pos: Int3::new(0, 0, 0), block_id: BLOCK_AIR, block_meta: 0, block_light: 0.0 }
    }
}

// This struct is a little heavy but since each mesher only has one instance of this render context at a time it should be fine
pub struct BlockRenderContext {
    pub temperature: f32,
    pub humidity: f32,
    pub world_x: i32,
    pub world_y: i32,
    pub world_z: i32,
    pub fancy_graphics: bool,
    pub smooth_lighting: bool,

    // 3x3x3 cube of blocks centered on the block being rendered
    // You can access the current block by requestion the neighbor with dx=0, dy=0, dz=0
    pub neighbors: [BlockData; 27],
}

impl BlockRenderContext {
    pub fn neighbor_index(&self, dx: i32, dy: i32, dz: i32) -> usize {
        ((dy + 1) * 9 + (dz + 1) * 3 + (dx + 1)) as usize
    }

    pub fn get_neighbor(&mut self, dx: i32, dy: i32, dz: i32) -> &mut BlockData {
        let index = self.neighbor_index(dx, dy, dz);
        &mut self.neighbors[index]
    }
}

impl Default for BlockRenderContext {
    fn default() -> Self {
        Self {
            temperature: 0.0,
            humidity: 0.0,
            world_x: 0,
            world_y: 0,
            world_z: 0,
            fancy_graphics: false,
            smooth_lighting: false,
            neighbors: std::array::from_fn(|_| BlockData::default()),
        }
    }
}
