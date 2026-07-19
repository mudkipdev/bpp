/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Mojang Studios (2011)
*/

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::blocks::block_properties;
use crate::enums::blocks::{
    BLOCK_AIR, BLOCK_CACTUS, BLOCK_CHEST, BLOCK_CLAY, BLOCK_COBBLESTONE, BLOCK_COBBLESTONE_MOSSY, BLOCK_DEADBUSH,
    BLOCK_DIRT, BLOCK_FIRE, BLOCK_GLOWSTONE, BLOCK_GRASS, BLOCK_LAVA_FLOWING, BLOCK_LAVA_STILL, BLOCK_LEAVES,
    BLOCK_MOB_SPAWNER, BLOCK_MUSHROOM_BROWN, BLOCK_MUSHROOM_RED, BLOCK_NETHERRACK, BLOCK_PUMPKIN, BLOCK_SAND,
    BLOCK_STONE, BLOCK_SUGARCANE, BLOCK_WATER_FLOWING, BLOCK_WATER_STILL, BlockType,
};
use crate::enums::items;
use crate::helpers::cross_platform::Math;
use crate::helpers::java::java_math::{JavaMath, MathHelper};
use crate::helpers::java::java_random::Random;
use crate::inventory::inventory::InventoryBehavior;
use crate::inventory::item_stack::ItemStack;
use crate::numeric_structs::{Int2, Int3};
use crate::tile_entities::tile_entity::{TileEntityChest, TileEntityMobSpawner};
use crate::world::chunk::{Chunk, ChunkState};
use crate::world::lighter::LightType;
use crate::world::world::WorldManager;

// 3x3 region of chunk pointers, centered on the chunk being populated
#[derive(Clone, Default)]
pub struct ChunkPtrRegion {
    pub chunks: [[Option<Arc<Mutex<Chunk>>>; 3]; 3],
}

impl ChunkPtrRegion {
    pub fn get_chunk(&self, pos: Int2) -> Option<Arc<Mutex<Chunk>>> {
        if pos.x < -1 || pos.x > 1 || *pos.z() < -1 || *pos.z() > 1 {
            return None;
        }
        self.chunks[(pos.x + 1) as usize][(*pos.z() + 1) as usize].clone()
    }
}

// Wrapper for world access during chunk population.
// Holds a 3x3 region of chunk pointers centered on the chunk being populated.
// Chunks are marked inUse on acquire and released on free.
pub struct WorldWrapper<'a> {
    pub manager: &'a mut WorldManager,
    pub chunk_region: ChunkPtrRegion,
    pub center_chunk_pos: Int2,
}

impl<'a> WorldWrapper<'a> {
    pub fn new(manager: &'a mut WorldManager, center_chunk_pos: Int2) -> Self {
        Self { manager, chunk_region: ChunkPtrRegion::default(), center_chunk_pos }
    }

    // Grab the 3x3 region. Any chunk that is already inUse is left as nullptr
    // (writes to it will fall through to the deferred path via the m_manager).
    pub fn get_chunk_region(&mut self) {
        for dx in -1..=1 {
            for dz in -1..=1 {
                let ax = self.center_chunk_pos.x + dx;
                let az = *self.center_chunk_pos.z() + dz;
                let c = self.manager.get_chunk_raw(Int2::new(ax, az));
                self.chunk_region.chunks[(dx + 1) as usize][(dz + 1) as usize] = match c {
                    Some(c) if !c.lock().unwrap().in_use.load(Ordering::SeqCst) => Some(c),
                    _ => None,
                };
            }
        }
        // Mark all successfully acquired chunks as inUse
        for row in self.chunk_region.chunks.iter() {
            for c in row.iter() {
                if let Some(c) = c {
                    c.lock().unwrap().in_use.store(true, Ordering::SeqCst);
                }
            }
        }
    }

    pub fn free_chunk_region(&mut self) {
        for row in self.chunk_region.chunks.iter() {
            for c in row.iter() {
                if let Some(c) = c {
                    c.lock().unwrap().in_use.store(false, Ordering::SeqCst);
                }
            }
        }
    }

    // Convert a world-space position to a region-local chunk offset (-1..1, -1..1)
    pub fn get_region_chunk_pos(&self, wpos: Int3) -> Int2 {
        Int2::new((wpos.x >> 4) - self.center_chunk_pos.x, (wpos.z >> 4) - *self.center_chunk_pos.z())
    }

    pub fn find_top_solid_block(&self, wx: i32, wz: i32) -> i32 {
        let chunk = match self.chunk_region.get_chunk(self.get_region_chunk_pos(Int3::new(wx, 0, wz))) {
            Some(chunk) => chunk,
            None => return -1,
        };
        let guard = chunk.lock().unwrap();
        if guard.state_load() < ChunkState::Generated {
            return -1;
        }
        let lx = wx & 15;
        let lz = wz & 15;
        for y in (1..=127).rev() {
            let block = guard.get_block(Int3::new(lx, y, lz));
            if block == BLOCK_AIR {
                continue;
            }
            let mat = block_properties::block_properties()[block.0 as u8 as usize].material;
            if mat.is_solid || mat.is_liquid {
                return y + 1;
            }
        }
        -1
    }

    pub fn get_height_value(&self, wx: i32, wz: i32) -> i32 {
        let chunk = match self.chunk_region.get_chunk(self.get_region_chunk_pos(Int3::new(wx, 0, wz))) {
            Some(chunk) => chunk,
            None => return 0,
        };
        let guard = chunk.lock().unwrap();
        if guard.state_load() < ChunkState::Generated {
            return 0;
        }
        i32::from(guard.get_height_value(Int2::new(wx & 15, wz & 15)))
    }

    pub fn get_temperature_at(&self, wx: i32, wz: i32) -> f64 {
        let chunk = match self.chunk_region.get_chunk(self.get_region_chunk_pos(Int3::new(wx, 0, wz))) {
            Some(chunk) => chunk,
            None => return 0.5,
        };
        let guard = chunk.lock().unwrap();
        if guard.state_load() < ChunkState::Generated {
            return 0.5;
        }
        f64::from(guard.get_temperature(Int2::new(wx & 15, wz & 15)))
    }

    pub fn get_humidity_at(&self, wx: i32, wz: i32) -> f64 {
        let chunk = match self.chunk_region.get_chunk(self.get_region_chunk_pos(Int3::new(wx, 0, wz))) {
            Some(chunk) => chunk,
            None => return 0.5,
        };
        let guard = chunk.lock().unwrap();
        if guard.state_load() < ChunkState::Generated {
            return 0.5;
        }
        f64::from(guard.get_humidity(Int2::new(wx & 15, wz & 15)))
    }

    pub fn get_block_id(&self, wpos: Int3) -> BlockType {
        if !Self::in_bounds(wpos.y) {
            return BLOCK_AIR;
        }
        let chunk = self.chunk_region.get_chunk(self.get_region_chunk_pos(wpos));
        // Falls outside our grabbed region -> ask the m_manager directly (read-only, safe)
        let chunk = match chunk {
            Some(chunk) => chunk,
            None => return self.manager.get_block_id(wpos),
        };
        let guard = chunk.lock().unwrap();
        if guard.state_load() < ChunkState::Generated {
            return BLOCK_AIR;
        }
        guard.get_block(Int3::new(wpos.x & 15, wpos.y, wpos.z & 15))
    }

    pub fn set_block(&mut self, wpos: Int3, r#type: BlockType, meta: u8) {
        if !Self::in_bounds(wpos.y) {
            return;
        }
        let chunk = self.chunk_region.get_chunk(self.get_region_chunk_pos(wpos));
        let chunk = match chunk {
            Some(chunk) if chunk.lock().unwrap().state_load() >= ChunkState::Generated => chunk,
            _ => {
                // Outside our locked region
                self.manager.set_block(wpos, r#type, meta);
                return;
            }
        };

        // Remove any tile entities that exist at this spot
        {
            let mut guard = chunk.lock().unwrap();
            guard.tile_entities.retain(|te| {
                let locked = te.lock().unwrap();
                locked.base().position != wpos
            });
        }

        // Unlight before changing the block
        self.unlight_at(wpos.x, wpos.y, wpos.z, LightType::Block);
        self.unlight_at(wpos.x, wpos.y, wpos.z, LightType::Sky);

        // Get the local coordinates of this block within the chunk and set it
        let lx = wpos.x & 15;
        let lz = wpos.z & 15;
        let local = Int3::new(lx, wpos.y, lz);
        let old_height = {
            let mut guard = chunk.lock().unwrap();
            guard.set_block(local, r#type);
            guard.set_meta(local, meta);
            i32::from(guard.get_height_value(Int2::new(lx, lz)))
        };

        let y = wpos.y;
        let x = wpos.x;
        let z = wpos.z;

        if block_properties::block_properties()[r#type.0 as u8 as usize].light_opacity != 0 {
            // Placing opaque block; heightmap may rise
            if y >= old_height {
                chunk.lock().unwrap().relight_column(Int2::new(lx, lz));

                // The column below the new top was zeroed out by relightColumn.
                // Notify the BFS that all blocks from y down to oldHeight need updating
                for sy in old_height..=y {
                    self.unlight_at(x, sy, z, LightType::Sky);
                }
            }
        } else if y == old_height - 1 {
            // Removing top opaque block; heightmap may fall
            chunk.lock().unwrap().relight_column(Int2::new(lx, lz));
        }

        let new_height = i32::from(chunk.lock().unwrap().get_height_value(Int2::new(lx, lz)));
        if new_height < old_height {
            for sy in new_height..old_height {
                self.manager.light_manager.schedule_light_update(Int3::new(x, sy, z), LightType::Sky);
            }
        }

        // Always re-evaluate the edited block and its 4 horizontal neighbours
        // across the height transition band.
        self.manager.light_manager.schedule_light_update(Int3::new(x, y, z), LightType::Sky);
        let ndx = [-1, 1, 0, 0];
        let ndz = [0, 0, -1, 1];
        for i in 0..4 {
            let nx = x + ndx[i];
            let nz = z + ndz[i];
            let neighbor_height = self.get_height_value(nx, nz);
            let this_height = i32::from(chunk.lock().unwrap().get_height_value(Int2::new(lx, lz)));
            if neighbor_height == this_height {
                continue;
            }
            let min_y = Math::min(this_height, neighbor_height);
            let max_y = Math::max(this_height, neighbor_height);
            self.manager.light_manager.schedule_light_region(Int3::new(nx, min_y, nz), Int3::new(nx, max_y, nz), LightType::Sky);
        }
        // Schedule a block light update for the position itself
        self.manager.light_manager.schedule_light_update(Int3::new(x, y, z), LightType::Block);

        // Callback for the client and server to know about this block update
        let (block_light, sky_light, cpos) = {
            let guard = chunk.lock().unwrap();
            (
                guard.get_block_light(Int3::new(wpos.x & 15, wpos.y, wpos.z & 15)),
                guard.get_sky_light(Int3::new(wpos.x & 15, wpos.y, wpos.z & 15)),
                guard.cpos,
            )
        };
        if let Some(callback) = self.manager.on_block_update.as_mut() {
            callback(
                crate::world::world::PendingBlock {
                    block: crate::base_structs::Block { r#type, data: meta },
                    block_pos: wpos,
                    light: Int2::new(i32::from(block_light), i32::from(sky_light)),
                },
                cpos,
            );
        }
    }

    pub fn get_sky_light(&self, wpos: Int3) -> u8 {
        if !Self::in_bounds(wpos.y) {
            return 0;
        }
        let chunk = self.chunk_region.get_chunk(self.get_region_chunk_pos(wpos));
        let chunk = match chunk {
            Some(chunk) => chunk,
            None => return 0,
        };
        let guard = chunk.lock().unwrap();
        if guard.state_load() < ChunkState::Generated {
            return 0;
        }
        guard.get_sky_light(Int3::new(wpos.x & 15, wpos.y, wpos.z & 15))
    }

    pub fn get_seed(&self) -> i64 {
        self.manager.seed
    }

    // Returns true when the world-space Y is within valid chunk bounds.
    pub const fn in_bounds(y: i32) -> bool {
        y >= 0 && y < crate::constants::CHUNK_HEIGHT
    }

    fn unlight_at(&mut self, x: i32, y: i32, z: i32, r#type: LightType) {
        let mut light_manager = std::mem::take(&mut self.manager.light_manager);
        light_manager.unlight_at(x, y, z, r#type, self.manager);
        self.manager.light_manager = light_manager;
    }
}

// Inline block-property helpers
pub fn is_solid(t: BlockType) -> bool {
    block_properties::block_properties()[t.0 as u8 as usize].material.is_solid
}
pub fn is_liquid(t: BlockType) -> bool {
    block_properties::block_properties()[t.0 as u8 as usize].material.is_liquid
}
pub fn is_opaque(t: BlockType) -> bool {
    block_properties::block_properties()[t.0 as u8 as usize].light_opacity > 0
}

// Used for generating features in the world
pub struct FeatureGenerator {
    pub r#type: BlockType,
    pub meta: i8,
}

impl FeatureGenerator {
    pub fn new(r#type: BlockType) -> Self {
        Self { r#type, meta: 0 }
    }

    pub fn with_type(r#type: BlockType) -> Self {
        Self { r#type, meta: 0 }
    }

    pub fn with_meta(r#type: BlockType, meta: i8) -> Self {
        Self { r#type, meta }
    }

    //  GenerateLake
    pub fn generate_lake(&self, world: &mut WorldWrapper, rand: &mut Random, mut pos: Int3) -> bool {
        pos.x -= 8;
        pos.z -= 8;

        // Sink to first non-air block
        while pos.y > 0 && world.get_block_id(Int3::new(pos.x, pos.y, pos.z)) == BLOCK_AIR {
            pos.y -= 1;
        }

        pos.y -= 4;

        let mut shape_mask = [false; 2048];
        let blob_count = rand.next_int_bound(4) + 4;

        for _blob_index in 0..blob_count {
            let rad_x = rand.next_double() * 6.0 + 3.0;
            let rad_y = rand.next_double() * 4.0 + 2.0;
            let rad_z = rand.next_double() * 6.0 + 3.0;
            let cx = rand.next_double() * (16.0 - rad_x - 2.0) + 1.0 + rad_x / 2.0;
            let cy = rand.next_double() * (8.0 - rad_y - 4.0) + 2.0 + rad_y / 2.0;
            let cz = rand.next_double() * (16.0 - rad_z - 2.0) + 1.0 + rad_z / 2.0;

            for x in 1..15 {
                for z in 1..15 {
                    for y in 1..7 {
                        let dx = (f64::from(x) - cx) / (rad_x / 2.0);
                        let dy = (f64::from(y) - cy) / (rad_y / 2.0);
                        let dz = (f64::from(z) - cz) / (rad_z / 2.0);
                        if dx * dx + dy * dy + dz * dz < 1.0 {
                            shape_mask[((x * 16 + z) * 8 + y) as usize] = true;
                        }
                    }
                }
            }
        }

        // Reject if edges touch existing liquid (above waterline) or non-solid/wrong block (below)
        for x in 0..16 {
            for z in 0..16 {
                for y in 0..8 {
                    let edge = !shape_mask[((x * 16 + z) * 8 + y) as usize]
                        && ((x < 15 && shape_mask[(((x + 1) * 16 + z) * 8 + y) as usize])
                            || (x > 0 && shape_mask[(((x - 1) * 16 + z) * 8 + y) as usize])
                            || (z < 15 && shape_mask[((x * 16 + z + 1) * 8 + y) as usize])
                            || (z > 0 && shape_mask[((x * 16 + z - 1) * 8 + y) as usize])
                            || (y < 7 && shape_mask[((x * 16 + z) * 8 + y + 1) as usize])
                            || (y > 0 && shape_mask[((x * 16 + z) * 8 + y - 1) as usize]));
                    if !edge {
                        continue;
                    }
                    let bt = world.get_block_id(Int3::new(pos.x + x, pos.y + y, pos.z + z));
                    if y >= 4 && is_liquid(bt) {
                        return false;
                    }
                    if y < 4 && !is_solid(bt) && bt != self.r#type {
                        return false;
                    }
                }
            }
        }

        // Fill
        for x in 0..16 {
            for z in 0..16 {
                for y in 0..8 {
                    if shape_mask[((x * 16 + z) * 8 + y) as usize] {
                        world.set_block(
                            Int3::new(pos.x + x, pos.y + y, pos.z + z),
                            if y >= 4 { BLOCK_AIR } else { self.r#type },
                            0,
                        );
                    }
                }
            }
        }

        // Exposed dirt -> grass
        for x in 0..16 {
            for z in 0..16 {
                for y in 4..8 {
                    if shape_mask[((x * 16 + z) * 8 + y) as usize]
                        && world.get_block_id(Int3::new(pos.x + x, pos.y + y - 1, pos.z + z)) == BLOCK_DIRT
                        && world.get_sky_light(Int3::new(pos.x + x, pos.y + y, pos.z + z)) > 0
                    {
                        world.set_block(Int3::new(pos.x + x, pos.y + y - 1, pos.z + z), BLOCK_GRASS, 0);
                    }
                }
            }
        }

        // Lava: solidify exposed edges
        if self.r#type == BLOCK_LAVA_STILL || self.r#type == BLOCK_LAVA_FLOWING {
            for x in 0..16 {
                for z in 0..16 {
                    for y in 0..8 {
                        let edge = !shape_mask[((x * 16 + z) * 8 + y) as usize]
                            && ((x < 15 && shape_mask[(((x + 1) * 16 + z) * 8 + y) as usize])
                                || (x > 0 && shape_mask[(((x - 1) * 16 + z) * 8 + y) as usize])
                                || (z < 15 && shape_mask[((x * 16 + z + 1) * 8 + y) as usize])
                                || (z > 0 && shape_mask[((x * 16 + z - 1) * 8 + y) as usize])
                                || (y < 7 && shape_mask[((x * 16 + z) * 8 + y + 1) as usize])
                                || (y > 0 && shape_mask[((x * 16 + z) * 8 + y - 1) as usize]));
                        if edge
                            && (y < 4 || rand.next_int_bound(2) != 0)
                            && is_solid(world.get_block_id(Int3::new(pos.x + x, pos.y + y, pos.z + z)))
                        {
                            world.set_block(Int3::new(pos.x + x, pos.y + y, pos.z + z), BLOCK_STONE, 0);
                        }
                    }
                }
            }
        }
        true
    }

    //  GenerateDungeon
    pub fn generate_dungeon(&self, world: &mut WorldWrapper, rand: &mut Random, pos: Int3) -> bool {
        let dungeon_height: i8 = 3;
        let dungeon_width_x = rand.next_int_bound(2) + 2;
        let dungeon_width_z = rand.next_int_bound(2) + 2;
        let mut valid_entries = 0;

        for xi in (pos.x - dungeon_width_x - 1)..=(pos.x + dungeon_width_x + 1) {
            for yi in (pos.y - 1)..=(pos.y + i32::from(dungeon_height) + 1) {
                for zi in (pos.z - dungeon_width_z - 1)..=(pos.z + dungeon_width_z + 1) {
                    let bt = world.get_block_id(Int3::new(xi, yi, zi));
                    if yi == pos.y - 1 && !is_solid(bt) {
                        return false;
                    }
                    if yi == pos.y + i32::from(dungeon_height) + 1 && !is_solid(bt) {
                        return false;
                    }
                    let is_wall = xi == pos.x - dungeon_width_x - 1
                        || xi == pos.x + dungeon_width_x + 1
                        || zi == pos.z - dungeon_width_z - 1
                        || zi == pos.z + dungeon_width_z + 1;
                    if is_wall
                        && yi == pos.y
                        && bt == BLOCK_AIR
                        && world.get_block_id(Int3::new(xi, yi + 1, zi)) == BLOCK_AIR
                    {
                        valid_entries += 1;
                    }
                }
            }
        }

        if valid_entries < 1 || valid_entries > 5 {
            return false;
        }

        for xi in (pos.x - dungeon_width_x - 1)..=(pos.x + dungeon_width_x + 1) {
            for yi in ((pos.y - 1)..=(pos.y + i32::from(dungeon_height))).rev() {
                for zi in (pos.z - dungeon_width_z - 1)..=(pos.z + dungeon_width_z + 1) {
                    let interior = xi != pos.x - dungeon_width_x - 1
                        && xi != pos.x + dungeon_width_x + 1
                        && yi != pos.y - 1
                        && yi != pos.y + i32::from(dungeon_height) + 1
                        && zi != pos.z - dungeon_width_z - 1
                        && zi != pos.z + dungeon_width_z + 1;
                    if interior {
                        world.set_block(Int3::new(xi, yi, zi), BLOCK_AIR, 0);
                    } else if yi >= 0 && !is_solid(world.get_block_id(Int3::new(xi, yi - 1, zi))) {
                        world.set_block(Int3::new(xi, yi, zi), BLOCK_AIR, 0);
                    } else if is_solid(world.get_block_id(Int3::new(xi, yi, zi))) {
                        let wall = if yi == pos.y - 1 && rand.next_int_bound(4) != 0 {
                            BLOCK_COBBLESTONE_MOSSY
                        } else {
                            BLOCK_COBBLESTONE
                        };
                        world.set_block(Int3::new(xi, yi, zi), wall, 0);
                    }
                }
            }
        }

        // Up to 2 chests, 3 placement attempts each
        for _chest_attempt in 0..2 {
            for _attempt in 0..3 {
                let cx = pos.x + rand.next_int_bound(dungeon_width_x * 2 + 1) - dungeon_width_x;
                let cz = pos.z + rand.next_int_bound(dungeon_width_z * 2 + 1) - dungeon_width_z;
                if world.get_block_id(Int3::new(cx, pos.y, cz)) != BLOCK_AIR {
                    continue;
                }
                let mut adj = 0;
                if is_solid(world.get_block_id(Int3::new(cx - 1, pos.y, cz))) {
                    adj += 1;
                }
                if is_solid(world.get_block_id(Int3::new(cx + 1, pos.y, cz))) {
                    adj += 1;
                }
                if is_solid(world.get_block_id(Int3::new(cx, pos.y, cz - 1))) {
                    adj += 1;
                }
                if is_solid(world.get_block_id(Int3::new(cx, pos.y, cz + 1))) {
                    adj += 1;
                }
                if adj == 1 {
                    world.set_block(Int3::new(cx, pos.y, cz), BLOCK_CHEST, 0);
                    let mut chest = TileEntityChest::new(Int3::new(cx, pos.y, cz));
                    for _slot in 0..8 {
                        let stack = Self::generate_dungeon_chest_loot(rand);
                        if stack.id != items::INVALID {
                            let slot_index = rand.next_int_bound(27);
                            chest.inventory.set_inventory_slot_contents(slot_index, Some(&stack));
                        }
                    }
                    world.manager.create_tile_entity(Arc::new(Mutex::new(chest)));
                    break;
                }
            }
        }

        world.set_block(pos, BLOCK_MOB_SPAWNER, 0);
        let mut spawner = TileEntityMobSpawner::new(pos);
        spawner.entity_id = Self::pick_mob_to_spawn(rand);
        world.manager.create_tile_entity(Arc::new(Mutex::new(spawner)));
        true
    }

    // Creates Dungeon Chest loot
    pub fn generate_dungeon_chest_loot(rand: &mut Random) -> ItemStack {
        let roll = rand.next_int_bound(11);
        match roll {
            0 => ItemStack { id: items::SADDLE, count: 1, data: 0 },
            1 => {
                let qty = (rand.next_int_bound(4) + 1) as i8;
                ItemStack { id: items::IRON, count: qty, data: 0 }
            }
            2 => ItemStack { id: items::BREAD, count: 1, data: 0 },
            3 => {
                let qty = (rand.next_int_bound(4) + 1) as i8;
                ItemStack { id: items::WHEAT, count: qty, data: 0 }
            }
            4 => {
                let qty = (rand.next_int_bound(4) + 1) as i8;
                ItemStack { id: items::GUNPOWDER, count: qty, data: 0 }
            }
            5 => {
                let qty = (rand.next_int_bound(4) + 1) as i8;
                ItemStack { id: items::STRING, count: qty, data: 0 }
            }
            6 => ItemStack { id: items::BUCKET, count: 1, data: 0 },
            7 => {
                if rand.next_int_bound(100) == 0 {
                    ItemStack { id: items::APPLE_GOLDEN, count: 1, data: 0 }
                } else {
                    ItemStack { id: items::INVALID, count: 0, data: 0 }
                }
            }
            8 => {
                if rand.next_int_bound(2) == 0 {
                    let qty = (rand.next_int_bound(4) + 1) as i8;
                    ItemStack { id: items::REDSTONE, count: qty, data: 0 }
                } else {
                    ItemStack { id: items::INVALID, count: 0, data: 0 }
                }
            }
            9 => {
                if rand.next_int_bound(10) == 0 {
                    let disc_id = if rand.next_int_bound(2) == 0 { items::RECORD_13 } else { items::RECORD_CAT };
                    ItemStack { id: disc_id, count: 1, data: 0 }
                } else {
                    ItemStack { id: items::INVALID, count: 0, data: 0 }
                }
            }
            10 => ItemStack { id: items::DYE, count: 1, data: 3 },
            _ => ItemStack { id: items::INVALID, count: 0, data: 0 },
        }
    }

    pub fn pick_mob_to_spawn(rand: &mut Random) -> String {
        match rand.next_int_bound(4) {
            0 => "Skeleton".to_string(),
            1 | 2 => "Zombie".to_string(),
            3 => "Spider".to_string(),
            _ => "Zombie".to_string(),
        }
    }

    //  GenerateClay
    pub fn generate_clay(&self, world: &mut WorldWrapper, rand: &mut Random, pos: Int3, blob_size: i32) -> bool {
        let at = world.get_block_id(pos);
        if at != BLOCK_WATER_STILL && at != BLOCK_WATER_FLOWING {
            return false;
        }

        let angle = rand.next_float() * JavaMath::PI_FLOAT;
        let x_start = f64::from((pos.x + 8) as f32 + MathHelper::sin(angle) * blob_size as f32 / 8.0);
        let x_end = f64::from((pos.x + 8) as f32 - MathHelper::sin(angle) * blob_size as f32 / 8.0);
        let z_start = f64::from((pos.z + 8) as f32 + MathHelper::cos(angle) * blob_size as f32 / 8.0);
        let z_end = f64::from((pos.z + 8) as f32 - MathHelper::cos(angle) * blob_size as f32 / 8.0);
        let y_start = f64::from(pos.y + rand.next_int_bound(3) + 2);
        let y_end = f64::from(pos.y + rand.next_int_bound(3) + 2);

        for i in 0..=blob_size {
            let x_c = x_start + (x_end - x_start) * f64::from(i) / f64::from(blob_size);
            let y_c = y_start + (y_end - y_start) * f64::from(i) / f64::from(blob_size);
            let z_c = z_start + (z_end - z_start) * f64::from(i) / f64::from(blob_size);
            let blob_scale = rand.next_double() * f64::from(blob_size) / 16.0;
            let rad_xz =
                f64::from(MathHelper::sin(i as f32 * JavaMath::PI_FLOAT / blob_size as f32) + 1.0) * blob_scale + 1.0;
            let rad_y =
                f64::from(MathHelper::sin(i as f32 * JavaMath::PI_FLOAT / blob_size as f32) + 1.0) * blob_scale + 1.0;
            let min_x = MathHelper::floor_double(x_c - rad_xz / 2.0);
            let max_x = MathHelper::floor_double(x_c + rad_xz / 2.0);
            let min_y = MathHelper::floor_double(y_c - rad_y / 2.0);
            let max_y = MathHelper::floor_double(y_c + rad_y / 2.0);
            let min_z = MathHelper::floor_double(z_c - rad_xz / 2.0);
            let max_z = MathHelper::floor_double(z_c + rad_xz / 2.0);
            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    for z in min_z..=max_z {
                        let dx = (f64::from(x) + 0.5 - x_c) / (rad_xz / 2.0);
                        let dy = (f64::from(y) + 0.5 - y_c) / (rad_y / 2.0);
                        let dz = (f64::from(z) + 0.5 - z_c) / (rad_xz / 2.0);
                        if dx * dx + dy * dy + dz * dz < 1.0 && world.get_block_id(Int3::new(x, y, z)) == BLOCK_SAND {
                            world.set_block(Int3::new(x, y, z), BLOCK_CLAY, 0);
                        }
                    }
                }
            }
        }
        true
    }

    //  GenerateMinable
    pub fn generate_minable(&self, world: &mut WorldWrapper, rand: &mut Random, pos: Int3, blob_size: i32) -> bool {
        let angle = rand.next_float() * JavaMath::PI_FLOAT;
        let x_start = f64::from((pos.x + 8) as f32 + MathHelper::sin(angle) * blob_size as f32 / 8.0);
        let x_end = f64::from((pos.x + 8) as f32 - MathHelper::sin(angle) * blob_size as f32 / 8.0);
        let z_start = f64::from((pos.z + 8) as f32 + MathHelper::cos(angle) * blob_size as f32 / 8.0);
        let z_end = f64::from((pos.z + 8) as f32 - MathHelper::cos(angle) * blob_size as f32 / 8.0);
        let y_start = f64::from(pos.y + rand.next_int_bound(3) + 2);
        let y_end = f64::from(pos.y + rand.next_int_bound(3) + 2);

        for i in 0..=blob_size {
            let x_c = x_start + (x_end - x_start) * f64::from(i) / f64::from(blob_size);
            let y_c = y_start + (y_end - y_start) * f64::from(i) / f64::from(blob_size);
            let z_c = z_start + (z_end - z_start) * f64::from(i) / f64::from(blob_size);
            let blob_scale = rand.next_double() * f64::from(blob_size) / 16.0;
            let rad_xz =
                f64::from(MathHelper::sin(i as f32 * JavaMath::PI_FLOAT / blob_size as f32) + 1.0) * blob_scale + 1.0;
            let rad_y =
                f64::from(MathHelper::sin(i as f32 * JavaMath::PI_FLOAT / blob_size as f32) + 1.0) * blob_scale + 1.0;
            let min_x = MathHelper::floor_double(x_c - rad_xz / 2.0);
            let max_x = MathHelper::floor_double(x_c + rad_xz / 2.0);
            let min_y = MathHelper::floor_double(y_c - rad_y / 2.0);
            let max_y = MathHelper::floor_double(y_c + rad_y / 2.0);
            let min_z = MathHelper::floor_double(z_c - rad_xz / 2.0);
            let max_z = MathHelper::floor_double(z_c + rad_xz / 2.0);
            for x in min_x..=max_x {
                let dx = (f64::from(x) + 0.5 - x_c) / (rad_xz / 2.0);
                if dx * dx >= 1.0 {
                    continue;
                }
                for y in min_y..=max_y {
                    let dy = (f64::from(y) + 0.5 - y_c) / (rad_y / 2.0);
                    if dx * dx + dy * dy >= 1.0 {
                        continue;
                    }
                    for z in min_z..=max_z {
                        let dz = (f64::from(z) + 0.5 - z_c) / (rad_xz / 2.0);
                        if dx * dx + dy * dy + dz * dz < 1.0 && world.get_block_id(Int3::new(x, y, z)) == BLOCK_STONE {
                            world.set_block(Int3::new(x, y, z), self.r#type, 0);
                        }
                    }
                }
            }
        }
        true
    }

    //  Attempts to generate flower/mushroom patches
    pub fn generate_flowers(&self, world: &mut WorldWrapper, rand: &mut Random, pos: Int3) -> bool {
        let is_mushroom = self.r#type == BLOCK_MUSHROOM_BROWN || self.r#type == BLOCK_MUSHROOM_RED;

        for _i in 0..64 {
            let x = pos.x + rand.next_int_bound(8) - rand.next_int_bound(8);
            let y = pos.y + rand.next_int_bound(4) - rand.next_int_bound(4);
            let z = pos.z + rand.next_int_bound(8) - rand.next_int_bound(8);
            if y < 0 || y >= crate::constants::CHUNK_HEIGHT {
                continue;
            }
            if world.get_block_id(Int3::new(x, y, z)) != BLOCK_AIR {
                continue;
            }

            if is_mushroom {
                if is_solid(world.get_block_id(Int3::new(x, y - 1, z))) && world.get_sky_light(Int3::new(x, y, z)) == 0 {
                    world.set_block(Int3::new(x, y, z), self.r#type, 0);
                }
            } else {
                let below = world.get_block_id(Int3::new(x, y - 1, z));
                if below == BLOCK_GRASS {
                    world.set_block(Int3::new(x, y, z), self.r#type, 0);
                }
            }
        }
        true
    }

    //  Attempts to generate tallgrass patches
    pub fn generate_tallgrass(&self, world: &mut WorldWrapper, rand: &mut Random, mut pos: Int3) -> bool {
        while pos.y > 0 {
            let b = world.get_block_id(Int3::new(pos.x, pos.y, pos.z));
            if b != BLOCK_AIR && b != BLOCK_LEAVES {
                break;
            }
            pos.y -= 1;
        }

        for _i in 0..128 {
            let x = pos.x + rand.next_int_bound(8) - rand.next_int_bound(8);
            let y = pos.y + rand.next_int_bound(4) - rand.next_int_bound(4);
            let z = pos.z + rand.next_int_bound(8) - rand.next_int_bound(8);
            if y < 0 || y >= crate::constants::CHUNK_HEIGHT {
                continue;
            }
            if world.get_block_id(Int3::new(x, y, z)) != BLOCK_AIR {
                continue;
            }
            let below = world.get_block_id(Int3::new(x, y - 1, z));
            if below == BLOCK_GRASS || below == BLOCK_DIRT {
                world.set_block(Int3::new(x, y, z), self.r#type, self.meta as u8);
            }
        }
        true
    }

    //  Attempts to generate deadbush patches
    pub fn generate_deadbush(&self, world: &mut WorldWrapper, rand: &mut Random, mut pos: Int3) -> bool {
        while pos.y > 0 {
            let b = world.get_block_id(Int3::new(pos.x, pos.y, pos.z));
            if b != BLOCK_AIR && b != BLOCK_LEAVES {
                break;
            }
            pos.y -= 1;
        }

        for _i in 0..4 {
            let x = pos.x + rand.next_int_bound(8) - rand.next_int_bound(8);
            let y = pos.y + rand.next_int_bound(4) - rand.next_int_bound(4);
            let z = pos.z + rand.next_int_bound(8) - rand.next_int_bound(8);
            if y < 0 || y >= crate::constants::CHUNK_HEIGHT {
                continue;
            }
            if world.get_block_id(Int3::new(x, y, z)) == BLOCK_AIR && world.get_block_id(Int3::new(x, y - 1, z)) == BLOCK_SAND
            {
                world.set_block(Int3::new(x, y, z), BLOCK_DEADBUSH, 0);
            }
        }
        true
    }

    //  Attempts to generate sugarcane patches
    pub fn generate_sugarcane(&self, world: &mut WorldWrapper, rand: &mut Random, pos: Int3) -> bool {
        fn is_water(world: &WorldWrapper, wx: i32, wy: i32, wz: i32) -> bool {
            let b = world.get_block_id(Int3::new(wx, wy, wz));
            b == BLOCK_WATER_STILL || b == BLOCK_WATER_FLOWING
        }

        for _i in 0..20 {
            let x = pos.x + rand.next_int_bound(4) - rand.next_int_bound(4);
            let y = pos.y; // Y is fixed across all attempts
            let z = pos.z + rand.next_int_bound(4) - rand.next_int_bound(4);
            if world.get_block_id(Int3::new(x, y, z)) != BLOCK_AIR {
                continue;
            }

            if !is_water(world, x - 1, y - 1, z)
                && !is_water(world, x + 1, y - 1, z)
                && !is_water(world, x, y - 1, z - 1)
                && !is_water(world, x, y - 1, z + 1)
            {
                continue;
            }

            let inner = rand.next_int_bound(3);
            let height = 2 + rand.next_int_bound(inner + 1);
            for h in 0..height {
                let below = world.get_block_id(Int3::new(x, y + h - 1, z));
                if below != BLOCK_GRASS && below != BLOCK_DIRT && below != BLOCK_SUGARCANE {
                    break;
                }
                if world.get_block_id(Int3::new(x, y + h, z)) != BLOCK_AIR {
                    break;
                }
                world.set_block(Int3::new(x, y + h, z), BLOCK_SUGARCANE, 0);
            }
        }
        true
    }

    //  Attempts to generate pumpkin patches
    pub fn generate_pumpkins(&self, world: &mut WorldWrapper, rand: &mut Random, pos: Int3) -> bool {
        for _i in 0..64 {
            let x = pos.x + rand.next_int_bound(8) - rand.next_int_bound(8);
            let y = pos.y + rand.next_int_bound(4) - rand.next_int_bound(4);
            let z = pos.z + rand.next_int_bound(8) - rand.next_int_bound(8);
            if y < 0 || y >= crate::constants::CHUNK_HEIGHT {
                continue;
            }
            if world.get_block_id(Int3::new(x, y, z)) != BLOCK_AIR {
                continue;
            }
            if world.get_block_id(Int3::new(x, y - 1, z)) != BLOCK_GRASS {
                continue;
            }
            // canPlaceBlockAt: no adjacent pumpkins on cardinal sides
            if world.get_block_id(Int3::new(x - 1, y, z)) == BLOCK_PUMPKIN {
                continue;
            }
            if world.get_block_id(Int3::new(x + 1, y, z)) == BLOCK_PUMPKIN {
                continue;
            }
            if world.get_block_id(Int3::new(x, y, z - 1)) == BLOCK_PUMPKIN {
                continue;
            }
            if world.get_block_id(Int3::new(x, y, z + 1)) == BLOCK_PUMPKIN {
                continue;
            }
            world.set_block(Int3::new(x, y, z), BLOCK_PUMPKIN, rand.next_int_bound(4) as u8);
        }
        true
    }

    //  Attempts to generate cacti patches
    pub fn generate_cacti(&self, world: &mut WorldWrapper, rand: &mut Random, pos: Int3) -> bool {
        for _i in 0..10 {
            let x = pos.x + rand.next_int_bound(8) - rand.next_int_bound(8);
            let y = pos.y + rand.next_int_bound(4) - rand.next_int_bound(4);
            let z = pos.z + rand.next_int_bound(8) - rand.next_int_bound(8);
            if world.get_block_id(Int3::new(x, y, z)) != BLOCK_AIR {
                continue;
            }

            let inner = rand.next_int_bound(3);
            let height = 1 + rand.next_int_bound(inner + 1);
            for h in 0..height {
                if block_properties::block_properties()[world.get_block_id(Int3::new(x - 1, y + h, z)).0 as u8 as usize]
                    .material
                    .is_solid
                {
                    continue;
                }
                if block_properties::block_properties()[world.get_block_id(Int3::new(x + 1, y + h, z)).0 as u8 as usize]
                    .material
                    .is_solid
                {
                    continue;
                }
                if block_properties::block_properties()[world.get_block_id(Int3::new(x, y + h, z - 1)).0 as u8 as usize]
                    .material
                    .is_solid
                {
                    continue;
                }
                if block_properties::block_properties()[world.get_block_id(Int3::new(x, y + h, z + 1)).0 as u8 as usize]
                    .material
                    .is_solid
                {
                    continue;
                }
                let below = world.get_block_id(Int3::new(x, y + h - 1, z));
                if below == BLOCK_SAND || below == BLOCK_CACTUS {
                    world.set_block(Int3::new(x, y + h, z), BLOCK_CACTUS, 0);
                }
            }
        }
        true
    }

    //  Attempts to generate a singular liquid source block
    pub fn generate_liquid(&self, world: &mut WorldWrapper, _rand: &mut Random, pos: Int3) -> bool {
        if world.get_block_id(Int3::new(pos.x, pos.y + 1, pos.z)) != BLOCK_STONE {
            return false;
        }
        if world.get_block_id(Int3::new(pos.x, pos.y - 1, pos.z)) != BLOCK_STONE {
            return false;
        }
        let cur = world.get_block_id(pos);
        if cur != BLOCK_AIR && cur != BLOCK_STONE {
            return false;
        }

        let mut stone = 0;
        let mut air = 0;
        if world.get_block_id(Int3::new(pos.x - 1, pos.y, pos.z)) == BLOCK_STONE {
            stone += 1;
        }
        if world.get_block_id(Int3::new(pos.x + 1, pos.y, pos.z)) == BLOCK_STONE {
            stone += 1;
        }
        if world.get_block_id(Int3::new(pos.x, pos.y, pos.z - 1)) == BLOCK_STONE {
            stone += 1;
        }
        if world.get_block_id(Int3::new(pos.x, pos.y, pos.z + 1)) == BLOCK_STONE {
            stone += 1;
        }
        if world.get_block_id(Int3::new(pos.x - 1, pos.y, pos.z)) == BLOCK_AIR {
            air += 1;
        }
        if world.get_block_id(Int3::new(pos.x + 1, pos.y, pos.z)) == BLOCK_AIR {
            air += 1;
        }
        if world.get_block_id(Int3::new(pos.x, pos.y, pos.z - 1)) == BLOCK_AIR {
            air += 1;
        }
        if world.get_block_id(Int3::new(pos.x, pos.y, pos.z + 1)) == BLOCK_AIR {
            air += 1;
        }

        if stone == 3 && air == 1 {
            world.set_block(pos, self.r#type, 0);
        }
        true
    }

    // Nether Features

    // TODO: Merge with GenerateLiquid?
    //  GenerateNetherLiquid
    pub fn generate_nether_liquid(&self, world: &mut WorldWrapper, _rand: &mut Random, pos: Int3) -> bool {
        if world.get_block_id(Int3::new(pos.x, pos.y + 1, pos.z)) != BLOCK_NETHERRACK {
            return false;
        }
        if world.get_block_id(Int3::new(pos.x, pos.y - 1, pos.z)) != BLOCK_NETHERRACK {
            return false;
        }
        let cur = world.get_block_id(pos);
        if cur != BLOCK_AIR && cur != BLOCK_NETHERRACK {
            return false;
        }

        let mut netherrack = 0;
        let mut air = 0;
        if world.get_block_id(Int3::new(pos.x - 1, pos.y, pos.z)) == BLOCK_NETHERRACK {
            netherrack += 1;
        }
        if world.get_block_id(Int3::new(pos.x + 1, pos.y, pos.z)) == BLOCK_NETHERRACK {
            netherrack += 1;
        }
        if world.get_block_id(Int3::new(pos.x, pos.y, pos.z - 1)) == BLOCK_NETHERRACK {
            netherrack += 1;
        }
        if world.get_block_id(Int3::new(pos.x, pos.y, pos.z + 1)) == BLOCK_NETHERRACK {
            netherrack += 1;
        }
        if world.get_block_id(Int3::new(pos.x - 1, pos.y, pos.z)) == BLOCK_AIR {
            air += 1;
        }
        if world.get_block_id(Int3::new(pos.x + 1, pos.y, pos.z)) == BLOCK_AIR {
            air += 1;
        }
        if world.get_block_id(Int3::new(pos.x, pos.y, pos.z - 1)) == BLOCK_AIR {
            air += 1;
        }
        if world.get_block_id(Int3::new(pos.x, pos.y, pos.z + 1)) == BLOCK_AIR {
            air += 1;
        }

        if netherrack == 3 && air == 1 {
            world.set_block(pos, self.r#type, 0);
        }
        true
    }

    //  GenerateNetherFire
    pub fn generate_nether_fire(&self, world: &mut WorldWrapper, rand: &mut Random, pos: Int3) -> bool {
        for _i in 0..64 {
            let test_pos = Int3::new(
                pos.x + rand.next_int_bound(8) - rand.next_int_bound(8),
                pos.y + rand.next_int_bound(4) - rand.next_int_bound(4),
                pos.z + rand.next_int_bound(8) - rand.next_int_bound(8),
            );
            // If air with netherrack underneath, generate
            if world.get_block_id(test_pos) == BLOCK_AIR
                && world.get_block_id(test_pos + Int3::new(0, -1, 0)) == BLOCK_NETHERRACK
            {
                world.set_block(test_pos, BLOCK_FIRE, 0);
            }
        }
        true
    }

    //  GenerateNetherGlowstone
    pub fn generate_nether_glowstone(&self, world: &mut WorldWrapper, rand: &mut Random, pos: Int3) -> bool {
        // Exit if tested block isn't air
        if world.get_block_id(pos) != BLOCK_AIR {
            return false;
        }
        // Exit if block above tested block isn't netherrack
        if world.get_block_id(pos + Int3::new(0, 1, 0)) != BLOCK_NETHERRACK {
            return false;
        }
        world.set_block(pos, BLOCK_GLOWSTONE, 0);
        for _i in 0..1500 {
            let test_pos = Int3::new(
                pos.x + rand.next_int_bound(8) - rand.next_int_bound(8),
                pos.y - rand.next_int_bound(12),
                pos.z + rand.next_int_bound(8) - rand.next_int_bound(8),
            );
            // Skip non-air blocks
            if world.get_block_id(test_pos) != BLOCK_AIR {
                continue;
            }
            let mut adjacent_glowstone_count = 0;
            // Check for adjacent glowstone blocks
            for direction in 0..6 {
                let adjacent_block = match direction {
                    0 => world.get_block_id(test_pos + Int3::new(-1, 0, 0)),
                    1 => world.get_block_id(test_pos + Int3::new(1, 0, 0)),
                    2 => world.get_block_id(test_pos + Int3::new(0, -1, 0)),
                    3 => world.get_block_id(test_pos + Int3::new(0, 1, 0)),
                    4 => world.get_block_id(test_pos + Int3::new(0, 0, -1)),
                    5 => world.get_block_id(test_pos + Int3::new(0, 0, 1)),
                    _ => BLOCK_AIR,
                };
                if adjacent_block == BLOCK_GLOWSTONE {
                    adjacent_glowstone_count += 1;
                }
            }
            // If onle one adjacent glowstone exists, place another
            if adjacent_glowstone_count == 1 {
                world.set_block(test_pos, BLOCK_GLOWSTONE, 0);
            }
        }
        true
    }
}

impl Default for FeatureGenerator {
    fn default() -> Self {
        Self { r#type: BLOCK_AIR, meta: 0 }
    }
}
