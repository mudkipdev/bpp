/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

// Lighter used by the main world.
use std::sync::{Arc, Mutex};

use crate::base_structs::Block;
use crate::blocks::block_properties;
use crate::constants::CHUNK_HEIGHT;
use crate::helpers::cross_platform::Math;
use crate::numeric_structs::{Int2, Int3, Int32_2};
use crate::world::chunk::{Chunk, ChunkState};
use crate::world::world::{PendingBlock, WorldManager};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LightType {
    Sky,
    Block,
}

// A light update entry covering a 3-D axis-aligned region [min, max] inclusive.
pub struct LightRegion {
    pub min: Int3,
    pub max: Int3,
    pub r#type: LightType,
}

impl LightRegion {
    // Returns true if [x1,y1,z1]->[x2,y2,z2] is contained within (or close
    // enough to merge into) this region, expanding it in-place when the volume
    // grows by <=2
    pub fn try_merge(&mut self, x1: i32, y1: i32, z1: i32, x2: i32, y2: i32, z2: i32) -> bool {
        if x1 >= self.min.x && y1 >= self.min.y && z1 >= self.min.z && x2 <= self.max.x && y2 <= self.max.y && z2 <= self.max.z {
            return true;
        }

        if x1 < self.min.x - 1 || y1 < self.min.y - 1 || z1 < self.min.z - 1 {
            return false;
        }
        if x2 > self.max.x + 1 || y2 > self.max.y + 1 || z2 > self.max.z + 1 {
            return false;
        }

        let old_vol = (self.max.x - self.min.x + 1) * (self.max.y - self.min.y + 1) * (self.max.z - self.min.z + 1);
        let nx1 = Math::min(self.min.x, x1);
        let ny1 = Math::min(self.min.y, y1);
        let nz1 = Math::min(self.min.z, z1);
        let nx2 = Math::max(self.max.x, x2);
        let ny2 = Math::max(self.max.y, y2);
        let nz2 = Math::max(self.max.z, z2);
        let new_vol = (nx2 - nx1 + 1) * (ny2 - ny1 + 1) * (nz2 - nz1 + 1);
        if new_vol - old_vol > 2 {
            return false;
        }

        self.min = Int3::new(nx1, ny1, nz1);
        self.max = Int3::new(nx2, ny2, nz2);
        true
    }
}

pub struct UnlightUpdate {
    pub pos: Int3,
    pub r#type: LightType,
    pub val: i32,
}

// 3x3 grid of chunk pointers centered on a chunk coordinate.
pub struct ChunkCache {
    pub grid: [[Option<Arc<Mutex<Chunk>>>; 3]; 3],
    pub cx: i32, // chunk coords of center (grid[1][1])
    pub cz: i32,
}

impl ChunkCache {
    // Fetch chunk at chunk-coord (tcx, tcz) from the grid.
    pub fn get(&self, tcx: i32, tcz: i32) -> Option<Arc<Mutex<Chunk>>> {
        let dx = tcx.wrapping_sub(self.cx);
        let dz = tcz.wrapping_sub(self.cz);
        if dx < -1 || dx > 1 || dz < -1 || dz > 1 {
            return None;
        }
        self.grid[(dx + 1) as usize][(dz + 1) as usize].clone()
    }

    // ChunkCache so we don't have to do map lookups for every neighbor access during light propagation.
    // Reuses chunks from the previous window whenever they overlap the new one.
    // Refresh the entire 3x3 grid around (ncx, ncz) if the center has changed.
    pub fn refresh(&mut self, ncx: i32, ncz: i32, world: &WorldManager) {
        if ncx == self.cx && ncz == self.cz {
            return;
        }

        let old_grid = self.grid.clone();
        let old_cx = self.cx;
        let old_cz = self.cz;

        self.cx = ncx;
        self.cz = ncz;

        for dx in -1..=1 {
            for dz in -1..=1 {
                let tcx = ncx + dx;
                let tcz = ncz + dz;
                let odx = tcx.wrapping_sub(old_cx);
                let odz = tcz.wrapping_sub(old_cz);

                // Already held from the previous window? Reuse it as-is.
                let mut c = if (-1..=1).contains(&odx) && (-1..=1).contains(&odz) {
                    old_grid[(odx + 1) as usize][(odz + 1) as usize].clone()
                } else {
                    None
                };

                if c.is_none() {
                    let fetched = world.get_chunk_raw(Int32_2::new(tcx, tcz));
                    c = match fetched {
                        Some(chunk) => {
                            let generated = chunk.lock().unwrap().state_load() >= ChunkState::Generated;
                            if generated { Some(chunk) } else { None }
                        }
                        None => None,
                    };
                }

                self.grid[(dx + 1) as usize][(dz + 1) as usize] = c;
            }
        }
    }
}

impl Default for ChunkCache {
    fn default() -> Self {
        Self { grid: Default::default(), cx: i32::MIN, cz: i32::MIN }
    }
}

fn get_light_direct(chunk: Option<&Arc<Mutex<Chunk>>>, lx: i32, y: i32, lz: i32, r#type: LightType) -> i32 {
    let Some(chunk) = chunk else { return 0 };
    if y < 0 || y >= CHUNK_HEIGHT {
        return 0;
    }
    let chunk = chunk.lock().unwrap();
    let pos = Int3::new(lx, y, lz);
    i32::from(if r#type == LightType::Sky { chunk.get_sky_light(pos) } else { chunk.get_block_light(pos) })
}

fn maybe_queue(
    light_queue: &mut Vec<LightRegion>,
    nx: i32,
    ny: i32,
    nz: i32,
    nc: Option<&Arc<Mutex<Chunk>>>,
    nlx: i32,
    nlz: i32,
    r#type: LightType,
    new_val: i32,
) {
    if ny < 0 || ny >= CHUNK_HEIGHT || nc.is_none() {
        return;
    }
    let expected = Math::max(0, new_val - 1);
    if get_light_direct(nc, nlx, ny, nlz, r#type) < expected && light_queue.len() < 1_000_000 {
        light_queue.push(LightRegion { min: Int3::new(nx, ny, nz), max: Int3::new(nx, ny, nz), r#type });
    }
}

pub struct Lighter {
    light_queue: Vec<LightRegion>,
    unlight_queue: Vec<UnlightUpdate>,
    processing_depth: i32,

    // Persistent cache for unlightAt
    unlight_cache: ChunkCache,
}

impl Lighter {
    pub fn propagate_light_at(&mut self, x: i32, y: i32, z: i32, r#type: LightType, world: &mut WorldManager, cache: &mut ChunkCache) {
        if y < 0 || y >= CHUNK_HEIGHT {
            return;
        }

        let cx = x >> 4;
        let cz = z >> 4;

        // Refresh the 3x3 cache only when we cross a chunk boundary.
        cache.refresh(cx, cz, world);

        // Center
        let chunk = match cache.grid[1][1].clone() {
            Some(chunk) => chunk,
            None => return,
        };

        let lx = x & 15;
        let lz = z & 15;
        let block_id = chunk.lock().unwrap().get_block(Int3::new(lx, y, lz));
        let mut opacity = i32::from(block_properties::block_properties()[block_id.0 as u8 as usize].light_opacity);
        if opacity == 0 {
            opacity = 1;
        }

        // Pick neighbor chunk pointers
        let cxn = if lx == 0 { cache.grid[0][1].clone() } else { Some(chunk.clone()) };
        let cxp = if lx == 15 { cache.grid[2][1].clone() } else { Some(chunk.clone()) };
        let czn = if lz == 0 { cache.grid[1][0].clone() } else { Some(chunk.clone()) };
        let czp = if lz == 15 { cache.grid[1][2].clone() } else { Some(chunk.clone()) };

        let lxn = if lx == 0 { 15 } else { lx - 1 };
        let lxp = if lx == 15 { 0 } else { lx + 1 };
        let lzn = if lz == 0 { 15 } else { lz - 1 };
        let lzp = if lz == 15 { 0 } else { lz + 1 };

        let mut new_val: i32 = 0;

        if r#type == LightType::Sky {
            let can_see_sky = chunk.lock().unwrap().can_block_see_sky(Int3::new(lx, y, lz));
            if can_see_sky {
                new_val = 15;
            } else if opacity < 15 {
                let mut best = 0;
                best = Math::max(best, get_light_direct(cxn.as_ref(), lxn, y, lz, r#type));
                best = Math::max(best, get_light_direct(cxp.as_ref(), lxp, y, lz, r#type));
                best = Math::max(best, get_light_direct(Some(&chunk), lx, y - 1, lz, r#type));
                best = Math::max(best, get_light_direct(Some(&chunk), lx, y + 1, lz, r#type));
                best = Math::max(best, get_light_direct(czn.as_ref(), lx, y, lzn, r#type));
                best = Math::max(best, get_light_direct(czp.as_ref(), lx, y, lzp, r#type));
                new_val = Math::max(0, best - opacity);
            }
            let old_val = i32::from(chunk.lock().unwrap().get_sky_light(Int3::new(lx, y, lz)));
            if old_val == new_val {
                return;
            }
            chunk.lock().unwrap().set_sky_light(Int3::new(lx, y, lz), new_val as u8);
            // Call a block update on the block that had its lighting updated
            // Beta doesn't have a direct on lighting change packet for server -> client
            // So this is what we have to resort to
            // Technically, this doesn't work for any light update that doesn't change more than 9 blocks but its good enough
            if let Some(callback) = world.on_block_update.as_mut() {
                let (meta, block_light, cpos) = {
                    let chunk = chunk.lock().unwrap();
                    (chunk.get_meta(Int3::new(lx, y, lz)), chunk.get_block_light(Int3::new(lx, y, lz)), chunk.cpos)
                };
                callback(
                    PendingBlock {
                        block: Block { r#type: block_id, data: meta },
                        block_pos: Int3::new(x, y, z),
                        light: Int2::new(i32::from(block_light), new_val),
                    },
                    cpos,
                );
            }
        } else {
            let emitted = i32::from(block_properties::block_properties()[block_id.0 as u8 as usize].light_emission);
            if opacity < 15 {
                let mut best = 0;
                best = Math::max(best, get_light_direct(cxn.as_ref(), lxn, y, lz, r#type));
                best = Math::max(best, get_light_direct(cxp.as_ref(), lxp, y, lz, r#type));
                best = Math::max(best, get_light_direct(Some(&chunk), lx, y - 1, lz, r#type));
                best = Math::max(best, get_light_direct(Some(&chunk), lx, y + 1, lz, r#type));
                best = Math::max(best, get_light_direct(czn.as_ref(), lx, y, lzn, r#type));
                best = Math::max(best, get_light_direct(czp.as_ref(), lx, y, lzp, r#type));
                new_val = Math::max(emitted, best - opacity);
            } else {
                new_val = emitted;
            }
            let old_val = i32::from(chunk.lock().unwrap().get_block_light(Int3::new(lx, y, lz)));
            if old_val == new_val {
                return;
            }
            chunk.lock().unwrap().set_block_light(Int3::new(lx, y, lz), new_val as u8);
            if let Some(callback) = world.on_block_update.as_mut() {
                let (meta, sky_light, cpos) = {
                    let chunk = chunk.lock().unwrap();
                    (chunk.get_meta(Int3::new(lx, y, lz)), chunk.get_sky_light(Int3::new(lx, y, lz)), chunk.cpos)
                };
                callback(
                    PendingBlock {
                        block: Block { r#type: block_id, data: meta },
                        block_pos: Int3::new(x, y, z),
                        light: Int2::new(new_val, i32::from(sky_light)),
                    },
                    cpos,
                );
            }
        }

        // Propagate to neighbors
        maybe_queue(&mut self.light_queue, x - 1, y, z, cxn.as_ref(), lxn, lz, r#type, new_val);
        maybe_queue(&mut self.light_queue, x + 1, y, z, cxp.as_ref(), lxp, lz, r#type, new_val);
        maybe_queue(&mut self.light_queue, x, y - 1, z, Some(&chunk), lx, lz, r#type, new_val);
        maybe_queue(&mut self.light_queue, x, y + 1, z, Some(&chunk), lx, lz, r#type, new_val);
        maybe_queue(&mut self.light_queue, x, y, z - 1, czn.as_ref(), lx, lzn, r#type, new_val);
        maybe_queue(&mut self.light_queue, x, y, z + 1, czp.as_ref(), lx, lzp, r#type, new_val);
    }

    // Separate function for this since beta's lighting engine can't decrease light values in place
    pub fn unlight_at(&mut self, x: i32, y: i32, z: i32, r#type: LightType, world: &mut WorldManager) {
        if y < 0 || y >= CHUNK_HEIGHT {
            return;
        }

        self.unlight_cache.refresh(x >> 4, z >> 4, world);
        let chunk = match self.unlight_cache.grid[1][1].clone() {
            Some(chunk) => chunk,
            None => return,
        };

        let lx = x & 15;
        let lz = z & 15;
        let old_val = {
            let chunk = chunk.lock().unwrap();
            i32::from(if r#type == LightType::Sky {
                chunk.get_sky_light(Int3::new(lx, y, lz))
            } else {
                chunk.get_block_light(Int3::new(lx, y, lz))
            })
        };
        if old_val == 0 {
            return;
        }

        if r#type == LightType::Sky {
            chunk.lock().unwrap().set_sky_light(Int3::new(lx, y, lz), 0);
        } else {
            chunk.lock().unwrap().set_block_light(Int3::new(lx, y, lz), 0);
        }
        if let Some(callback) = world.on_block_update.as_mut() {
            let (block, meta, block_light, sky_light, cpos) = {
                let chunk = chunk.lock().unwrap();
                (
                    chunk.get_block(Int3::new(lx, y, lz)),
                    chunk.get_meta(Int3::new(lx, y, lz)),
                    chunk.get_block_light(Int3::new(lx, y, lz)),
                    chunk.get_sky_light(Int3::new(lx, y, lz)),
                    chunk.cpos,
                )
            };
            callback(
                PendingBlock {
                    block: Block { r#type: block, data: meta },
                    block_pos: Int3::new(x, y, z),
                    // Whichever type we just unlit was just set to 0 above
                    light: Int2::new(
                        if r#type == LightType::Block { 0 } else { i32::from(block_light) },
                        if r#type == LightType::Sky { 0 } else { i32::from(sky_light) },
                    ),
                },
                cpos,
            );
        }

        self.unlight_queue.push(UnlightUpdate { pos: Int3::new(x, y, z), r#type, val: old_val });

        // Drain everything at once since there isn't a lot of unlighting events
        while let Some(update) = self.unlight_queue.pop() {
            let pos = update.pos;
            let t = update.r#type;
            let val = update.val;

            self.unlight_cache.refresh(pos.x >> 4, pos.z >> 4, world);

            let ndx = [-1, 1, 0, 0, 0, 0];
            let ndy = [0, 0, -1, 1, 0, 0];
            let ndz = [0, 0, 0, 0, -1, 1];
            for i in 0..6 {
                let nx = pos.x + ndx[i];
                let ny = pos.y + ndy[i];
                let nz = pos.z + ndz[i];
                if ny < 0 || ny >= CHUNK_HEIGHT {
                    continue;
                }

                let ncx = nx >> 4;
                let ncz = nz >> 4;
                let nlx = nx & 15;
                let nlz = nz & 15;
                let dx = ncx - self.unlight_cache.cx;
                let dz = ncz - self.unlight_cache.cz;
                let nc = if (-1..=1).contains(&dx) && (-1..=1).contains(&dz) {
                    self.unlight_cache.grid[(dx + 1) as usize][(dz + 1) as usize].clone()
                } else {
                    world.get_chunk_raw(Int32_2::new(ncx, ncz))
                };
                let nc = match nc {
                    Some(nc) => nc,
                    None => continue,
                };

                let n_val = {
                    let nc = nc.lock().unwrap();
                    i32::from(if t == LightType::Sky { nc.get_sky_light(Int3::new(nlx, ny, nlz)) } else { nc.get_block_light(Int3::new(nlx, ny, nlz)) })
                };
                if n_val == 0 {
                    continue;
                }

                if n_val < val {
                    if t == LightType::Sky {
                        nc.lock().unwrap().set_sky_light(Int3::new(nlx, ny, nlz), 0);
                    } else {
                        nc.lock().unwrap().set_block_light(Int3::new(nlx, ny, nlz), 0);
                    }
                    if let Some(callback) = world.on_block_update.as_mut() {
                        let (block, meta, block_light, sky_light, cpos) = {
                            let nc = nc.lock().unwrap();
                            (
                                nc.get_block(Int3::new(nlx, ny, nlz)),
                                nc.get_meta(Int3::new(nlx, ny, nlz)),
                                nc.get_block_light(Int3::new(nlx, ny, nlz)),
                                nc.get_sky_light(Int3::new(nlx, ny, nlz)),
                                nc.cpos,
                            )
                        };
                        callback(
                            PendingBlock {
                                block: Block { r#type: block, data: meta },
                                block_pos: Int3::new(nx, ny, nz),
                                light: Int2::new(i32::from(block_light), i32::from(sky_light)),
                            },
                            cpos,
                        );
                    }
                    self.unlight_queue.push(UnlightUpdate { pos: Int3::new(nx, ny, nz), r#type: t, val: n_val });
                    // Always re-queue for re-light
                    self.schedule_light_update(Int3::new(nx, ny, nz), t);
                } else {
                    // Neighbor is at least as bright
                    self.schedule_light_update(Int3::new(nx, ny, nz), t);
                }
            }
        }
    }

    // Process up to `maxIterations` light-queue entries this call.
    pub fn process_light_queue(&mut self, world: &mut WorldManager, max_iterations: i32) -> bool {
        if self.processing_depth >= 50 {
            return !self.light_queue.is_empty();
        }
        self.processing_depth += 1;

        let mut cache = ChunkCache::default();
        let mut iters = 0;

        while !self.light_queue.is_empty() && iters < max_iterations {
            let mut region = self.light_queue.pop().unwrap();
            iters += 1;

            let dx = region.max.x - region.min.x + 1;
            let dy = region.max.y - region.min.y + 1;
            let dz = region.max.z - region.min.z + 1;
            if dx * dy * dz > 32768 {
                continue;
            }

            region.min.y = Math::max(region.min.y, 0);
            region.max.y = Math::min(region.max.y, CHUNK_HEIGHT - 1);

            for x in region.min.x..=region.max.x {
                for z in region.min.z..=region.max.z {
                    for y in region.min.y..=region.max.y {
                        self.propagate_light_at(x, y, z, region.r#type, world, &mut cache);
                    }
                }
            }
        }

        self.processing_depth -= 1;
        if self.light_queue.is_empty() {
            self.light_queue.shrink_to_fit();
        }

        !self.light_queue.is_empty()
    }

    // Schedule a single-block update; bypasses merge (used for BFS fan-out).
    pub fn schedule_light_update(&mut self, pos: Int3, r#type: LightType) {
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT {
            return;
        }
        if self.light_queue.len() < 1_000_000 {
            self.light_queue.push(LightRegion { min: pos, max: pos, r#type });
        }
    }

    // Schedule a region [mn, mx] update with merge/dedup
    pub fn schedule_light_region(&mut self, mn: Int3, mx: Int3, r#type: LightType) {
        let mut mn = mn;
        let mut mx = mx;
        mn.y = Math::max(mn.y, 0);
        mx.y = Math::min(mx.y, CHUNK_HEIGHT - 1);
        if mn.y > mx.y {
            return;
        }

        let check_count = Math::min(self.light_queue.len(), 5);
        for i in 0..check_count {
            let idx = self.light_queue.len() - 1 - i;
            if self.light_queue[idx].r#type != r#type {
                continue;
            }
            if self.light_queue[idx].try_merge(mn.x, mn.y, mn.z, mx.x, mx.y, mx.z) {
                return;
            }
        }

        if self.light_queue.len() >= 1_000_000 {
            return;
        }
        self.light_queue.push(LightRegion { min: mn, max: mx, r#type });
    }
}

impl Default for Lighter {
    fn default() -> Self {
        Self { light_queue: Vec::new(), unlight_queue: Vec::new(), processing_depth: 0, unlight_cache: ChunkCache::default() }
    }
}
