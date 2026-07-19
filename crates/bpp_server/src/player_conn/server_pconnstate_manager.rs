/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
use std::sync::{Arc, Mutex};

use bpp_shared::base_types::WindowId;
use bpp_shared::constants::PLAYER_EYE_HEIGHT;
use bpp_shared::enums::dimensions::Dimension;
use bpp_shared::enums::network::packet_ids;
use bpp_shared::enums::network::packet_ids::PacketId;
use bpp_shared::helpers::cross_platform::Math;
use bpp_shared::inventory::inventory_interaction::InventoryInteractionBehavior;
use bpp_shared::logger::logger::global_logger;
use bpp_shared::networking::packets::{
    ChatMessage, Disconnect, Login, PacketBehavior, PlayerPositionAndRotation, PreLogin, SetHealth,
    SetSpawnPosition, SetTime,
};
use bpp_shared::numeric_structs::{Int32_2, Vec3};
use bpp_shared::version;

use crate::entities::entity_mp_player::EntityMPPlayer;
use crate::packet::packet_utils;
use crate::player_conn::player_session::{ConnectionState, PlayerSession};
use crate::server::Server;

// For managing the player's connection state
#[derive(Default)]
pub struct PlayerConnStateManager;

impl PlayerConnStateManager {
    pub fn handle_connection_state(&self, session: &mut PlayerSession, server: &mut Server) {
        match session.conn_state {
            ConnectionState::Handshaking => self.handle_handshake(session, server),
            ConnectionState::LoggingIn => self.handle_login(session, server),
            ConnectionState::WaitingForSpawnChunks => self.wait_for_spawn_chunks(session, server),
            ConnectionState::Playing => {
                let mut world_guard = if session.dimension == -1 {
                    server.game_runtime.world_hell.lock().unwrap()
                } else {
                    server.game_runtime.world.lock().unwrap()
                };
                server.chunk_sender.enqueue(session, &mut world_guard, 16);
                server.chunk_sender.flush(session);
                if world_guard.elapsed_ticks % 20 == 0 {
                    // Update the server time so client's don't desync
                    let mut time = SetTime::new();
                    time.time = world_guard.elapsed_ticks;
                    time.serialize(&mut session.stream);
                }
            }
        }
    }

    pub fn handle_handshake(&self, session: &mut PlayerSession, _server: &mut Server) {
        if !session.stream.has_data() {
            return;
        }
        let packet_id = PacketId(session.stream.read_u8());

        if session.stream.check_and_clear_short_read() {
            return;
        }
        if packet_id != packet_ids::PRE_LOGIN {
            return;
        }

        let mut incoming = PreLogin::new();
        incoming.deserialize(&mut session.stream);
        if session.stream.check_and_clear_short_read() {
            return;
        }
        session.username = incoming.username;

        let mut response = PreLogin::new();
        response.username = "-".to_string();
        response.serialize(&mut session.stream);

        global_logger().info(format!("Player {} is logging in.\n", session.username));

        session.conn_state = ConnectionState::LoggingIn;
    }

    pub fn handle_login(&self, session: &mut PlayerSession, server: &mut Server) {
        if !session.stream.has_data() {
            return;
        }

        let packet_id = PacketId(session.stream.read_u8());
        if session.stream.check_and_clear_short_read() {
            return;
        }
        if packet_id != packet_ids::LOGIN {
            return;
        }

        let mut incoming = Login::new();
        incoming.deserialize(&mut session.stream);
        if session.stream.check_and_clear_short_read() {
            return;
        }

        // Load player data before building the Login response so we know which dimension they're in
        let player_nbt = server.game_runtime.save_manager.get_player_nbt(&session.username);
        session.load_player_nbt(&player_nbt);

        let dimension = if session.dimension == -1 { Dimension::Nether } else { Dimension::Overworld };

        // Initialize our entity
        if session.entity.is_none() {
            session.entity = Some(Arc::new(Mutex::new(EntityMPPlayer::new())));
        }
        let entity_arc = session.entity.as_ref().unwrap().clone();
        let entity_id;
        {
            let mut world_guard = if session.dimension == -1 {
                server.game_runtime.world_hell.lock().unwrap()
            } else {
                server.game_runtime.world.lock().unwrap()
            };
            let mut entity_guard = entity_arc.lock().unwrap();
            entity_guard.base.base.id = world_guard.entity_manager.get_next_entity_id();
            entity_guard.base.base.dim = dimension;
            entity_id = entity_guard.base.base.id;
        }

        let mut response = Login::new();
        response.entity_id = entity_id;
        response.username = session.username.clone();
        response.world_seed = server.game_runtime.world.lock().unwrap().seed;
        response.dimension = dimension;
        response.serialize(&mut session.stream);

        let mut spawn = SetSpawnPosition::new();
        {
            let world_guard = if session.dimension == -1 {
                server.game_runtime.world_hell.lock().unwrap()
            } else {
                server.game_runtime.world.lock().unwrap()
            };
            spawn.position = world_guard.spawn_point;
        }
        spawn.serialize(&mut session.stream);

        let mut health = SetHealth::new();
        health.health = 20;
        health.serialize(&mut session.stream);

        let mut time = SetTime::new();
        {
            let world_guard = if session.dimension == -1 {
                server.game_runtime.world_hell.lock().unwrap()
            } else {
                server.game_runtime.world.lock().unwrap()
            };
            time.time = world_guard.elapsed_ticks;
        }
        time.serialize(&mut session.stream);

        // Get a fresh respawn point
        let respawn_point = {
            let mut world_guard = if session.dimension == -1 {
                server.game_runtime.world_hell.lock().unwrap()
            } else {
                server.game_runtime.world.lock().unwrap()
            };
            world_guard.get_spawn_point(true)
        };

        // If our session position is the default then overwrite it
        if session.position.pos == Vec3::new(-1.0, -1000000.0, -1.0) {
            session.position.pos = Vec3::new(
                (respawn_point.x as f32) as f64 + 0.5,
                (respawn_point.y as f32) as f64,
                (respawn_point.z as f32) as f64 + 0.5,
            );
        }

        // Convert the feet-based respawn height into our posY convention (eye level)
        session.position.pos.y += PLAYER_EYE_HEIGHT + 0.00001;

        // Log that we logged in!
        global_logger().info(format!(
            "Player {} logged in with entity ID {} at ({}, {}, {})\n",
            session.username,
            entity_id.value(),
            session.position.pos.x,
            session.position.pos.y,
            session.position.pos.z
        ));

        // Let everyone else know we logged in
        server.send_global_chat_message_from(&format!("§e{} joined the game.", session.username), session);

        // Send our inventory
        let inventory = session.inventory.base.clone();
        packet_utils::send_inventory(session, WindowId(0), inventory);

        // Snapshot current contents so the tick loop's diffing (tickDiff) has a real baseline
        // to compare against, instead of starting from an empty snapshot for the whole session.
        session.with_own_interaction(&server.game_runtime, |i| i.init_snapshot());

        session.conn_state = ConnectionState::WaitingForSpawnChunks;
    }

    pub fn disconnect_player(&self, session: &mut PlayerSession, reason: &str, _server: &mut Server) {
        // Send disconnect reason to the leaving player
        let mut kick = Disconnect::new();
        kick.reason = reason.to_string();
        kick.serialize(&mut session.stream);
        session.stream.set_connected(false); // This should force an NBT save
        global_logger().info(format!("Player {} disconnected: {reason}\n", session.username));
    }

    pub fn wait_for_spawn_chunks(&self, session: &mut PlayerSession, server: &mut Server) {
        let radius;
        {
            let mut world_guard = if session.dimension == -1 {
                server.game_runtime.world_hell.lock().unwrap()
            } else {
                server.game_runtime.world.lock().unwrap()
            };
            server.chunk_sender.enqueue(session, &mut world_guard, server.flush_chunk_count);
            radius = Math::min(3, world_guard.get_view_radius());
        }
        server.chunk_sender.flush(session);

        // Force a tiny view distance for players trying to spawn in
        session.position.view_distance_override = 3;

        // Spawn chunk radius; 3 chunks in each direction
        let spawn_chunk_x = (session.position.pos.x.floor() as i32) >> 4;
        let spawn_chunk_z = (session.position.pos.z.floor() as i32) >> 4;

        let total_spawn_chunks = ((radius * 2) + 1) * ((radius * 2) + 1);
        let mut loaded_chunks = 0;

        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let p = Int32_2::new(spawn_chunk_x + dx, spawn_chunk_z + dz);
                if session.flushed_chunks.contains(&p) {
                    loaded_chunks += 1;
                }
            }
        }

        global_logger().info(format!("Spawn chunks: {loaded_chunks} / {total_spawn_chunks}\n"));

        if loaded_chunks < total_spawn_chunks {
            return;
        }

        global_logger().info("Spawn chunks sent. Setting player position\n");

        session.position.pos.y += 0.0625;
        let mut pos = PlayerPositionAndRotation::new();
        pos.position = session.position.pos;
        pos.camera_y = session.position.pos.y + PLAYER_EYE_HEIGHT;
        pos.rotation = session.rotation;
        pos.on_ground = false;
        pos.serialize(&mut session.stream);

        // Set view distance to server default
        session.position.view_distance_override = 0;

        global_logger().info("Client connected\n");
        session.conn_state = ConnectionState::Playing;

        // Register our entity with the world
        if !session.entity_registered {
            let entity_arc = session.entity.as_ref().unwrap().clone();
            let entity_id = entity_arc.lock().unwrap().base.base.id;
            let mut world_guard = if session.dimension == -1 {
                server.game_runtime.world_hell.lock().unwrap()
            } else {
                server.game_runtime.world.lock().unwrap()
            };
            world_guard.entity_manager.add_entity(entity_arc, entity_id);
        }
        session.entity_registered = true;

        // Give our player session a pointer to the entity tracker
        session.entity_tracker = if session.dimension == 0 {
            Arc::downgrade(&server.overworld_entity_tracker)
        } else {
            Arc::downgrade(&server.hell_entity_tracker)
        };

        // Welcome message
        let mut welcome_msg = ChatMessage::new();
        welcome_msg.message = format!("§eThis Server runs on {}", version::PROJECT_FULL_VERSION_LABEL);
        welcome_msg.serialize(&mut session.stream);
    }
}
