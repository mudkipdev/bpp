/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, Weak};

use crate::helpers::thread_pool::ThreadPool;
use crate::logger::logger::global_logger;
use crate::nbt::nbt::Tag;
use crate::numeric_structs::{Int2, Int32_2};
use crate::world::chunk::Chunk;
use crate::world::storage::region::{Region, region_position_to_file_name};
use crate::world::world::WorldManager;

pub struct SnapshotContainer {
    pub chunk_snapshot: Arc<Mutex<Chunk>>,
    pub entity_snapshot: Arc<Vec<Tag>>,
    pub save_time: i64,
}

pub struct RegionManager {
    pub iopool: ThreadPool,

    pub save_queue: Mutex<Vec<SnapshotContainer>>,

    pub out_chunks: Arc<Mutex<HashMap<Int32_2, Arc<Mutex<Chunk>>>>>,

    // As much as I hate to do this it makes my job easier
    pub world: Weak<Mutex<WorldManager>>,

    pending_regions: Vec<Arc<Mutex<Region>>>,
    region_cache: [Option<Arc<Mutex<Region>>>; 8],
    cache_index: i32,
    folder_path: String, // Path to where all the regions get dumped
}

impl RegionManager {
    pub fn initialize(&mut self, folder_path: String) -> bool {
        if !std::path::Path::new(&folder_path).is_dir() {
            global_logger().error("Tried to initialize region manager with an invalid directory!\n");
            return false; // No region folder
        }
        self.folder_path = folder_path;
        true
    }

    pub fn release(&mut self) -> bool {
        self.flush_all();

        // Clear regions in cache
        for slot in self.region_cache.iter_mut() {
            *slot = None;
        }

        // Drop any regions that couldn't fit in cache
        self.pending_regions.clear();

        // Clear the folder path so the manager can't be accidentally reused
        self.folder_path.clear();

        self.world = Weak::new();

        true
    }

    // Does this region file exist on the disk?
    pub fn region_exists(&self, rpos: Int32_2) -> bool {
        std::path::Path::new(&format!("{}/{}", self.folder_path, region_position_to_file_name(rpos))).exists()
    }

    // Has this chunk been saved to a region file yet?
    pub fn chunk_exists(&mut self, cpos: Int32_2) -> bool {
        if !self.region_exists(Int32_2::new(cpos.x >> 5, *cpos.z() >> 5)) {
            return false;
        }
        let region = match self.load_region(Int32_2::new(cpos.x >> 5, *cpos.z() >> 5)) {
            Some(region) => region,
            None => return false,
        };
        region.lock().unwrap().chunk_exists(Int2::new(cpos.x & 31, *cpos.z() & 31))
    }

    // Creates a new region file
    pub fn create_region(&self, rpos: Int32_2) -> bool {
        let path = format!("{}/{}", self.folder_path, region_position_to_file_name(rpos));
        let _ = std::fs::create_dir_all(&self.folder_path);
        let mut file = match std::fs::File::create(&path) {
            Ok(file) => file,
            Err(_) => return false,
        };
        let zeros = vec![0u8; 8192]; // 2 sectors for the header
        if file.write_all(&zeros).is_err() {
            return false;
        }
        drop(file); // explicit close before FileHandle opens it
        true // catch write failures too
    }

    // Serialize and save a chunk to a region
    pub fn save_chunk(&mut self, chunk: Arc<Mutex<Chunk>>, entities: Vec<Tag>, current_time: i64) {
        let cpos = chunk.lock().unwrap().cpos;
        {
            self.out_chunks.lock().unwrap().remove(&cpos);
        }

        // Make a snapshot
        let mut snapshot = Chunk::default();
        {
            let guard = chunk.lock().unwrap();
            snapshot.cpos = guard.cpos;
            snapshot.is_terrain_populated = guard.is_terrain_populated;
            snapshot.is_modified = guard.is_modified;
            snapshot.spawn_chunk = guard.spawn_chunk;
            snapshot.state.store(guard.state.load(Ordering::Acquire), Ordering::SeqCst);
            snapshot.in_use.store(false, Ordering::SeqCst);
            snapshot.blocks = guard.blocks;
            snapshot.light_nibble = guard.light_nibble;
            snapshot.nibble_block_meta = guard.nibble_block_meta;
            snapshot.height_map = guard.height_map;
            snapshot.temperature = guard.temperature;
            snapshot.humidity = guard.humidity;
            snapshot.tile_entities = guard.tile_entities.clone();
        }

        // Entities are wrapped in an Arc<Vec<Tag>>
        let entities = Arc::new(entities);

        let container =
            SnapshotContainer { chunk_snapshot: Arc::new(Mutex::new(snapshot)), entity_snapshot: entities, save_time: current_time };

        self.save_queue.lock().unwrap().push(container);
    }

    // Queue a chunk to be loaded from disk
    pub fn load_chunk(&mut self, cpos: Int32_2) {
        let rpos = Int32_2::new(cpos.x >> 5, *cpos.z() >> 5);
        if !self.region_exists(rpos) {
            return;
        }
        let region = match self.load_region(rpos) {
            Some(region) => region, // shared_ptr keeps Region alive for the task
            None => return,
        };
        let out_chunks = Arc::clone(&self.out_chunks);
        self.iopool.detach_task(move || {
            let chunk = region.lock().unwrap().get_chunk(cpos); // blocks until region is free
            let chunk = match chunk {
                Some(chunk) => chunk,
                None => return,
            };
            out_chunks.lock().unwrap().insert(cpos, chunk);
        });
    }

    // Returns None until chunk is done loading
    pub fn get_chunk(&mut self, cpos: Int32_2) -> Option<Arc<Mutex<Chunk>>> {
        self.out_chunks.lock().unwrap().remove(&cpos)
    }

    pub fn pump_pipeline(&mut self) {
        let mut to_save: Vec<SnapshotContainer> = {
            let mut queue = self.save_queue.lock().unwrap();
            let taken = std::mem::take(&mut *queue);
            if queue.is_empty() {
                queue.shrink_to_fit();
            }
            taken
        };

        let mut requeue: Vec<SnapshotContainer> = Vec::new();
        for snapshot in to_save.drain(..) {
            let chunk = Arc::clone(&snapshot.chunk_snapshot);
            let entity_snapshot = Arc::clone(&snapshot.entity_snapshot); // shared_ptr copy
            let cpos = chunk.lock().unwrap().cpos;
            let rpos = Int32_2::new(cpos.x >> 5, *cpos.z() >> 5);
            if !self.region_exists(rpos) {
                self.create_region(rpos);
            }
            let region = self.load_region(rpos); // shared_ptr keeps Region alive for the task
            let region = match region {
                Some(region) => region,
                None => {
                    requeue.push(snapshot); // keep chunk + entities together
                    continue;
                }
            };
            let current_time = snapshot.save_time;
            self.iopool.detach_task(move || {
                region.lock().unwrap().add_chunk(chunk, current_time, Some(entity_snapshot)); // Region stays alive via shared_ptr capture
            });
        }

        if !requeue.is_empty() {
            let mut queue = self.save_queue.lock().unwrap();
            queue.extend(requeue);
        }

        // Try to merge any pending regions that couldn't fit before
        let pending = std::mem::take(&mut self.pending_regions);
        let mut still_pending = Vec::new();
        for region in pending {
            if !self.try_merge_pending_region(&region) {
                still_pending.push(region);
            }
        }
        self.pending_regions = still_pending;
    }

    // Flush all pending saves synchronously
    pub fn flush_all(&mut self) {
        self.pump_pipeline();
        self.iopool.wait();
    }

    // Loads a region into cache, creating the file if needed
    pub fn load_region(&mut self, rpos: Int32_2) -> Option<Arc<Mutex<Region>>> {
        // Check cache first
        for slot in self.region_cache.iter() {
            if let Some(region) = slot {
                if region.lock().unwrap().rpos == rpos {
                    return Some(Arc::clone(region));
                }
            }
        }

        // Also check regions awaiting merge
        for pending in &self.pending_regions {
            if pending.lock().unwrap().rpos == rpos {
                return Some(Arc::clone(pending));
            }
        }

        if !self.region_exists(rpos) {
            if !self.create_region(rpos) {
                global_logger().error(format!("Failed to create region file for {},{}\n", rpos.x, *rpos.z()));
                return None;
            }
        }

        if self.create_region_on_cache(Int2::new(rpos.x, *rpos.z())) {
            for slot in self.region_cache.iter() {
                if let Some(region) = slot {
                    if region.lock().unwrap().rpos == rpos {
                        return Some(Arc::clone(region));
                    }
                }
            }
        }

        None // all 8 slots still busy
    }

    fn try_merge_pending_region(&mut self, region: &Arc<Mutex<Region>>) -> bool {
        let region_rpos = region.lock().unwrap().rpos;
        for slot in self.region_cache.iter() {
            if let Some(cached) = slot {
                if cached.lock().unwrap().rpos == region_rpos {
                    return true; // already cached
                }
            }
        }
        for slot in self.region_cache.iter_mut() {
            if slot.is_none() {
                *slot = Some(Arc::clone(region));
                return true;
            }
            // Evict slot if no IO task currently holds a reference to it.
            if Arc::strong_count(slot.as_ref().unwrap()) == 1 {
                *slot = Some(Arc::clone(region));
                return true;
            }
        }
        false // all 8 slots actively in use
    }

    fn create_region_on_cache(&mut self, rpos: Int2) -> bool {
        let region = Arc::new(Mutex::new(Region::new(rpos, self.folder_path.clone())));
        if !self.try_merge_pending_region(&region) {
            self.pending_regions.push(region);
            false
        } else {
            true
        }
    }
}

impl Drop for RegionManager {
    fn drop(&mut self) {
        self.release();
    }
}

impl Default for RegionManager {
    fn default() -> Self {
        Self {
            iopool: ThreadPool::new(2),
            save_queue: Mutex::new(Vec::new()),
            out_chunks: Arc::new(Mutex::new(HashMap::new())),
            world: Weak::new(),
            pending_regions: Vec::new(),
            region_cache: std::array::from_fn(|_| None),
            cache_index: 0,
            folder_path: String::new(),
        }
    }
}
