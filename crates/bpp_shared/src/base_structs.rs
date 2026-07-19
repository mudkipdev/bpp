/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::fmt;

use crate::enums::blocks::{BLOCK_AIR, BlockType};

// Block Struct
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Block {
    pub r#type: BlockType,
    pub data: u8,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            r#type: BLOCK_AIR,
            data: 0,
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}:{})", i32::from(self.r#type.0), i32::from(self.data))
    }
}

// Lighting + Block Struct
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LitBlock {
    pub block: Block,
    pub blocklight: u8,
    pub skylight: u8,
}
