/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

// The world manager acts like a wrapper around the chunk manager and lighting manager.
// It handles all world-related operations and provides a simple interface for the rest of the code to interact with the world.
// WorldManager.h
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::base_structs::Block;
use crate::base_types::TickTime;
use crate::blocks::block_properties;
use crate::blocks::materials::Material;
use crate::enums::biomes::Biome;
use crate::enums::blocks::{BLOCK_AIR, BLOCK_INVALID, BLOCK_SAND, BlockType};
use crate::enums::dimensions::Dimension;
use crate::entities::entity::Entity;
use crate::entities::entity_manager::EntityManager;
use crate::helpers::aabb::AABB;
use crate::helpers::cross_platform::Math;
use crate::helpers::java::java_math::{MathHelper, double_to_int32, hash_code};
use crate::helpers::java::java_random::Random;
use crate::helpers::thread_pool::ThreadPool;
use crate::logger::logger::global_logger;
use crate::numeric_structs::{Int2, Int3, Int32_2, VEC3_ZERO};
use crate::tile_entities::tile_entity::TileEntityBehavior;
use crate::tile_entities::tile_entity_manager::TileEntityManager;
use crate::world::chunk::{Chunk, ChunkState};
use crate::world::client_pos::ClientPosition;
use crate::world::generator::generator::GeneratorBehavior;
use crate::world::generator::nether::chunk_gen::NetherGenerator;
use crate::world::generator::overworld::biome_gen::BiomeGenerator;
use crate::world::generator::overworld::chunk_gen::OverworldGenerator;
use crate::world::generator::shared::feature_gen::WorldWrapper;
use crate::world::lighter::{Lighter, LightType};
use crate::world::storage::region_manager::RegionManager;

pub struct PendingBlock {
    pub block: Block,
    pub block_pos: Int3,
    pub light: Int2, // block light, sky light
}

impl Default for PendingBlock {
    fn default() -> Self {
        Self { block: Block::default(), block_pos: Int3::new(0, 0, 0), light: Int2::new(0, 15) }
    }
}

pub struct WorldManager {
    pub region_manager: Option<Arc<Mutex<RegionManager>>>,
    pub chunks: HashMap<Int32_2, Arc<Mutex<Chunk>>>,
    pub on_block_update: Option<Box<dyn FnMut(PendingBlock, Int32_2) + Send>>,

    pub pending_bleed_writes: HashMap<Int32_2, Vec<(Int3, Block)>>,

    pub gen_done_queue: Arc<Mutex<VecDeque<Arc<Mutex<Chunk>>>>>,

    pub light_manager: Lighter,
    pub tile_entity_manager: TileEntityManager,
    pub entity_manager: EntityManager,

    pub pool: ThreadPool,
    pub population_pool: ThreadPool, // unused

    pub seed: i64,
    pub elapsed_ticks: TickTime,

    pub spawn_point: Int3,

    pub this_dimension: Dimension,

    pub rand: Random,

    is_hell: bool, // for the nether
}

impl WorldManager {
    // I believe the vanilla default is
    const VIEW_RADIUS: i32 = 12;
    const SIMULATION_RADIUS: i32 = 9;

    pub fn new(is_hell: bool) -> Self {
        let mut world = Self {
            region_manager: None,
            chunks: HashMap::new(),
            on_block_update: None,
            pending_bleed_writes: HashMap::new(),
            gen_done_queue: Arc::new(Mutex::new(VecDeque::new())),
            light_manager: Lighter::default(),
            tile_entity_manager: TileEntityManager::new(),
            entity_manager: EntityManager::new(),
            pool: ThreadPool::new(2),
            population_pool: ThreadPool::new(1),
            seed: 0,
            elapsed_ticks: 0,
            spawn_point: Int3::new(0, 0, 0),
            this_dimension: Dimension::Overworld,
            rand: Random::new(),
            is_hell,
        };
        if world.is_hell {
            world.this_dimension = Dimension::Nether;
        }
        world
    }

    pub fn init_world_seed(&mut self, seed: i64) {
        self.seed = seed;
    }

    pub fn init_world_seed_str(&mut self, seed: &str) {
        self.seed = i64::from(hash_code(seed));
    }

    pub fn tick(&mut self, players: &[ClientPosition]) {
        self.elapsed_ticks += 1;
        self.drain_gen_queue(); // process generation results first
        self.drain_load_queue(); // integrate finished loads

        // Queue any modified chunks for saving
        let region_manager = match self.region_manager.clone() {
            Some(region_manager) => region_manager,
            None => {
                global_logger().error("No region manager while trying to tick!\n");
                return;
            }
        };
        if self.elapsed_ticks % 40 == 0 {
            // Save periodically
            for chunk in self.chunks.values() {
                let is_modified = chunk.lock().unwrap().is_modified;
                if !is_modified {
                    continue;
                }
                let s = chunk.lock().unwrap().state_load();
                if s < ChunkState::Generated {
                    continue;
                }
                if s == ChunkState::Generating || s == ChunkState::Loading {
                    continue;
                }
                let cpos = chunk.lock().unwrap().cpos;
                let entities = self.entity_manager.collect_entities_for_save(cpos, false);
                region_manager.lock().unwrap().save_chunk(Arc::clone(chunk), entities, self.elapsed_ticks);
                chunk.lock().unwrap().is_modified = false;
            }
        }
        // Save entities in a chunk every 30 seconds
        if self.elapsed_ticks % 600 == 0 {
            for (pos, chunk) in self.chunks.iter() {
                let s = chunk.lock().unwrap().state_load();
                if s < ChunkState::Generated {
                    continue;
                }
                if s == ChunkState::Generating || s == ChunkState::Loading {
                    continue;
                }
                if self.entity_manager.chunk_has_entities(*pos) {
                    let entities = self.entity_manager.collect_entities_for_save(*pos, false);
                    region_manager.lock().unwrap().save_chunk(Arc::clone(chunk), entities, self.elapsed_ticks);
                }
                chunk.lock().unwrap().is_modified = false;
            }
        }
        region_manager.lock().unwrap().pump_pipeline();

        self.update_load_radius(players);
        self.populate_ready(); // population runs on main thread
        let mut light_manager = std::mem::take(&mut self.light_manager);
        light_manager.process_light_queue(self, i32::MAX);
        self.light_manager = light_manager;

        // Update our entities
        self.entity_manager.tick();
    }

    pub fn update(&mut self, players: &[ClientPosition]) {
        self.pump_pipeline(players);
    }

    pub fn shutdown(&mut self) {
        let region_manager = match self.region_manager.clone() {
            Some(region_manager) => region_manager,
            None => return,
        };
        if self.is_hell {
            global_logger().info("Saving chunks for level -1\n");
        } else {
            global_logger().info("Saving chunks for level 0\n");
        }

        // Save all currently loaded modified chunks
        let positions: Vec<Int32_2> = self.chunks.keys().copied().collect();
        for pos in &positions {
            let chunk = match self.chunks.get(pos) {
                Some(chunk) => Arc::clone(chunk),
                None => continue,
            };
            let is_modified = chunk.lock().unwrap().is_modified;
            if !is_modified && !self.entity_manager.chunk_has_entities(*pos) {
                continue;
            }
            let s = chunk.lock().unwrap().state_load();
            if s < ChunkState::Generated {
                continue;
            }
            if s == ChunkState::Generating || s == ChunkState::Loading {
                continue;
            }
            let entities = self.entity_manager.collect_entities_for_save(*pos, true);
            region_manager.lock().unwrap().save_chunk(Arc::clone(&chunk), entities, self.elapsed_ticks);
            chunk.lock().unwrap().is_modified = false;
        }

        // For every position that still has pending bleed writes, forceload or forcegenerate the chunk, apply the writes, then save it.
        while !self.pending_bleed_writes.is_empty() {
            let cpos = *self.pending_bleed_writes.keys().next().unwrap();

            // Insert a placeholder if not already in the map
            if !self.chunks.contains_key(&cpos) {
                let mut c = Chunk::default();
                c.cpos = cpos;
                self.chunks.insert(cpos, Arc::new(Mutex::new(c)));
            }

            // Wait until the chunk is ready
            loop {
                let state = self.chunks.get(&cpos).unwrap().lock().unwrap().state_load();
                if state >= ChunkState::Generated {
                    break;
                }
                self.pump_pipeline(&[]);
                self.pool.wait();
                self.drain_gen_queue();
                region_manager.lock().unwrap().iopool.wait();
                self.drain_load_queue();
            }

            // Apply the pending writes
            self.flush_bleed_writes();

            // Save it
            let chunk = Arc::clone(self.chunks.get(&cpos).unwrap());
            let is_modified = chunk.lock().unwrap().is_modified;
            if is_modified {
                let entities = self.entity_manager.collect_entities_for_save(cpos, false);
                region_manager.lock().unwrap().save_chunk(Arc::clone(&chunk), entities, self.elapsed_ticks);
                chunk.lock().unwrap().is_modified = false;
            }
        }

        // Flush everything to disk and wait for IO to finish
        region_manager.lock().unwrap().flush_all();
    }

    // Get colliders for an area
    pub fn get_colliding_bounding_boxes(&self, area: AABB) -> Vec<AABB> {
        let mut colliding_boxes = Vec::new();

        let min_x = double_to_int32(area.min_x.floor());
        let max_x = double_to_int32((area.max_x + 1.0).floor());
        let min_y = double_to_int32(area.min_y.floor());
        let max_y = double_to_int32((area.max_y + 1.0).floor());
        let min_z = double_to_int32(area.min_z.floor());
        let max_z = double_to_int32((area.max_z + 1.0).floor());

        // Java iterates Y from var5-1 to var6 (exclusive)
        let start_y = Math::max(0, min_y - 1);
        let end_y = Math::min(127, max_y);

        // Iterate for our potential grid
        for x in min_x..max_x {
            for z in min_z..max_z {
                // Get the chunk once for this X/Z column
                let chunk = self.get_chunk_raw(Int32_2::new(x >> 4, z >> 4));

                // If chunk isn't loaded, Beta 1.7.3 usually treats it as air
                let chunk = match chunk {
                    Some(chunk) => chunk,
                    None => continue,
                };

                // local coords inside the chunk
                let local_x = x & 15;
                let local_z = z & 15;

                for y in start_y..=end_y {
                    let (block_id, block_meta) = {
                        let guard = chunk.lock().unwrap();
                        (guard.get_block(Int3::new(local_x, y, local_z)), guard.get_meta(Int3::new(local_x, y, local_z)))
                    };
                    // Air isn't collidable
                    if block_id == BLOCK_AIR {
                        continue;
                    }
                    if !block_properties::block_properties()[block_id.0 as u8 as usize].is_collidable {
                        continue;
                    }
                    let get_collider = block_properties::block_behaviors()[block_id.0 as u8 as usize].get_collider;
                    let get_collider = match get_collider {
                        Some(f) => f,
                        None => continue,
                    };
                    // Offset local collider to world coordinates
                    let world_collider = get_collider(block_meta).offset(f64::from(x), f64::from(y), f64::from(z));
                    for boxed in world_collider.boxes.iter() {
                        if boxed.intersects(&area) {
                            colliding_boxes.push(*boxed);
                        }
                    }
                }
            }
        }
        colliding_boxes
    }

    pub fn get_view_radius(&self) -> i32 {
        Self::VIEW_RADIUS
    }
    pub fn get_simulation_distance(&self) -> i32 {
        Self::SIMULATION_RADIUS
    }

    pub fn handle_fluid_acceleration(&mut self, collider: AABB, material: Material, entity: &mut Entity) -> bool {
        // Handles the fluid push physics, only counts fluids of the same material
        // Returns whether the entity is in the material
        // This is almost entirely used for water
        let min_x = MathHelper::floor_double(collider.min_x);
        let max_x = MathHelper::floor_double(collider.max_x + 1.0);
        let min_y = MathHelper::floor_double(collider.min_y);
        let max_y = MathHelper::floor_double(collider.max_y + 1.0);
        let min_z = MathHelper::floor_double(collider.min_z);
        let max_z = MathHelper::floor_double(collider.max_z + 1.0);
        if !self.aabb_in_valid_chunks(AABB {
            min_x: f64::from(min_x),
            min_y: f64::from(min_y),
            min_z: f64::from(min_z),
            max_x: f64::from(max_x),
            max_y: f64::from(max_y),
            max_z: f64::from(max_z),
        }) {
            return false;
        }

        let mut in_material = false;
        let mut push_vector = VEC3_ZERO;

        // Check every block within the collider
        // We are looking to see if the materials match
        for x in min_x..max_x {
            for y in min_y..max_y {
                for z in min_z..max_z {
                    let block_id = self.get_block_id(Int3::new(x, y, z));
                    let block = block_properties::block_properties()[block_id.0 as u8 as usize];
                    if block.material == material {
                        let fluid_height =
                            f64::from((y + 1) as f32 - block_properties::get_fluid_percent_air(self.get_metadata(Int3::new(x, y, z))));
                        if f64::from(max_y) >= fluid_height {
                            // We are definitely in this material
                            // Lets get how this material contributes to our flow vector
                            in_material = true;
                            let velocity_function = block_properties::block_behaviors()[block_id.0 as u8 as usize].velocity_to_add_to_entity;
                            if let Some(velocity_function) = velocity_function {
                                velocity_function(self, Int3::new(x, y, z), &mut push_vector);
                            }
                        }
                    }
                }
            }
        }

        // Normalize the vector
        let magnitude = (push_vector.x * push_vector.x + push_vector.y * push_vector.y + push_vector.z * push_vector.z).sqrt();
        if magnitude > 0.0 {
            push_vector.x /= magnitude;
            push_vector.y /= magnitude;
            push_vector.z /= magnitude;

            // Apply the vector
            let push_force = 0.014;
            entity.motion_x += push_vector.x * push_force;
            entity.motion_y += push_vector.y * push_force;
            entity.motion_z += push_vector.z * push_force;
        }

        in_material
    }

    pub fn is_material_in_aabb(&self, collider: AABB, material: Material) -> bool {
        let min_x = MathHelper::floor_double(collider.min_x);
        let max_x = MathHelper::floor_double(collider.max_x + 1.0);
        let min_y = MathHelper::floor_double(collider.min_y);
        let max_y = MathHelper::floor_double(collider.max_y + 1.0);
        let min_z = MathHelper::floor_double(collider.min_z);
        let max_z = MathHelper::floor_double(collider.max_z + 1.0);

        // Check every block within the collider
        // We are looking to see if the materials match
        for x in min_x..max_x {
            for y in min_y..max_y {
                for z in min_z..max_z {
                    let block_id = self.get_block_id(Int3::new(x, y, z));
                    let block = block_properties::block_properties()[block_id.0 as u8 as usize];
                    if block.material == material {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn update_load_radius(&mut self, players: &[ClientPosition]) {
        let mut wanted: HashSet<Int32_2> = HashSet::new();
        for player in players {
            let center = player.get_chunk_pos();
            let view_dist = if player.view_distance_override != 0 { player.view_distance_override } else { Self::VIEW_RADIUS };
            for dx in -view_dist..=view_dist {
                for dz in -view_dist..=view_dist {
                    wanted.insert(Int32_2::new(center.x + dx, *center.z() + dz));
                }
            }
        }

        // Get chunks we want
        for pos in &wanted {
            if !self.chunks.contains_key(pos) {
                let mut c = Chunk::default();
                c.cpos = *pos;
                self.chunks.insert(*pos, Arc::new(Mutex::new(c)));
            }
        }

        // Remove chunks we don't want
        let positions: Vec<Int32_2> = self.chunks.keys().copied().collect();
        for pos in positions {
            if wanted.contains(&pos) {
                continue;
            }
            let chunk = match self.chunks.get(&pos) {
                Some(chunk) => Arc::clone(chunk),
                None => continue,
            };
            if chunk.lock().unwrap().spawn_chunk {
                continue;
            }
            let s = chunk.lock().unwrap().state_load();
            if s == ChunkState::Generating || s == ChunkState::Loading {
                continue;
            }

            // This chunk is actually leaving simulation so force unload entities
            let cpos = chunk.lock().unwrap().cpos;
            let is_modified = chunk.lock().unwrap().is_modified;
            if is_modified || self.entity_manager.chunk_has_entities(cpos) {
                if let Some(region_manager) = self.region_manager.clone() {
                    let entities = self.entity_manager.collect_entities_for_save(cpos, true);
                    region_manager.lock().unwrap().save_chunk(Arc::clone(&chunk), entities, self.elapsed_ticks);
                }
                chunk.lock().unwrap().is_modified = false;
            }

            self.chunks.remove(&pos);
        }
    }

    pub fn pump_pipeline(&mut self, players: &[ClientPosition]) {
        // Take a snapshot of all the current chunk positions so we don't have to worry about threads
        // This is technically a relic from when we had chunks put themselves into the world's chunk map but now the world does it all at the end of the tick
        // Still good practice, though
        let snapshot: Vec<Int32_2> = self.chunks.keys().copied().collect();

        let player_count = players.len();
        let slice_per_player = 16usize;

        let mut no_player_candidates: Vec<Int32_2> = Vec::new();
        let mut per_player_queues: Vec<Vec<Int32_2>> = Vec::new();
        if player_count == 0 {
            // No players so try and get every chunk within load distance if its not already generating
            for p in &snapshot {
                let chunk = match self.chunks.get(p) {
                    Some(chunk) => chunk,
                    None => continue,
                };
                if chunk.lock().unwrap().state_load() != ChunkState::Unloaded {
                    continue;
                }
                no_player_candidates.push(*p);
            }
            no_player_candidates.sort_by(|a, b| a.x.cmp(&b.x).then_with(|| a.z().cmp(b.z())));
        } else {
            per_player_queues.reserve(player_count);
            for _player in players {
                let mut candidates: Vec<Int32_2> = Vec::new();
                for p in &snapshot {
                    let chunk = match self.chunks.get(p) {
                        Some(chunk) => chunk,
                        None => continue,
                    };
                    if chunk.lock().unwrap().state_load() != ChunkState::Unloaded {
                        continue;
                    }
                    candidates.push(*p);
                }
                // Sort by load order that beta 1.7.3 seems to use
                candidates.sort_by(|a, b| a.x.cmp(&b.x).then_with(|| a.z().cmp(b.z())));
                per_player_queues.push(candidates);
            }
        }

        let mut started_this_tick: HashSet<Int32_2> = HashSet::new();

        if player_count == 0 {
            let mut started = 0;
            for pos in no_player_candidates {
                if started >= slice_per_player {
                    break;
                }
                let chunk_exists = match self.region_manager.clone() {
                    Some(region_manager) => region_manager.lock().unwrap().chunk_exists(pos),
                    None => false,
                };
                if chunk_exists {
                    if self.start_loading(pos, &mut started_this_tick) {
                        started += 1;
                    }
                    continue;
                }
                if self.start_generation(pos, &mut started_this_tick) {
                    started += 1;
                }
            }
        } else {
            // Make sure everyone gets their share of the budget
            let mut cursors = vec![0usize; player_count];
            let mut total_started = 0usize;
            let total_budget = slice_per_player * player_count;
            let mut any_progress = true;
            while total_started < total_budget && any_progress {
                any_progress = false;
                for i in 0..player_count {
                    if total_started >= total_budget {
                        break;
                    }
                    let mut player_consumed = 0usize;
                    while player_consumed < slice_per_player && cursors[i] < per_player_queues[i].len() {
                        let cpos = per_player_queues[i][cursors[i]];
                        cursors[i] += 1;
                        let chunk_exists = match self.region_manager.clone() {
                            Some(region_manager) => region_manager.lock().unwrap().chunk_exists(cpos),
                            None => false,
                        };
                        if chunk_exists {
                            if self.start_loading(cpos, &mut started_this_tick) {
                                player_consumed += 1;
                                total_started += 1;
                                any_progress = true;
                            }
                            continue;
                        }
                        if self.start_generation(cpos, &mut started_this_tick) {
                            player_consumed += 1;
                            total_started += 1;
                            any_progress = true;
                        }
                    }
                }
            }
        }
    }

    pub fn populate_ready(&mut self) {
        // Try and match beta's population order its finicky lol
        let mut ordered: Vec<Int32_2> = Vec::new();
        for (pos, chunk) in self.chunks.iter() {
            if chunk.lock().unwrap().is_terrain_populated {
                continue;
            }
            // Only consider chunks that could possibly be ready
            // This excludes border chunks on the positive X and Z axes since their population needs neighbors that can't exist
            if !self.chunks.contains_key(&Int32_2::new(pos.x + 1, *pos.z()))
                || !self.chunks.contains_key(&Int32_2::new(pos.x, *pos.z() + 1))
                || !self.chunks.contains_key(&Int32_2::new(pos.x + 1, *pos.z() + 1))
            {
                continue;
            }
            ordered.push(*pos);
        }

        // Sort by population order that beta 1.7.3 seems to use
        ordered.sort_by(|a, b| a.x.cmp(&b.x).then_with(|| a.z().cmp(b.z())));

        // Make sure we don't try to populate the same chunk multiple times in one tick (can happen with the weird population order and multiple players)
        // Also make sure we populate in the right order!
        // We break if the target chunk isn't ready yet so population order is guaranteed
        let mut populated_this_tick: HashSet<Int32_2> = HashSet::new();
        for pos in ordered {
            if !self.can_populate_direct(pos) {
                break;
            }
            if populated_this_tick.contains(&pos) {
                continue;
            }
            let chunk = match self.chunks.get(&pos) {
                Some(chunk) => Arc::clone(chunk),
                None => break,
            };
            chunk.lock().unwrap().state_store(ChunkState::Populating);
            let seed = self.seed;
            let is_hell = self.is_hell;
            let mut wrapper = WorldWrapper::new(self, pos);
            wrapper.get_chunk_region();
            if is_hell {
                let mut generator = NetherGenerator::new(seed);
                generator.populate_chunk(pos, &mut wrapper);
            } else {
                let mut generator = OverworldGenerator::new(seed);
                generator.populate_chunk(pos, &mut wrapper);
            }
            chunk.lock().unwrap().is_terrain_populated = true;
            chunk.lock().unwrap().is_modified = true;
            chunk.lock().unwrap().state_store(ChunkState::Populated);
            populated_this_tick.insert(pos);
            wrapper.free_chunk_region();
            self.flush_bleed_writes();
        }
    }

    pub fn drain_load_queue(&mut self) {
        let positions: Vec<Int32_2> = self
            .chunks
            .iter()
            .filter(|(_, chunk)| chunk.lock().unwrap().state_load() == ChunkState::Loading)
            .map(|(pos, _)| *pos)
            .collect();

        for pos in positions {
            let region_manager = match self.region_manager.clone() {
                Some(region_manager) => region_manager,
                None => continue,
            };
            let loaded = region_manager.lock().unwrap().get_chunk(pos);
            let loaded = match loaded {
                Some(loaded) => loaded,
                None => continue,
            };

            let existing = match self.chunks.get(&pos) {
                Some(existing) => Arc::clone(existing),
                None => continue,
            };
            let was_spawn_chunk = existing.lock().unwrap().spawn_chunk;
            loaded.lock().unwrap().spawn_chunk = was_spawn_chunk;
            self.chunks.insert(pos, Arc::clone(&loaded));

            // Regenerate temp and humidity data
            let mut biome_gen = BiomeGenerator::new(self.seed);
            let mut temp: Vec<f64> = Vec::new();
            let mut humi: Vec<f64> = Vec::new();
            let mut weird: Vec<f64> = Vec::new();
            let mut ignored = [Biome::None; crate::constants::CHUNK_AREA as usize];
            biome_gen.generate_biome_map(&mut ignored, &mut temp, &mut humi, &mut weird, Int2::new(pos.x * crate::constants::CHUNK_WIDTH, *pos.z() * crate::constants::CHUNK_WIDTH));
            {
                let mut loaded_guard = loaded.lock().unwrap();
                for i in 0..crate::constants::CHUNK_AREA as usize {
                    loaded_guard.temperature[i] = temp[i] as f32;
                    loaded_guard.humidity[i] = humi[i] as f32;
                }
            }

            // Replay any writes that arrived while this chunk was loading.
            if let Some(writes) = self.pending_bleed_writes.remove(&pos) {
                for (wpos, block) in writes {
                    self.set_block(wpos, block.r#type, block.data);
                }
            }

            // Register our tile entities
            self.register_chunk_tile_entities(&loaded);

            // Register our entities
            let entity_tags = {
                let mut loaded_guard = loaded.lock().unwrap();
                std::mem::take(&mut loaded_guard.entity_tags)
            };
            for mut entity_tag in entity_tags {
                self.entity_manager.create_entity_from_nbt(&mut entity_tag);
            }
        }
    }

    pub fn notify_neighbors_of_update(&mut self, global_pos: Int3) {
        // Update our six neighbors
        let ndx = [-1, 1, 0, 0];
        let ndz = [0, 0, -1, 1];

        // Notify horizontal neighbors
        for i in 0..4 {
            let dx = ndx[i];
            let dz = ndz[i];
            let new_pos = Int3::new(global_pos.x + dx, global_pos.y, global_pos.z + dz);
            let block = self.get_block_id(new_pos);
            let update_function = block_properties::block_behaviors()[block.0 as u8 as usize].on_neighbor_block_change;
            if let Some(update_function) = update_function {
                update_function(self, new_pos);
            }
        }

        // Vertical neighbors
        for i in 0..2 {
            let dy = ndx[i]; // we are using ndx because the first two items are -1, 1
            let new_pos = Int3::new(global_pos.x, global_pos.y + dy, global_pos.z);
            let block = self.get_block_id(new_pos);
            let update_function = block_properties::block_behaviors()[block.0 as u8 as usize].on_neighbor_block_change;
            if let Some(update_function) = update_function {
                update_function(self, new_pos);
            }
        }
    }

    // For creating a fresh tile entity for generation etc
    pub fn create_tile_entity(&mut self, tile_entity: Arc<Mutex<dyn TileEntityBehavior + Send>>) {
        let (px, pz) = {
            let te = tile_entity.lock().unwrap();
            (te.base().position.x, te.base().position.z)
        };
        let cpos = Int32_2::new(px >> 4, pz >> 4);
        let chunk = match self.get_chunk_raw(cpos) {
            Some(chunk) => chunk,
            None => return,
        };
        tile_entity.lock().unwrap().base_mut().chunk = Arc::downgrade(&chunk);
        self.tile_entity_manager.initialize_tile_entity(&tile_entity); // weak_ptr added if canTick
        chunk.lock().unwrap().tile_entities.push(tile_entity); // chunk takes ownership
    }

    // For registering a tile entity that already exists in the world (e.g. loaded from disk)
    pub fn register_chunk_tile_entities(&mut self, chunk: &Arc<Mutex<Chunk>>) {
        let guard = chunk.lock().unwrap();
        for te in guard.tile_entities.iter() {
            self.tile_entity_manager.initialize_tile_entity(te);
            te.lock().unwrap().base_mut().chunk = Arc::downgrade(chunk);
        }
    }

    // Returns the tile entity at world position `pos`, or nullptr if none.
    pub fn get_tile_entity(&self, pos: Int3) -> Option<Arc<Mutex<dyn TileEntityBehavior + Send>>> {
        let chunk = self.get_chunk_raw(Int32_2::new(pos.x >> 4, pos.z >> 4))?;
        let guard = chunk.lock().unwrap();
        for te in guard.tile_entities.iter() {
            let matches = {
                let locked = te.lock().unwrap();
                let p = locked.base().position;
                p.x == pos.x && p.y == pos.y && p.z == pos.z
            };
            if matches {
                return Some(Arc::clone(te));
            }
        }
        None
    }

    // Returns nullptr if not found or wrong type.
    pub fn get_tile_entity_as<T: TileEntityBehavior + 'static>(&self, pos: Int3) -> Option<Arc<Mutex<dyn TileEntityBehavior + Send>>> {
        self.get_tile_entity(pos)
    }

    pub fn get_tile_entity_shared<T: TileEntityBehavior + 'static>(&self, pos: Int3) -> Option<Arc<Mutex<dyn TileEntityBehavior + Send>>> {
        self.get_tile_entity(pos)
    }

    // Remove the tile entity at world position `pos`.
    pub fn remove_tile_entity(&mut self, pos: Int3) {
        let chunk = match self.get_chunk_raw(Int32_2::new(pos.x >> 4, pos.z >> 4)) {
            Some(chunk) => chunk,
            None => return,
        };
        let mut guard = chunk.lock().unwrap();
        guard.tile_entities.retain(|te| {
            let locked = te.lock().unwrap();
            let p = locked.base().position;
            !(p.x == pos.x && p.y == pos.y && p.z == pos.z)
        });
    }

    // Called from pool gen threads
    pub fn post_gen_result(&self, chunk: Arc<Mutex<Chunk>>) {
        self.gen_done_queue.lock().unwrap().push_back(chunk);
    }

    pub fn drain_gen_queue(&mut self) {
        // Integrate chunks that finished generating
        let ready: VecDeque<Arc<Mutex<Chunk>>> = {
            let mut queue = self.gen_done_queue.lock().unwrap();
            std::mem::take(&mut *queue)
        };
        for c in ready {
            let pos = c.lock().unwrap().cpos;
            if self.chunks.contains_key(&pos) {
                let was_spawn_chunk = self.chunks.get(&pos).unwrap().lock().unwrap().spawn_chunk;
                c.lock().unwrap().spawn_chunk = was_spawn_chunk;
                self.chunks.insert(pos, Arc::clone(&c));

                // Replay any writes that arrived while this chunk was unloaded.
                if let Some(writes) = self.pending_bleed_writes.remove(&pos) {
                    for (wpos, block) in writes {
                        self.set_block(wpos, block.r#type, block.data);
                    }
                }

                c.lock().unwrap().generate_skylight_map(); // Regen our skylight map
                self.seed_chunk_lighting(pos); // Reseed our lighting
            }
        }
    }

    pub fn get_chunk(&self, pos: Int32_2) -> Option<Arc<Mutex<Chunk>>> {
        self.get_chunk_shared(pos)
    }

    pub fn can_populate(&self, pos: Int32_2) -> bool {
        self.can_populate_direct(pos)
    }

    pub fn get_block_id(&self, wpos: Int3) -> BlockType {
        if !Self::in_bounds(wpos.y) {
            return BLOCK_AIR;
        }
        let chunk = match self.get_chunk_raw(Int32_2::new(wpos.x >> 4, wpos.z >> 4)) {
            Some(chunk) => chunk,
            None => return BLOCK_AIR,
        };
        let guard = chunk.lock().unwrap();
        if guard.state_load() < ChunkState::Generated {
            return BLOCK_AIR;
        }
        guard.get_block(Int3::new(wpos.x & 15, wpos.y, wpos.z & 15))
    }

    pub fn get_metadata(&self, wpos: Int3) -> u8 {
        if !Self::in_bounds(wpos.y) {
            return 0;
        }
        let chunk = match self.get_chunk_raw(Int32_2::new(wpos.x >> 4, wpos.z >> 4)) {
            Some(chunk) => chunk,
            None => return 0,
        };
        let guard = chunk.lock().unwrap();
        if guard.state_load() < ChunkState::Generated {
            return 0;
        }
        guard.get_meta(Int3::new(wpos.x & 15, wpos.y, wpos.z & 15))
    }

    pub fn set_block_from(&mut self, wpos: Int3, block: Block) {
        self.set_block(wpos, block.r#type, block.data);
    }

    pub fn set_meta(&mut self, wpos: Int3, metadata: u8) {
        if !Self::in_bounds(wpos.y) {
            return;
        }
        let cp = Int32_2::new(wpos.x >> 4, wpos.z >> 4);
        if !self.is_chunk_valid(cp) {
            return;
        }
        let chunk = self.get_chunk_raw(cp).unwrap();
        let local = Int3::new(wpos.x & 15, wpos.y, wpos.z & 15);
        let (old_meta, block_id, new_block, block_light, sky_light, cpos) = {
            let mut guard = chunk.lock().unwrap();
            let old_meta = guard.get_meta(local);
            let block_id = guard.get_block(local);
            guard.set_meta(local, metadata);
            (old_meta, block_id, guard.get_block(local), guard.get_block_light(local), guard.get_sky_light(local), guard.cpos)
        };

        // Update our neighbors
        self.notify_neighbors_of_update(wpos);

        // Callback for the client and server to know about this block update
        if old_meta != metadata && block_properties::block_properties()[block_id.0 as u8 as usize].notify_self_on_meta_change {
            if let Some(callback) = self.on_block_update.as_mut() {
                callback(
                    PendingBlock {
                        block: Block { r#type: new_block, data: metadata },
                        block_pos: wpos,
                        light: Int2::new(i32::from(block_light), i32::from(sky_light)),
                    },
                    cpos,
                );
            }
        }
    }

    pub fn set_block(&mut self, wpos: Int3, block_type: BlockType, metadata: u8) {
        if !Self::in_bounds(wpos.y) {
            return;
        }
        let cp = Int32_2::new(wpos.x >> 4, wpos.z >> 4);
        if !self.is_chunk_valid(cp) {
            // Target chunk isn't ready; cache the write for replay
            self.pending_bleed_writes.entry(cp).or_default().push((wpos, Block { r#type: block_type, data: metadata }));
            return;
        }
        let chunk = self.get_chunk_raw(cp).unwrap();

        // Remove any tile entities that exist at this spot
        {
            let mut guard = chunk.lock().unwrap();
            guard.tile_entities.retain(|te| {
                let locked = te.lock().unwrap();
                locked.base().position != wpos
            });
        }

        // Unlight before changing the block
        self.light_manager_unlight_at(wpos.x, wpos.y, wpos.z, LightType::Block);
        self.light_manager_unlight_at(wpos.x, wpos.y, wpos.z, LightType::Sky);

        // Get the local coordinates of this block within the chunk and set it
        let lx = wpos.x & 15;
        let lz = wpos.z & 15;
        let local = Int3::new(lx, wpos.y, lz);
        let (old_block, old_meta, old_height) = {
            let mut guard = chunk.lock().unwrap();
            let old_block = guard.get_block(local);
            let old_meta = guard.get_meta(local);
            guard.set_block(local, block_type);
            guard.set_meta(local, metadata);
            let old_height = i32::from(guard.get_height_value(Int2::new(lx, lz)));
            (old_block, old_meta, old_height)
        };

        let y = wpos.y;
        let x = wpos.x;
        let z = wpos.z;

        if block_properties::block_properties()[block_type.0 as u8 as usize].light_opacity != 0 {
            // Placing opaque block; heightmap may rise
            if y >= old_height {
                chunk.lock().unwrap().relight_column(Int2::new(lx, lz));

                // The column below the new top was zeroed out by relightColumn.
                // Notify the BFS that all blocks from y down to oldHeight need updating
                for sy in old_height..=y {
                    self.light_manager_unlight_at(x, sy, z, LightType::Sky);
                }
            }
        } else if y == old_height - 1 {
            // Removing top opaque block; heightmap may fall
            chunk.lock().unwrap().relight_column(Int2::new(lx, lz));
        }

        let new_height = i32::from(chunk.lock().unwrap().get_height_value(Int2::new(lx, lz)));
        if new_height < old_height {
            for sy in new_height..old_height {
                self.light_manager.schedule_light_update(Int3::new(x, sy, z), LightType::Sky);
            }
        }

        // Always re-evaluate the edited block and its 4 horizontal neighbours
        self.light_manager.schedule_light_update(Int3::new(x, y, z), LightType::Sky);
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
            self.light_manager.schedule_light_region(Int3::new(nx, min_y, nz), Int3::new(nx, max_y, nz), LightType::Sky);
        }
        // Schedule a block light update for the position itself
        self.light_manager.schedule_light_update(Int3::new(x, y, z), LightType::Block);

        // Update our neighbors
        self.notify_neighbors_of_update(wpos);

        if block_type == BLOCK_AIR {
            // We removed this block effectively
            let function = block_properties::block_behaviors()[old_block.0 as u8 as usize].on_block_removal;
            if let Some(function) = function {
                function(self, wpos);
            }
        } else {
            // Java has this functionality in the chunk setters themselves, but
            // in my opinion (Aidan here) that is stupid and redundant
            let function = block_properties::block_behaviors()[block_type.0 as u8 as usize].on_block_added;
            if let Some(function) = function {
                function(self, wpos);
            }
        }

        // Callback for the client and server to know about this block update
        let should_notify = old_block != block_type
            || (old_meta != metadata && block_properties::block_properties()[block_type.0 as u8 as usize].notify_self_on_meta_change);
        if should_notify {
            let (block_light, sky_light, cpos) = {
                let guard = chunk.lock().unwrap();
                (guard.get_block_light(local), guard.get_sky_light(local), guard.cpos)
            };
            if let Some(callback) = self.on_block_update.as_mut() {
                callback(
                    PendingBlock {
                        block: Block { r#type: block_type, data: metadata },
                        block_pos: wpos,
                        light: Int2::new(i32::from(block_light), i32::from(sky_light)),
                    },
                    cpos,
                );
            }
        }
    }

    pub fn is_air_block(&self, wpos: Int3) -> bool {
        self.get_block_id(wpos) == BLOCK_AIR
    }

    pub fn get_material(&self, wpos: Int3) -> Material {
        block_properties::block_properties()[self.get_block_id(wpos).0 as u8 as usize].material
    }

    pub fn is_block_normal_cube(&self, wpos: Int3) -> bool {
        let block = self.get_block_id(wpos);
        if block == BLOCK_AIR {
            return false;
        }
        let props = &block_properties::block_properties()[block.0 as u8 as usize];
        props.material.is_solid && props.is_normal_cube
    }

    pub fn find_top_solid_block(&self, wx: i32, wz: i32) -> i32 {
        let chunk = match self.get_chunk_raw(Int32_2::new(wx >> 4, wz >> 4)) {
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

    pub fn init_spawn(&mut self) {
        let mut sx = 0;
        let mut sz = 0;
        let can_coordinate_be_spawn = |world: &mut WorldManager, x: i32, z: i32| -> bool {
            let mut b = world.get_first_uncovered_block(x, z);
            if b == BLOCK_INVALID {
                // Force generate this chunk so we can check the block type.
                let cpos = Int32_2::new(x >> 4, z >> 4);
                world.force_gen_chunk_sync(cpos);
                b = world.get_first_uncovered_block(x, z);
            }
            let _ = b;
            world.get_first_uncovered_block(x, z) == BLOCK_SAND
        };
        while !can_coordinate_be_spawn(self, sx, sz) {
            sx += self.rand.next_int_bound(64) - self.rand.next_int_bound(64);
            sz += self.rand.next_int_bound(64) - self.rand.next_int_bound(64);
        }
        self.spawn_point = Int3::new(sx, 64, sz);
        self.chunks.clear(); // Clear all chunks so we can start fresh from the spawn area
    }

    // Force generate a chunk synchronously, blocking until the chunk is fully generated
    pub fn force_gen_chunk_sync(&mut self, pos: Int32_2) {
        if !self.chunks.contains_key(&pos) {
            let mut chunk = Chunk::default();
            chunk.cpos = pos;

            let region_manager = self.region_manager.clone();
            let chunk_exists = match &region_manager {
                Some(region_manager) => region_manager.lock().unwrap().chunk_exists(pos),
                None => false,
            };
            if chunk_exists {
                // Chunk already exists on disk; load it instead of regenerating it.
                chunk.state_store(ChunkState::Loading);
                self.chunks.insert(pos, Arc::new(Mutex::new(chunk)));
                if let Some(region_manager) = &region_manager {
                    region_manager.lock().unwrap().load_chunk(pos);
                }
            } else {
                // Brand new chunk; generate it on the pool, same as pumpPipeline does.
                chunk.state_store(ChunkState::Generating);
                self.chunks.insert(pos, Arc::new(Mutex::new(chunk)));
                let seed = self.seed;
                let is_hell = self.is_hell;
                let gen_done_queue = Arc::clone(&self.gen_done_queue);
                self.pool.detach_task(move || {
                    let mut gen_chunk = Chunk::default();
                    gen_chunk.cpos = pos;
                    if is_hell {
                        let mut generator = NetherGenerator::new(seed);
                        generator.generate_chunk(&mut gen_chunk);
                    } else {
                        let mut generator = OverworldGenerator::new(seed);
                        generator.generate_chunk(&mut gen_chunk);
                    }
                    gen_chunk.is_modified = true;
                    gen_chunk.generate_skylight_map();
                    gen_chunk.state_store(ChunkState::Generated);
                    gen_done_queue.lock().unwrap().push_back(Arc::new(Mutex::new(gen_chunk)));
                });
            }
        }

        // Block only on this specific chunk
        loop {
            let state = self.chunks.get(&pos).unwrap().lock().unwrap().state_load();
            if state >= ChunkState::Generated {
                break;
            }
            self.pool.wait();
            if let Some(region_manager) = self.region_manager.clone() {
                region_manager.lock().unwrap().iopool.wait();
            }
            self.drain_gen_queue();
            self.drain_load_queue();
        }
    }

    pub fn get_spawn_point(&mut self, adjust: bool) -> Int3 {
        if !adjust {
            return self.spawn_point;
        }
        let mut sx = self.spawn_point.x;
        let mut sz = self.spawn_point.z;
        sx += self.rand.next_int_bound(20) - 10;
        sz += self.rand.next_int_bound(20) - 10;
        let sy = self.find_top_solid_block(sx, sz);
        Int3::new(sx, sy, sz)
    }

    pub fn get_first_uncovered_block(&self, wx: i32, wz: i32) -> BlockType {
        let chunk = match self.get_chunk_raw(Int32_2::new(wx >> 4, wz >> 4)) {
            Some(chunk) => chunk,
            None => return BLOCK_INVALID,
        };
        let guard = chunk.lock().unwrap();
        if guard.state_load() < ChunkState::Generated {
            return BLOCK_INVALID;
        }
        let lx = wx & 15;
        let lz = wz & 15;
        let mut y = 63;
        while y < 127 && guard.get_block(Int3::new(lx, y + 1, lz)) != BLOCK_AIR {
            y += 1;
        }
        guard.get_block(Int3::new(lx, y, lz))
    }

    pub fn get_height_value(&self, wx: i32, wz: i32) -> i32 {
        let chunk = match self.get_chunk_raw(Int32_2::new(wx >> 4, wz >> 4)) {
            Some(chunk) => chunk,
            None => return 0,
        };
        let guard = chunk.lock().unwrap();
        if guard.state_load() < ChunkState::Generated {
            return 0;
        }
        i32::from(guard.get_height_value(Int2::new(wx & 15, wz & 15)))
    }

    // Returns the baked temperature/humidity for a world column.
    pub fn get_temperature_at(&self, wx: i32, wz: i32) -> f64 {
        let chunk = match self.get_chunk_raw(Int32_2::new(wx >> 4, wz >> 4)) {
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
        let chunk = match self.get_chunk_raw(Int32_2::new(wx >> 4, wz >> 4)) {
            Some(chunk) => chunk,
            None => return 0.5,
        };
        let guard = chunk.lock().unwrap();
        if guard.state_load() < ChunkState::Generated {
            return 0.5;
        }
        f64::from(guard.get_humidity(Int2::new(wx & 15, wz & 15)))
    }

    pub fn get_sky_light(&self, pos: Int3) -> i32 {
        if !Self::in_bounds(pos.y) {
            return 0;
        }
        let chunk = match self.get_chunk_raw(Int32_2::new(pos.x >> 4, pos.z >> 4)) {
            Some(chunk) => chunk,
            None => return 0,
        };
        let guard = chunk.lock().unwrap();
        if guard.state_load() < ChunkState::Generated {
            return 0;
        }
        i32::from(guard.get_sky_light(Int3::new(pos.x & 15, pos.y, pos.z & 15)))
    }

    pub fn get_block_light(&self, pos: Int3) -> i32 {
        if !Self::in_bounds(pos.y) {
            return 0;
        }
        let chunk = match self.get_chunk_raw(Int32_2::new(pos.x >> 4, pos.z >> 4)) {
            Some(chunk) => chunk,
            None => return 0,
        };
        let guard = chunk.lock().unwrap();
        if guard.state_load() < ChunkState::Generated {
            return 0;
        }
        i32::from(guard.get_block_light(Int3::new(pos.x & 15, pos.y, pos.z & 15)))
    }

    pub fn propagate_chunk_light_borders(&mut self, cpos: Int32_2) {
        // Iterate through our chunk borders
        let ndx = [-1, 1, 0, 0];
        let ndz = [0, 0, -1, 1];
        let bx = cpos.x * 16;
        let bz = *cpos.z() * 16;
        for i in 0..4 {
            let neighbor_chunk = self.get_chunk_raw(Int32_2::new(cpos.x + ndx[i], *cpos.z() + ndz[i]));
            let neighbor_chunk = match neighbor_chunk {
                Some(chunk) => chunk,
                None => continue,
            };

            // Walk the border edge of this chunk that faces the neighbor
            for t in 0..16 {
                // Pick the border column of this chunk facing direction i
                let (lx, lz, nx, nz);
                if ndx[i] == -1 {
                    lx = 0;
                    lz = t;
                    nx = 15;
                    nz = t;
                } else if ndx[i] == 1 {
                    lx = 15;
                    lz = t;
                    nx = 0;
                    nz = t;
                } else if ndz[i] == -1 {
                    lx = t;
                    lz = 0;
                    nx = t;
                    nz = 15;
                } else {
                    lx = t;
                    lz = 15;
                    nx = t;
                    nz = 0;
                }

                for y in 0..crate::constants::CHUNK_HEIGHT {
                    // Does our neighbor block have a block light > 0 or sky light > 0? If so, schedule a light update for the block on our side of the border.
                    let (block_light, sky_light) = {
                        let guard = neighbor_chunk.lock().unwrap();
                        (guard.get_block_light(Int3::new(nx, y, nz)), guard.get_sky_light(Int3::new(nx, y, nz)))
                    };
                    if block_light > 0 {
                        self.light_manager.schedule_light_update(Int3::new(bx + lx, y, bz + lz), LightType::Block);
                    }
                    if sky_light > 0 {
                        self.light_manager.schedule_light_update(Int3::new(bx + lx, y, bz + lz), LightType::Sky);
                    }
                }
            }
        }
    }

    pub fn get_chunk_raw(&self, pos: Int32_2) -> Option<Arc<Mutex<Chunk>>> {
        self.chunks.get(&pos).cloned()
    }

    pub fn is_chunk_valid(&self, pos: Int32_2) -> bool {
        let chunk = match self.get_chunk_raw(Int32_2::new(pos.x, *pos.z())) {
            Some(chunk) => chunk,
            None => return false,
        };
        chunk.lock().unwrap().state_load() >= ChunkState::Generated
    }

    pub fn aabb_in_valid_chunks(&self, collider: AABB) -> bool {
        if collider.min_y < 0.0 || collider.max_y >= 128.0 {
            return false;
        }
        let min_cx = MathHelper::floor_double(collider.min_x) >> 4;
        let max_cx = MathHelper::floor_double(collider.max_x + 1.0) >> 4;
        let min_cz = MathHelper::floor_double(collider.min_z) >> 4;
        let max_cz = MathHelper::floor_double(collider.max_z + 1.0) >> 4;

        for cx in min_cx..=max_cx {
            for cz in min_cz..=max_cz {
                if !self.is_chunk_valid(Int32_2::new(cx, cz)) {
                    return false;
                }
            }
        }
        true
    }

    pub fn block_to_chunk_pos(&self, block_pos: Int32_2) -> Int32_2 {
        Int32_2::new(block_pos.x >> 4, *block_pos.z() >> 4)
    }

    pub fn flush_bleed_writes(&mut self) {
        let positions: Vec<Int32_2> = self.pending_bleed_writes.keys().copied().collect();
        for pos in positions {
            let target = self.get_chunk_raw(pos);
            let ready = match &target {
                Some(target) => {
                    let guard = target.lock().unwrap();
                    guard.state_load() >= ChunkState::Generated && !guard.in_use.load(Ordering::SeqCst)
                }
                None => false,
            };
            if ready {
                if let Some(writes) = self.pending_bleed_writes.remove(&pos) {
                    for (wpos, block) in writes {
                        self.set_block(wpos, block.r#type, block.data);
                    }
                }
            }
        }
    }

    // Returns true when the world-space Y is within valid chunk bounds.
    pub const fn in_bounds(y: i32) -> bool {
        y >= 0 && y < crate::constants::CHUNK_HEIGHT
    }

    fn seed_chunk_lighting(&mut self, pos: Int32_2) {
        let chunk = match self.get_chunk_raw(pos) {
            Some(chunk) => chunk,
            None => return,
        };

        // We check each column in the chunk's height against its neighbors, if they differ then we schedule light updates for the vertical column between them.
        // This works like 99% of the time but can miss some edge cases; its fast though!
        let bx = pos.x * 16;
        let bz = *pos.z() * 16;
        for x in 0..16 {
            for z in 0..16 {
                let wx = bx + x;
                let wz = bz + z;
                let this_h = i32::from(chunk.lock().unwrap().get_height_value(Int2::new(x, z)));
                let ndx = [-1, 1, 0, 0];
                let ndz = [0, 0, -1, 1];
                for i in 0..4 {
                    let nx = wx + ndx[i];
                    let nz = wz + ndz[i];
                    let neighbor_h = self.get_height_value(nx, nz);
                    if neighbor_h == this_h {
                        continue;
                    }
                    let min_y = Math::min(this_h, neighbor_h);
                    let max_y = Math::max(this_h, neighbor_h);
                    self.light_manager.schedule_light_region(Int3::new(nx, min_y, nz), Int3::new(nx, max_y, nz), LightType::Sky);
                }
            }
        }

        // Block light emitters
        for x in 0..16 {
            for z in 0..16 {
                for y in 0..crate::constants::CHUNK_HEIGHT {
                    let id = chunk.lock().unwrap().get_block(Int3::new(x, y, z));
                    if block_properties::block_properties()[id.0 as u8 as usize].light_emission > 0 {
                        self.light_manager.schedule_light_update(Int3::new(bx + x, y, bz + z), LightType::Block);
                    }
                }
            }
        }
        self.propagate_chunk_light_borders(pos);
    }

    fn get_chunk_shared(&self, pos: Int32_2) -> Option<Arc<Mutex<Chunk>>> {
        self.chunks.get(&pos).cloned()
    }

    // Check if a chunk can be populated
    fn can_populate_direct(&self, pos: Int32_2) -> bool {
        let chunk = match self.get_chunk_raw(pos) {
            Some(chunk) => chunk,
            None => return false,
        };
        {
            let guard = chunk.lock().unwrap();
            if guard.is_terrain_populated {
                return false;
            }
            if guard.state_load() < ChunkState::Generated {
                return false;
            }
        }
        let a = self.get_chunk_raw(Int32_2::new(pos.x + 1, *pos.z()));
        let b = self.get_chunk_raw(Int32_2::new(pos.x, *pos.z() + 1));
        let c = self.get_chunk_raw(Int32_2::new(pos.x + 1, *pos.z() + 1));
        let (a, b, c) = match (a, b, c) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => return false,
        };
        if a.lock().unwrap().state_load() < ChunkState::Generated {
            return false;
        }
        if b.lock().unwrap().state_load() < ChunkState::Generated {
            return false;
        }
        if c.lock().unwrap().state_load() < ChunkState::Generated {
            return false;
        }
        true
    }

    fn light_manager_unlight_at(&mut self, x: i32, y: i32, z: i32, r#type: LightType) {
        let mut light_manager = std::mem::take(&mut self.light_manager);
        light_manager.unlight_at(x, y, z, r#type, self);
        self.light_manager = light_manager;
    }

    fn start_loading(&mut self, pos: Int32_2, started_this_tick: &mut HashSet<Int32_2>) -> bool {
        if started_this_tick.contains(&pos) {
            return false;
        }
        let chunk = match self.chunks.get(&pos) {
            Some(chunk) => Arc::clone(chunk),
            None => return false,
        };
        if chunk.lock().unwrap().state_load() != ChunkState::Unloaded {
            return false;
        }
        chunk.lock().unwrap().state_store(ChunkState::Loading);
        if let Some(region_manager) = self.region_manager.clone() {
            region_manager.lock().unwrap().load_chunk(pos);
        }
        started_this_tick.insert(pos);
        true
    }

    // Check if already started this tick (can happen with multiple players), and if chunk is still Unloaded (can be changed by another thread).
    fn start_generation(&mut self, pos: Int32_2, started_this_tick: &mut HashSet<Int32_2>) -> bool {
        if started_this_tick.contains(&pos) {
            return false;
        }
        let chunk = match self.chunks.get(&pos) {
            Some(chunk) => Arc::clone(chunk),
            None => return false,
        };
        if chunk.lock().unwrap().state_load() != ChunkState::Unloaded {
            return false;
        }

        // Actually generate this chunk
        chunk.lock().unwrap().state_store(ChunkState::Generating);
        let seed = self.seed;
        let is_hell = self.is_hell;
        let gen_done_queue = Arc::clone(&self.gen_done_queue);
        self.pool.detach_task(move || {
            // We make a new chunk here instead of modifying the existing chunk because multithreading is a pain
            // The placeholder chunk in the map will be replaced by this one when we push to genDoneQueue
            let mut new_chunk = Chunk::default();
            new_chunk.cpos = pos;
            if is_hell {
                let mut generator = NetherGenerator::new(seed);
                generator.generate_chunk(&mut new_chunk);
            } else {
                let mut generator = OverworldGenerator::new(seed);
                generator.generate_chunk(&mut new_chunk);
            }
            new_chunk.is_modified = true;
            new_chunk.generate_skylight_map();
            new_chunk.state_store(ChunkState::Generated);

            // This just posts the result so we can start lighting and check for population
            gen_done_queue.lock().unwrap().push_back(Arc::new(Mutex::new(new_chunk)));
        });
        started_this_tick.insert(pos);
        true
    }
}

impl Default for WorldManager {
    fn default() -> Self {
        Self::new(false)
    }
}
