/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bpp_shared::base_structs::Block;
use bpp_shared::constants::{CHUNK_HEIGHT, CHUNK_WIDTH};
use bpp_shared::helpers::cross_platform::Math;
use bpp_shared::helpers::thread_pool::{Task, ThreadPool};
use bpp_shared::networking::packets::{
    ChunkData, PacketBehavior, SetBlock, SetChunkVisibility, SetMultipleBlocks,
};
use bpp_shared::numeric_structs::{Int32_2, SlimInt3};
use bpp_shared::world::chunk::{Chunk, ChunkState};
use bpp_shared::world::world::{PendingBlock, WorldManager};

use crate::chunk_io::chunk_serializer;
use crate::player_conn::player_session::PlayerSession;

// One result slot per in-flight chunk.
pub struct PendingChunk {
    pub pos: Int32_2,
    pub data: Task<Vec<u8>>,          // async compression result
    pub chunk_ref: Arc<Mutex<Chunk>>, // kept alive until flush drains pending updates
}

// One result slot per in-flight sub-region block update (>= 10 changes).
pub struct PendingSubRegion {
    pub chunk_pos: Int32_2,
    pub header: ChunkData, // pre-filled, compressedData empty until ready
    pub data: Option<Task<Vec<u8>>>,
}

// ChunkSender offloads zlib chunk serialization onto a thread-pool
pub struct ChunkSender {
    // Per-session queue of in-flight full-chunk serialization jobs.
    pub in_flight: HashMap<usize, Vec<PendingChunk>>,

    // Per-session queue of in-flight sub-region compression jobs.
    // Drained in-order by flush() after the full-chunk queue.
    pub sub_region_flight: HashMap<usize, Vec<PendingSubRegion>>,

    pub pool: ThreadPool,
}

impl ChunkSender {
    const POOL_THREAD_COUNT: usize = 2;

    pub fn enqueue(
        &mut self,
        session: &mut PlayerSession,
        world: &mut WorldManager,
        batch_size: i32,
    ) -> usize {
        let mut batch_size = batch_size;
        if batch_size < 0 {
            batch_size = Self::POOL_THREAD_COUNT as i32 * 2;
        }
        let cx = (session.position.pos.x.floor() as i32) >> 4;
        let cz = (session.position.pos.z.floor() as i32) >> 4;

        let radius = world.get_view_radius();

        let mut to_send: Vec<Int32_2> = Vec::new();
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let p = Int32_2::new(cx + dx, cz + dz);
                if !world.chunks.contains_key(&p) {
                    continue;
                }
                if session.sent_chunks.contains(&p) {
                    continue;
                }
                if world.chunks[&p].lock().unwrap().state_load() < ChunkState::Generated {
                    continue;
                }
                to_send.push(p);
            }
        }

        // Sort closer chunks first
        to_send.sort_by(|a, b| {
            let da = (a.x - cx).abs() + (*a.z() - cz).abs();
            let db = (b.x - cx).abs() + (*b.z() - cz).abs();
            da.cmp(&db)
        });

        // Unload all out-of-range chunks immediately
        let mut to_unload: Vec<Int32_2> = Vec::new();
        for p in session.sent_chunks.iter() {
            if (p.x - cx).abs() > radius || (*p.z() - cz).abs() > radius {
                to_unload.push(*p);
            }
        }
        for p in to_unload.iter() {
            let mut vis = SetChunkVisibility::new();
            vis.pos.x = p.x;
            *vis.pos.z_mut() = *p.z();
            vis.visible = false;
            vis.serialize(&mut session.stream);
            session.sent_chunks.remove(p);
            session.flushed_chunks.remove(p);
            session.pending_block_changes.remove(p); // drop queued updates for unloaded chunk
            session.newly_unloaded.push(*p);
        }

        // Also cancel any in-flight jobs for chunks that are now out of range
        let key = session as *const PlayerSession as usize;
        let queue = self.in_flight.entry(key).or_default();
        queue.retain(|pc| !((pc.pos.x - cx).abs() > radius || (*pc.pos.z() - cz).abs() > radius));

        // Rebuild toSend, excluding chunks that already have an in-flight job.
        let mut submitted: usize = 0;
        for p in to_send.iter() {
            if batch_size > 0 && submitted as i32 >= batch_size {
                break;
            }
            let chunk_ref = Arc::clone(world.chunks.get(p).unwrap());
            let task_chunk_ref = Arc::clone(&chunk_ref);
            let pc = PendingChunk {
                pos: *p,
                chunk_ref,
                data: self.pool.submit_task(move || {
                    chunk_serializer::serialize(&task_chunk_ref.lock().unwrap())
                }),
            };
            queue.push(pc);
            session.sent_chunks.insert(*p);
            submitted += 1;
        }
        submitted
    }

    pub fn send_block_updates(
        &mut self,
        session: &mut PlayerSession,
        chunk: &Int32_2,
        changes: &[PendingBlock],
        chunk_ref: Option<Arc<Mutex<Chunk>>>,
    ) {
        if changes.is_empty() {
            return;
        }

        if changes.len() == 1 {
            let pb = &changes[0];
            let mut sb = SetBlock::new();
            sb.block = Block {
                r#type: pb.block.r#type,
                data: pb.block.data,
            };
            sb.position = SlimInt3::new(
                pb.block_pos.x + (chunk.x * 16),
                pb.block_pos.y as i8,
                pb.block_pos.z + (*chunk.z() * 16),
            );
            sb.serialize(&mut session.stream);
        } else if changes.len() < 10 {
            fn format_multi_block(x: i8, y: i8, z: i8) -> i16 {
                ((i16::from(x) & 0x0F) << 12) | ((i16::from(z) & 0x0F) << 8) | (i16::from(y) & 0xFF)
            }
            let mut smb = SetMultipleBlocks::new();
            smb.chunk_position = Int32_2::new(chunk.x, *chunk.z());
            for pb in changes.iter() {
                smb.block_coordinates.push(format_multi_block(
                    pb.block_pos.x as i8,
                    pb.block_pos.y as i8,
                    pb.block_pos.z as i8,
                ));
                smb.block_metadata.push(pb.block.data as i8);
                smb.block_types.push(pb.block.r#type);
            }
            smb.number_of_blocks = smb.block_coordinates.len() as i16;
            smb.serialize(&mut session.stream);
        } else {
            // Find bounding box in chunk-local space
            let p0 = &changes[0].block_pos;
            let mut xmin = p0.x;
            let mut xmax = p0.x;
            let mut ymin = p0.y;
            let mut ymax = p0.y;
            let mut zmin = p0.z;
            let mut zmax = p0.z;
            for pb in changes.iter() {
                let pos = &pb.block_pos;
                if pos.x > xmax {
                    xmax = pos.x;
                }
                if pos.x < xmin {
                    xmin = pos.x;
                }
                if pos.y > ymax {
                    ymax = pos.y;
                }
                if pos.y < ymin {
                    ymin = pos.y;
                }
                if pos.z > zmax {
                    zmax = pos.z;
                }
                if pos.z < zmin {
                    zmin = pos.z;
                }
            }
            // Force even ySize so the client's nibble copy doesn't desync
            ymin = (ymin / 2) * 2;
            ymax = (ymax / 2 + 1) * 2 - 1;
            ymin = Math::max(ymin, 0);
            ymax = Math::min(ymax, CHUNK_HEIGHT - 1);

            let mut psr = PendingSubRegion {
                chunk_pos: *chunk,
                header: ChunkData::new(),
                data: None,
            };
            psr.header.pos.x = chunk.x * CHUNK_WIDTH + xmin;
            psr.header.pos.z = *chunk.z() * CHUNK_WIDTH + zmin;
            psr.header.pos.y = ymin as i16;
            psr.header.size.x = (xmax - xmin) as u8;
            psr.header.size.y = (ymax - ymin) as u8;
            psr.header.size.z = (zmax - zmin) as u8;
            if let Some(chunk_ref) = chunk_ref {
                let reff = Arc::clone(&chunk_ref);
                psr.data = Some(self.pool.submit_task(move || {
                    chunk_serializer::serialize_region(
                        &reff.lock().unwrap(),
                        xmin,
                        xmax + 1,
                        ymin,
                        ymax + 1,
                        zmin,
                        zmax + 1,
                    )
                }));
            }
            self.sub_region_flight
                .entry(session as *const PlayerSession as usize)
                .or_default()
                .push(psr);
        }
    }

    // Drains every job that is already done and writes the resulting
    // SetChunkVisibility + ChunkData packets to the session stream.
    // Jobs that aren't finished yet stay in the queue for the next tick.
    pub fn flush(&mut self, session: &mut PlayerSession) {
        let key = session as *const PlayerSession as usize;
        let queue = match self.in_flight.get_mut(&key) {
            Some(queue) => queue,
            None => return,
        };

        let mut still_pending: Vec<PendingChunk> = Vec::new();
        let taken = std::mem::take(queue);

        for mut pc in taken {
            // Non-blocking check: only consume results that are ready now.
            if !pc.data.is_ready() {
                still_pending.push(pc);
                continue;
            }

            let compressed = pc.data.get();

            let mut vis = SetChunkVisibility::new();
            vis.pos.x = pc.pos.x;
            *vis.pos.z_mut() = *pc.pos.z();
            vis.visible = true;
            vis.serialize(&mut session.stream);

            let mut data = ChunkData::new();
            data.pos.x = pc.pos.x * 16;
            data.pos.z = *pc.pos.z() * 16;
            data.compressed_data = compressed;
            data.serialize(&mut session.stream);

            session.flushed_chunks.insert(pc.pos);
            session.newly_flushed.push(pc.pos);

            // Drain any block updates that queued up while this chunk
            // was in-flight. They go out immediately after the chunk
            // data in the same tick, so the client receives them in
            // order and applies them to freshly loaded terrain.
            if let Some(pending) = session.pending_block_changes.remove(&pc.pos) {
                self.send_block_updates(
                    session,
                    &pc.pos,
                    &pending,
                    Some(Arc::clone(&pc.chunk_ref)),
                );
            }
        }

        *self.in_flight.entry(key).or_default() = still_pending;

        // Drain in-flight sub-region compression jobs in submission order.
        if let Some(sr_queue) = self.sub_region_flight.get_mut(&key) {
            while let Some(psr) = sr_queue.first_mut() {
                let ready = match psr.data.as_mut() {
                    Some(data) => data.is_ready(),
                    None => false,
                };
                if !ready {
                    break;
                }
                let compressed = psr.data.take().unwrap().get();
                psr.header.compressed_data = compressed;
                psr.header.serialize(&mut session.stream);
                sr_queue.remove(0);
            }
        }
    }

    // Remove all in-flight state for a disconnected session.
    pub fn remove(&mut self, session: &PlayerSession) {
        let key = session as *const PlayerSession as usize;
        self.in_flight.remove(&key);
        self.sub_region_flight.remove(&key);
    }
}

impl Default for ChunkSender {
    fn default() -> Self {
        Self {
            in_flight: HashMap::new(),
            sub_region_flight: HashMap::new(),
            pool: ThreadPool::new(Self::POOL_THREAD_COUNT),
        }
    }
}
