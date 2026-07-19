/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use crate::numeric_structs::{Int2, Int3, Vec3, VEC3_ZERO};

// This struct stores the client's position as well as some helper functions to get their current chunk and exact block coords.
// This is used by the world manager to determine which chunks to load and unload around a player
pub struct ClientPosition {
    pub pos: Vec3,
    pub view_distance_override: i32,
}

impl ClientPosition {
    pub fn get_chunk_pos(&self) -> Int2 {
        Int2::new(self.pos.x.floor() as i32 >> 4, self.pos.z.floor() as i32 >> 4)
    }

    pub fn get_block_pos(&self) -> Int3 {
        Int3::new(self.pos.x.floor() as i32, self.pos.y.floor() as i32, self.pos.z.floor() as i32)
    }

    pub fn get_region_pos(&self) -> Int2 {
        Int2::new(self.pos.x.floor() as i32 >> 9, self.pos.z.floor() as i32 >> 9)
    }
}

impl Default for ClientPosition {
    fn default() -> Self {
        Self { pos: VEC3_ZERO, view_distance_override: 0 }
    }
}
