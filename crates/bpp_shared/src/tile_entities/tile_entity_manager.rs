/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
use std::sync::{Arc, Mutex, Weak};

use crate::tile_entities::tile_entity::TileEntityBehavior;

// Simple wrapper so we don't have to manually add
pub struct TileEntityManager {
    pub tickable_tile_entities: Vec<Weak<Mutex<dyn TileEntityBehavior + Send>>>,
}

impl TileEntityManager {
    pub fn new() -> Self {
        TileEntityManager { tickable_tile_entities: Vec::new() }
    }

    // Initialize a tile entity into the world
    pub fn initialize_tile_entity(&mut self, tile_entity: &Arc<Mutex<dyn TileEntityBehavior + Send>>) {
        if tile_entity.lock().unwrap().base().can_tick {
            self.tickable_tile_entities.push(Arc::downgrade(tile_entity));
        }
    }

    pub fn tick_tile_entities(&mut self) {
        self.tickable_tile_entities.retain(|wp| match wp.upgrade() {
            Some(te) => {
                te.lock().unwrap().tick();
                true
            }
            None => false,
        });
    }
}
