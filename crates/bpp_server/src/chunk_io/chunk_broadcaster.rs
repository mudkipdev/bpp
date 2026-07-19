/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
use std::collections::HashMap;
use std::sync::Arc;

use bpp_shared::base_structs::Block;
use bpp_shared::networking::network_stream::NetworkStream;
use bpp_shared::networking::packets::{PacketBehavior, SetBlock, SetMultipleBlocks};
use bpp_shared::numeric_structs::{Int32_2, SlimInt3};
use bpp_shared::world::world::{PendingBlock, WorldManager};

use crate::player_conn::player_session::ConnectionState;
use crate::server::Server;

// Sends accumulated per-tick block changes out to whichever player sessions care about them
pub fn broadcast_block_changes(
    server: &mut Server,
    changes: &mut HashMap<Int32_2, Vec<PendingBlock>>,
    dimension: i8,
    dim_world: &mut WorldManager,
) {
    for (chunk, block_changes) in changes.iter() {
        // Find which sessions care about this chunk
        // Split into flushed (send immediately) and sentOnly (queue).
        let index_it = server
            .chunk_sessions
            .get(&Server::chunk_key(chunk, dimension));
        let mut flushed_sessions = Vec::new();
        let mut sent_only_sessions = Vec::new();

        if let Some(index_it) = index_it {
            flushed_sessions = index_it.clone();
        }

        // Sessions that have the chunk in-flight (sentChunks but not flushedChunks) still need to queue the updates.
        for session in server.players.iter() {
            let guard = session.lock().unwrap();
            if guard.conn_state != ConnectionState::Playing
                && guard.conn_state != ConnectionState::WaitingForSpawnChunks
            {
                continue;
            }
            if guard.dimension != dimension {
                continue;
            }
            if guard.flushed_chunks.contains(chunk) {
                continue; // already in flushedSessions
            }
            if guard.sent_chunks.contains(chunk) {
                drop(guard);
                sent_only_sessions.push(Arc::clone(session));
            }
        }

        // Queue updates for sessions still waiting on the chunk to flush.
        for session in sent_only_sessions.iter() {
            let mut guard = session.lock().unwrap();
            let q = guard.pending_block_changes.entry(*chunk).or_default();
            q.extend(block_changes.iter().map(|pb| PendingBlock {
                block: pb.block,
                block_pos: pb.block_pos,
                light: pb.light,
            }));
        }
        if flushed_sessions.is_empty() {
            continue;
        }

        // Capture chunk ref once for sub-region jobs.
        let chunk_ref = dim_world.get_chunk(*chunk);

        if block_changes.len() == 1 {
            // Single block change: serialise once, raw-copy to every session.
            let pb = &block_changes[0];
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
            // Serialise into a temporary buffer, then send to all sessions.
            let mut tmp_stream = NetworkStream::new_detached();
            sb.serialize(&mut tmp_stream);
            let buf = tmp_stream.get_raw_write_buffer();
            for session in flushed_sessions.iter() {
                session.lock().unwrap().stream.write_raw(buf);
            }
        } else if block_changes.len() < 10 {
            // Multi-block packet
            fn format_multi_block(x: i8, y: i8, z: i8) -> i16 {
                ((i16::from(x) & 0x0F) << 12) | ((i16::from(z) & 0x0F) << 8) | (i16::from(y) & 0xFF)
            }
            let mut smb = SetMultipleBlocks::new();
            smb.chunk_position = Int32_2::new(chunk.x, *chunk.z());
            for pb in block_changes.iter() {
                smb.block_coordinates.push(format_multi_block(
                    pb.block_pos.x as i8,
                    pb.block_pos.y as i8,
                    pb.block_pos.z as i8,
                ));
                smb.block_metadata.push(pb.block.data as i8);
                smb.block_types.push(pb.block.r#type);
            }
            smb.number_of_blocks = smb.block_coordinates.len() as i16;
            let mut tmp_stream = NetworkStream::new_detached();
            smb.serialize(&mut tmp_stream);
            let buf = tmp_stream.get_raw_write_buffer();
            for session in flushed_sessions.iter() {
                session.lock().unwrap().stream.write_raw(buf);
            }
        } else {
            // Sub-region: compression is async per-session via ChunkSender.
            for session in flushed_sessions.iter() {
                let mut session_guard = session.lock().unwrap();
                server.chunk_sender.send_block_updates(
                    &mut session_guard,
                    chunk,
                    block_changes,
                    chunk_ref.clone(),
                );
            }
        }
    }
}
