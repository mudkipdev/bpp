/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 * Copyright (c) 2026, jwaxy <jwaxy.is-a.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::fmt;
use std::hash::{Hash, Hasher};

use crate::base_types::{ItemAmount, ItemDamage, ItemId};
use crate::enums::items;
use crate::helpers::hash::hash_combine;

// Just a virtual container
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemStack {
    pub id: ItemId,
    pub count: ItemAmount,
    pub data: ItemDamage, // This is "damage" in the og java but data makes more sense for what this is used for
}

impl Default for ItemStack {
    fn default() -> Self {
        ItemStack {
            id: items::INVALID,
            count: 0,
            data: 0,
        }
    }
}

impl ItemStack {
    pub fn decrement_count(&mut self, amount: i8) {
        if (self.count as i32) - (amount as i32) < 0 {
            self.count = 0;
        } else {
            self.count -= amount;
        }
        if self.count <= 0 {
            self.id = items::INVALID;
            self.data = 0;
        }
    }

    pub fn str(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for ItemStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}:{} x{})",
            self.id.value() as i64,
            self.data as i64,
            self.count as i64
        )
    }
}

impl Hash for ItemStack {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut h: u64 = 0;
        hash_combine(&mut h, self.id);
        hash_combine(&mut h, self.count);
        hash_combine(&mut h, self.data);
        h.hash(state);
    }
}
