/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
use std::collections::{HashMap, HashSet};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use bpp_shared::base_types::{EntityId, NetworkSlotId, WindowId};
use bpp_shared::config::config::Config;
use bpp_shared::entities::entity::EntityBehavior;
use bpp_shared::entities::entity_manager::EntityManager;
use bpp_shared::enums::dimensions::Dimension;
use bpp_shared::enums::entities::EntityType;
use bpp_shared::enums::network::packet_ids::PacketId;
use bpp_shared::helpers::java::java_random::Random;
use bpp_shared::inventory::inventory_interaction::InventoryInteractionBehavior;
use bpp_shared::logger::logger::global_logger;
use bpp_shared::networking::packets::{ChatMessage, PacketBehavior, Respawn};
use bpp_shared::numeric_structs::{Int2, Int3, Int32_2, Int32_3};
use bpp_shared::runtime::Runtime;
use bpp_shared::world::chunk::{Chunk, ChunkState};
use bpp_shared::world::client_pos::ClientPosition;
use bpp_shared::world::world::{PendingBlock, WorldManager};

use crate::blocks::server_block_behaviors;
use crate::chunk_io::chunk_broadcaster;
use crate::chunk_io::chunk_sender::ChunkSender;
use crate::commands::command_manager::CommandManager;
use crate::entities::entity_tracker::EntityTracker;
use crate::packet::packet_dispatcher;
use crate::packet::packet_utils;
use crate::player_conn::player_session::{ConnectionState, PlayerSession};
use crate::player_conn::server_pconnstate_manager::PlayerConnStateManager;
use crate::server_socket;

pub static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

pub struct Server {
    pub game_runtime: Runtime,
    pub chunk_sender: ChunkSender,
    pub flush_chunk_count: i32,

    // Entity trackers are so we can send entity updates to players and vice versa.
    pub overworld_entity_tracker: Arc<Mutex<EntityTracker>>,
    pub hell_entity_tracker: Arc<Mutex<EntityTracker>>,

    pub(crate) conn_state_manager: PlayerConnStateManager,

    // Global players and dimensional players
    pub(crate) players: Vec<Arc<Mutex<PlayerSession>>>,

    // Block change tracking
    chunk_block_changes: Arc<Mutex<HashMap<Int32_2, Vec<PendingBlock>>>>,
    chunk_block_changes_hell: Arc<Mutex<HashMap<Int32_2, Vec<PendingBlock>>>>,

    // Which sessions currently have a given chunk loaded?
    pub(crate) chunk_sessions: HashMap<Int32_3, Vec<Arc<Mutex<PlayerSession>>>>,

    // Server specifics
    server_socket: Option<TcpListener>,
    server_port: u16,
    timeout_seconds: i64,
    command_manager: CommandManager,
    stopped: bool,
    config: Config,
}

impl Server {
    const TICKS_PER_SECOND: i32 = 20;
    const MAX_TICK_CATCH_UP: i32 = 5;

    pub fn new() -> Self {
        server_block_behaviors::initialize();

        let mut server = Self {
            game_runtime: Runtime::new(),
            chunk_sender: ChunkSender::default(),
            flush_chunk_count: 10,
            overworld_entity_tracker: Arc::new(Mutex::new(EntityTracker::default())),
            hell_entity_tracker: Arc::new(Mutex::new(EntityTracker::default())),
            conn_state_manager: PlayerConnStateManager::default(),
            players: Vec::new(),
            chunk_block_changes: Arc::new(Mutex::new(HashMap::new())),
            chunk_block_changes_hell: Arc::new(Mutex::new(HashMap::new())),
            chunk_sessions: HashMap::new(),
            server_socket: None,
            server_port: 25565,
            timeout_seconds: 60,
            command_manager: CommandManager,
            stopped: false,
            config: Config::new("server.properties"),
        };

        server.load_config();
        server.server_socket = server_socket::create_server_socket(server.server_port);
        if server.server_socket.is_none() {
            global_logger().error("**** FAILED TO CREATE SERVER SOCKET!\n");
            std::process::exit(1);
        }
        global_logger().info(format!("Server initialized on port {}\n", server.server_port));
        let level_name = server.config.get_as_string("level-name");
        let level_seed = server.config.get_as_string("level-seed");
        server.game_runtime.init(&level_name, &level_seed);
        server
    }

    // Send a message to all players
    pub fn send_global_chat_message(&self, message: &str) {
        self.send_global_chat_message_impl(message, None);
    }

    // Same as send_global_chat_message, but for use while the caller already holds the lock
    // for one of the sessions in self.players (e.g. from inside packet/command dispatch).
    // That session is passed in already unlocked so it isn't re-locked (which would deadlock).
    pub fn send_global_chat_message_from(&self, message: &str, already_locked: &mut PlayerSession) {
        self.send_global_chat_message_impl(message, Some(already_locked));
    }

    fn send_global_chat_message_impl(&self, message: &str, mut already_locked: Option<&mut PlayerSession>) {
        for other in &self.players {
            let mut reply = ChatMessage::new();
            reply.message = message.to_string();
            match other.try_lock() {
                Ok(mut guard) => {
                    if guard.conn_state != ConnectionState::Playing {
                        continue;
                    }
                    reply.serialize(&mut guard.stream);
                }
                Err(_) => {
                    if let Some(session) = already_locked.as_deref_mut() {
                        if session.conn_state != ConnectionState::Playing {
                            continue;
                        }
                        reply.serialize(&mut session.stream);
                    }
                }
            }
        }
    }

    pub fn get_players(&self) -> &Vec<Arc<Mutex<PlayerSession>>> {
        &self.players
    }

    pub fn get_session_by_id(&self, entity_id: EntityId) -> Arc<Mutex<PlayerSession>> {
        for player in &self.players {
            let matches = {
                let guard = player.lock().unwrap();
                match &guard.entity {
                    Some(entity) => entity.lock().unwrap().base().id == entity_id,
                    None => false,
                }
            };
            if matches {
                return Arc::clone(player);
            }
        }
        panic!("Session by Id not found");
    }

    pub fn get_session_by_username(&self, username: &str) -> Arc<Mutex<PlayerSession>> {
        for player in &self.players {
            if player.lock().unwrap().username == username {
                return Arc::clone(player);
            }
        }
        panic!("Session by Username not found");
    }

    pub fn get_username_by_entity_id(&self, id: EntityId) -> String {
        for player in &self.players {
            let guard = player.lock().unwrap();
            if let Some(entity) = &guard.entity {
                if entity.lock().unwrap().base().id == id {
                    return guard.username.clone();
                }
            }
        }
        panic!("Username by Entity Id not found");
    }

    pub fn send_entity_to_dimension(&mut self, dim: Dimension, entity: Arc<Mutex<dyn EntityBehavior + Send>>) {
        // Remove our entity from our watcher
        let old_dim = entity.lock().unwrap().base().dim;
        if old_dim == dim {
            return;
        }

        // Remove the entity from the world's entity managers
        let world = self.get_world_for_dimension(old_dim);
        let new_world = self.get_world_for_dimension(dim);

        let id = entity.lock().unwrap().base().id;
        world.lock().unwrap().entity_manager.remove_entity(id);

        // Rebind entity
        entity.lock().unwrap().base_mut().is_dead = false;
        new_world.lock().unwrap().entity_manager.add_entity(entity, EntityId(-1));
    }

    pub fn send_player_to_dimension(&mut self, dim: Dimension, session: &mut PlayerSession) {
        if dim as i8 == session.dimension {
            return;
        }

        // Flush all of our dimension dependent data
        session.dimension = dim as i8;
        session.flushed_chunks.clear();
        session.sent_chunks.clear();
        session.pending_block_changes.clear();
        session.newly_flushed.clear();
        session.newly_unloaded.clear();
        session.entity_tracker = if session.dimension == 0 {
            Arc::downgrade(&self.overworld_entity_tracker)
        } else {
            Arc::downgrade(&self.hell_entity_tracker)
        };

        // Make sure we don't send any pending chunk updates
        self.chunk_sender.remove(session);

        // Send a respawn packet
        let mut pkt = Respawn::new();
        pkt.dimension = dim;
        pkt.serialize(&mut session.stream);
        session.conn_state = ConnectionState::WaitingForSpawnChunks;
        let inventory = session.inventory.base.clone();
        packet_utils::send_inventory(session, WindowId(0), inventory);
        let entity = session.entity.clone();

        // Transfer our entity
        if let Some(entity) = entity {
            self.send_entity_to_dimension(dim, entity);
        }
    }

    fn index_add_chunk(&mut self, session: &Arc<Mutex<PlayerSession>>, pos: &Int32_2) {
        let dimension = session.lock().unwrap().dimension;
        let vec = self.chunk_sessions.entry(Server::chunk_key(pos, dimension)).or_default();
        // Avoid duplicates (should never happen, but be safe)
        if vec.iter().any(|existing| Arc::ptr_eq(existing, session)) {
            return;
        }
        vec.push(Arc::clone(session));
    }

    fn index_remove_chunk(&mut self, session: &Arc<Mutex<PlayerSession>>, pos: &Int32_2) {
        let dimension = session.lock().unwrap().dimension;
        let key = Server::chunk_key(pos, dimension);
        let should_remove_key = match self.chunk_sessions.get_mut(&key) {
            Some(vec) => {
                vec.retain(|existing| !Arc::ptr_eq(existing, session));
                vec.is_empty()
            }
            None => return,
        };
        if should_remove_key {
            self.chunk_sessions.remove(&key);
        }
    }

    fn index_remove_session(&mut self, session: &Arc<Mutex<PlayerSession>>) {
        let flushed_chunks: Vec<Int32_2> = session.lock().unwrap().flushed_chunks.iter().copied().collect();
        for pos in &flushed_chunks {
            self.index_remove_chunk(session, pos);
        }
    }

    fn load_config(&mut self) {
        if !self.config.load_from_disk() {
            let mut defaults: HashMap<String, String> = HashMap::new();
            defaults.insert("level-name".to_string(), "world".to_string());
            //defaults.insert("view-distance".to_string(), "10".to_string());
            //defaults.insert("white-list".to_string(), "false".to_string());
            //defaults.insert("server-ip".to_string(), String::new());
            //defaults.insert("motd".to_string(), "A Minecraft Server".to_string());
            //defaults.insert("pvp".to_string(), "true".to_string());
            // use a random device to seed another prng that gives us our seed
            defaults.insert("level-seed".to_string(), Random::new().next_long().to_string());
            //defaults.insert("spawn-animals".to_string(), "true".to_string());
            defaults.insert("server-port".to_string(), "25565".to_string());
            //defaults.insert("allow-nether".to_string(), "true".to_string());
            //defaults.insert("spawn-monsters".to_string(), "true".to_string());
            //defaults.insert("max-players".to_string(), "-1".to_string());
            //defaults.insert("online-mode".to_string(), "false".to_string());
            //defaults.insert("allow-flight".to_string(), "false".to_string());
            self.config.overwrite(defaults);
            self.config.save_to_disk();
        }
        //chunkDistance = self.config.get_as_number::<i32>("view-distance");
        self.server_port = self.config.get_as_number::<u16>("server-port");
        //motd = self.config.get_as_string("motd");
        //maximumPlayers = self.config.get_as_number::<i32>("max-players");
        //maximumThreads = self.config.get_as_number::<i32>("max-generator-threads");
        //whitelistEnabled = self.config.get_as_boolean("white-list");
    }

    fn startup(&mut self) {
        let startup_start = Instant::now();
        global_logger().info("Initializing server startup.. \n");

        // Setup commands
        CommandManager::init(self);

        // Setup the block callback so we can send it to clients
        {
            let mut world_guard = self.game_runtime.world.lock().unwrap();
            world_guard.on_block_update =
                Some(Box::new(Server::make_block_update_callback(0, Arc::clone(&self.chunk_block_changes))));
        }
        {
            let mut world_hell_guard = self.game_runtime.world_hell.lock().unwrap();
            world_hell_guard.on_block_update =
                Some(Box::new(Server::make_block_update_callback(-1, Arc::clone(&self.chunk_block_changes_hell))));
        }

        {
            let mut world_guard = self.game_runtime.world.lock().unwrap();
            Server::register_entity_tracker_callbacks(&self.overworld_entity_tracker, &mut world_guard.entity_manager);
        }
        {
            let mut world_hell_guard = self.game_runtime.world_hell.lock().unwrap();
            Server::register_entity_tracker_callbacks(&self.hell_entity_tracker, &mut world_hell_guard.entity_manager);
        }

        // Get spawn ready
        let spawn_chunk_distance: i32 = 9;
        let total_spawn_chunks =
            (spawn_chunk_distance + spawn_chunk_distance + 1) * (spawn_chunk_distance + spawn_chunk_distance + 1);

        let spawn_point = self.game_runtime.world.lock().unwrap().spawn_point;
        global_logger().info(format!("Server spawn is {}\n", Int2::new(spawn_point.x, spawn_point.z)));

        global_logger().info("Preparing spawn chunks..\n");
        // Push every single spawn chunk to get ready for generation
        let mut wanted: HashSet<Int32_2> = HashSet::new();
        for dx in -spawn_chunk_distance..=spawn_chunk_distance {
            for dz in -spawn_chunk_distance..=spawn_chunk_distance {
                let pos = Int32_2::new((spawn_point.x >> 4) + dx, (spawn_point.z >> 4) + dz);
                wanted.insert(pos);
            }
        }

        // Actually request chunks
        for pos in &wanted {
            {
                let mut world_guard = self.game_runtime.world.lock().unwrap();
                if !world_guard.chunks.contains_key(pos) {
                    let mut c = Chunk::default();
                    c.spawn_chunk = true;
                    c.cpos = *pos;
                    world_guard.chunks.insert(*pos, Arc::new(Mutex::new(c)));
                }
            }
            {
                let mut world_hell_guard = self.game_runtime.world_hell.lock().unwrap();
                if !world_hell_guard.chunks.contains_key(pos) {
                    let mut c = Chunk::default();
                    c.spawn_chunk = true;
                    c.cpos = *pos;
                    world_hell_guard.chunks.insert(*pos, Arc::new(Mutex::new(c)));
                }
            }
        }

        // Chunks are ready to load at this point.
        global_logger().info(format!("Loading spawn chunks for Overworld: ({total_spawn_chunks})\n"));
        Server::load_spawn_chunks(&self.game_runtime.world, total_spawn_chunks);

        global_logger().info(format!("Loading spawn chunks for Hell: ({total_spawn_chunks})\n"));
        Server::load_spawn_chunks(&self.game_runtime.world_hell, total_spawn_chunks);

        let startup_seconds = startup_start.elapsed().as_secs_f32();
        global_logger().info(format!("Startup Complete. ({startup_seconds:.4}s)\n"));
    }

    pub fn run(&mut self) {
        self.startup();

        let tick_duration = Duration::from_secs(1) / Self::TICKS_PER_SECOND as u32;

        let mut avg_total_tick_duration = Duration::from_nanos(0);
        let mut avg_tick_count: i32 = 0;

        let mut ticks: u64 = 0;
        let mut base_time = Instant::now();

        // Main tick loop
        // Heavily based on https://github.com/Minestom/Minestom/blob/59406d5b54d5221df85f381f204fbc07fd861a43/src/main/java/net/minestom/server/thread/TickSchedulerThread.java
        while !SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            let tick_start = Instant::now();
            self.tick();
            let tick_end = Instant::now();

            // Sample and print average tick data
            avg_total_tick_duration += tick_end - tick_start;
            avg_tick_count += 1;

            if ticks % (Self::TICKS_PER_SECOND as u64 * 2) == 0 {
                let avg_ms = avg_total_tick_duration.as_secs_f64() * 1000.0 / f64::from(avg_tick_count);
                global_logger().info(format!("Avg MSPT: {avg_ms} ms\n"));
                avg_total_tick_duration = Duration::from_nanos(0);
                avg_tick_count = 0;
            }

            ticks += 1;
            let next_tick_time = base_time + tick_duration * ticks as u32;
            thread::sleep(next_tick_time.saturating_duration_since(Instant::now()));

            // Check if the server can not keep up with the tickrate
            // if it gets too far behind, reset the ticks & baseTime
            // to avoid running too many ticks at once
            if Instant::now() > next_tick_time + tick_duration * Self::MAX_TICK_CATCH_UP as u32 {
                base_time = Instant::now();
                ticks = 0;
                global_logger().warn("Can't keep up with ticks!");
            }
        }

        // Shutdown was requested. Save and clean up on the main thread
        self.stop();
        SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst); // Unblock the ctrl handler thread
    }

    pub fn stop(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        global_logger().info("Server shutting down...\n");

        let players = self.players.clone();
        let conn_state_manager = std::mem::take(&mut self.conn_state_manager);
        for session in &players {
            conn_state_manager.disconnect_player(&mut session.lock().unwrap(), "Server Closed", self);
            let (saved_nbt, username) = {
                let mut guard = session.lock().unwrap();
                guard.stream.flush_write_buffer_blocking();
                (guard.serialize_to_nbt(), guard.username.clone())
            };
            self.game_runtime.save_manager.save_player_nbt(&username, &saved_nbt);
        }
        self.conn_state_manager = conn_state_manager;

        if let Some(server_socket) = self.server_socket.take() {
            server_socket::close_socket(server_socket);
        }
        self.game_runtime.world.lock().unwrap().shutdown();
        self.game_runtime.world_hell.lock().unwrap().shutdown();

        // Save our level file
        let mut cur_level_data = self.game_runtime.save_manager.get_level_data().clone();
        cur_level_data.random_seed = self.game_runtime.world.lock().unwrap().seed;
        cur_level_data.spawn_point = self.game_runtime.world.lock().unwrap().spawn_point;
        cur_level_data.time = self.game_runtime.world.lock().unwrap().elapsed_ticks;
        self.game_runtime.save_manager.save_level_file(&cur_level_data);
    }

    fn accept_new_players(&mut self) {
        let client_socket = self.server_socket.as_ref().and_then(server_socket::create_client_socket);
        let client_socket = match client_socket {
            Some(client_socket) => client_socket,
            None => return,
        };
        let session = PlayerSession::new(client_socket);
        self.players.push(Arc::new(Mutex::new(session)));
    }

    fn tick(&mut self) {
        self.accept_new_players();
        let players = self.players.clone();

        for session in &players {
            let is_playing = session.lock().unwrap().conn_state == ConnectionState::Playing;
            if is_playing {
                self.process_incoming(session);
            }
        }

        let mut overworld_positions: Vec<ClientPosition> = Vec::new();
        let mut nether_positions: Vec<ClientPosition> = Vec::new();
        for session in &players {
            let guard = session.lock().unwrap();
            if guard.conn_state == ConnectionState::WaitingForSpawnChunks || guard.conn_state == ConnectionState::Playing {
                let position = ClientPosition {
                    pos: guard.position.pos,
                    view_distance_override: guard.position.view_distance_override,
                };
                if guard.dimension == -1 {
                    nether_positions.push(position);
                } else {
                    overworld_positions.push(position);
                }
            }
        }
        self.game_runtime.world.lock().unwrap().tick(&overworld_positions);
        self.game_runtime.world.lock().unwrap().update(&overworld_positions);
        self.game_runtime.world_hell.lock().unwrap().tick(&nether_positions);
        self.game_runtime.world_hell.lock().unwrap().update(&nether_positions);

        // Send all of the block changes that have accumulated since the last tick, then clear the list.
        let mut local_block_changes = std::mem::take(&mut *self.chunk_block_changes.lock().unwrap());
        let mut local_block_changes_hell = std::mem::take(&mut *self.chunk_block_changes_hell.lock().unwrap());

        // Update the entity trackers
        self.overworld_entity_tracker.lock().unwrap().tick();
        self.hell_entity_tracker.lock().unwrap().tick();

        // Handle connection state for each player
        let conn_state_manager = std::mem::take(&mut self.conn_state_manager);
        for session in &players {
            conn_state_manager.handle_connection_state(&mut session.lock().unwrap(), self);

            // Drain chunk-session index updates that ChunkSender recorded.
            let newly_flushed: Vec<Int32_2> = session.lock().unwrap().newly_flushed.clone();
            for pos in &newly_flushed {
                self.index_add_chunk(session, pos);
            }
            session.lock().unwrap().newly_flushed.clear();
            let newly_unloaded: Vec<Int32_2> = session.lock().unwrap().newly_unloaded.clone();
            for pos in &newly_unloaded {
                self.index_remove_chunk(session, pos);
            }
            session.lock().unwrap().newly_unloaded.clear();

            // Check inventory diffs
            let diffs2 = {
                let mut guard = session.lock().unwrap();
                guard.with_own_interaction(&self.game_runtime, |i| i.tick_diff())
            };
            if diffs2.len() <= 5 {
                for difference in &diffs2 {
                    let mut guard = session.lock().unwrap();
                    packet_utils::send_slot(&mut guard, WindowId(0), NetworkSlotId(difference.slot as i16), Some(&difference.stack));
                }
            } else {
                // Too many changes, just resend the whole inventory
                let mut guard = session.lock().unwrap();
                let inventory = guard.inventory.base.clone();
                packet_utils::send_inventory(&mut guard, WindowId(0), inventory);
            }

            let has_active_interaction = session.lock().unwrap().has_active_container();
            if !has_active_interaction {
                continue;
            }

            let dimension = session.lock().unwrap().dimension;
            let world = if dimension == -1 { &self.game_runtime.world_hell } else { &self.game_runtime.world };
            let mut world_guard = world.lock().unwrap();

            // Force close windows that reference tile entities that have been deleted
            let can_exist = {
                let mut guard = session.lock().unwrap();
                guard
                    .with_active_interaction(&self.game_runtime, &mut world_guard, |i| i.can_exist())
                    .unwrap_or(false)
            };
            if !can_exist {
                packet_utils::close_container(&mut session.lock().unwrap());
                drop(world_guard);
                continue;
            }

            // Send each differing slot
            let diffs = {
                let mut guard = session.lock().unwrap();
                guard
                    .with_active_interaction(&self.game_runtime, &mut world_guard, |i| i.tick_diff())
                    .unwrap_or_default()
            };
            if diffs.len() <= 5 {
                for difference in &diffs {
                    let mut guard = session.lock().unwrap();
                    let window_id = guard.open_window_id;
                    packet_utils::send_slot(&mut guard, window_id, NetworkSlotId(difference.slot as i16), Some(&difference.stack));
                }
            } else {
                // Too many changes, just resend the whole inventory
                let mut guard = session.lock().unwrap();
                let window_id = guard.open_window_id;
                let inventory = guard.with_active_interaction(&self.game_runtime, &mut world_guard, |i| i.inventory().base().clone());
                if let Some(inventory) = inventory {
                    packet_utils::send_inventory(&mut guard, window_id, inventory);
                }
            }
            drop(world_guard);

            if self.game_runtime.world.lock().unwrap().elapsed_ticks % 40 == 0 {
                // Save periodically
                let mut guard = session.lock().unwrap();
                let saved_nbt = guard.serialize_to_nbt();
                let username = guard.username.clone();
                drop(guard);
                self.game_runtime.save_manager.save_player_nbt(&username, &saved_nbt);
            }
        }
        self.conn_state_manager = conn_state_manager;

        // Dispatch block changes.
        {
            let world = Arc::clone(&self.game_runtime.world);
            let mut world_guard = world.lock().unwrap();
            chunk_broadcaster::broadcast_block_changes(self, &mut local_block_changes, 0, &mut world_guard);
        }
        {
            let world_hell = Arc::clone(&self.game_runtime.world_hell);
            let mut world_hell_guard = world_hell.lock().unwrap();
            chunk_broadcaster::broadcast_block_changes(self, &mut local_block_changes_hell, -1, &mut world_hell_guard);
        }

        // Flush all pending outgoing data to the socket once per tick.
        for session in &players {
            session.lock().unwrap().stream.flush_write_buffer();
        }
        self.disconnect_clients();
    }

    fn disconnect_clients(&mut self) {
        // Mark clients who have timed out for removal
        let now = Instant::now();
        let players = self.players.clone();
        let conn_state_manager = std::mem::take(&mut self.conn_state_manager);
        for session in &players {
            let (conn_state, elapsed, username) = {
                let guard = session.lock().unwrap();
                (guard.conn_state, now.duration_since(guard.last_packet_time).as_secs() as i64, guard.username.clone())
            };
            if conn_state == ConnectionState::Playing {
                if elapsed > self.timeout_seconds {
                    global_logger().info(format!("Player {username} timed out\n"));
                    conn_state_manager.disconnect_player(&mut session.lock().unwrap(), "Connection timed out.", self);
                    self.send_global_chat_message(&format!("§e{username} left the game."));
                }
            } else {
                // Kill stuck handshakers
                if elapsed > self.timeout_seconds {
                    session.lock().unwrap().stream.set_connected(false);
                    global_logger().info("Disconnected dataless stream. (Most likely a prober!)\n");
                }
            }
        }
        self.conn_state_manager = conn_state_manager;

        // Force disconnect players that quit
        let mut still_connected: Vec<Arc<Mutex<PlayerSession>>> = Vec::new();
        for session in players {
            let is_connected = session.lock().unwrap().stream.is_connected();
            if !is_connected {
                let (entity_id, username, conn_state) = {
                    let guard = session.lock().unwrap();
                    (guard.entity.as_ref().map(|entity| entity.lock().unwrap().base().id), guard.username.clone(), guard.conn_state)
                };
                if let Some(entity_id) = entity_id {
                    global_logger().info(format!("Disconnected client {username} with entity id {}\n", entity_id.0));
                }

                if conn_state == ConnectionState::Playing || conn_state == ConnectionState::WaitingForSpawnChunks {
                    let mut guard = session.lock().unwrap();
                    let saved_nbt = guard.serialize_to_nbt();
                    drop(guard);
                    self.game_runtime.save_manager.save_player_nbt(&username, &saved_nbt);
                }

                self.index_remove_session(&session);
                {
                    let guard = session.lock().unwrap();
                    self.chunk_sender.remove(&guard);
                }
                self.send_global_chat_message(&format!("§e{username} left the game."));
            } else {
                still_connected.push(session);
            }
        }
        self.players = still_connected;
    }

    pub fn transfer_player_dimension(&mut self, _session: &mut PlayerSession) {}

    pub fn process_incoming(&mut self, session: &Arc<Mutex<PlayerSession>>) {
        let dimension = session.lock().unwrap().dimension;
        let session_world = if dimension == -1 { Arc::clone(&self.game_runtime.world_hell) } else { Arc::clone(&self.game_runtime.world) };

        loop {
            let has_data = session.lock().unwrap().stream.has_data();
            if !has_data {
                break;
            }
            let packet_id = PacketId(session.lock().unwrap().stream.read_u8());

            let dispatched = {
                let mut world_guard = session_world.lock().unwrap();
                let mut session_guard = session.lock().unwrap();
                packet_dispatcher::dispatch(packet_id, &mut session_guard, &mut world_guard, self)
            };

            if !dispatched {
                return; // session is dead, or sent an unknown packet
            }

            let short_read = session.lock().unwrap().stream.check_and_clear_short_read();
            if short_read {
                break;
            }
        }
        // Update our last packet time for the timeout code
        session.lock().unwrap().last_packet_time = Instant::now();
    }

    fn get_world_for_dimension(&self, dim: Dimension) -> Arc<Mutex<WorldManager>> {
        if dim == Dimension::Nether { Arc::clone(&self.game_runtime.world_hell) } else { Arc::clone(&self.game_runtime.world) }
    }

    // Encodes chunk position + dimension into a single key for chunkSessions.
    // x = chunk X, y = chunk Z, z = dimension id
    pub(crate) fn chunk_key(pos: &Int32_2, dimension: i8) -> Int32_3 {
        Int32_3::new(pos.x, *pos.z(), i32::from(dimension))
    }

    fn make_block_update_callback(
        _dimension_id: i8,
        block_change_map: Arc<Mutex<HashMap<Int32_2, Vec<PendingBlock>>>>,
    ) -> impl FnMut(PendingBlock, Int32_2) + Send {
        move |pending_block: PendingBlock, chunk_pos: Int32_2| {
            let pending_new = PendingBlock {
                block: pending_block.block,
                block_pos: Int3::new(pending_block.block_pos.x & 15, pending_block.block_pos.y, pending_block.block_pos.z & 15),
                light: pending_block.light,
            };
            block_change_map.lock().unwrap().entry(chunk_pos).or_default().push(pending_new);
        }
    }

    fn register_entity_tracker_callbacks(entity_tracker: &Arc<Mutex<EntityTracker>>, entity_manager: &mut EntityManager) {
        let spawn_tracker = Arc::clone(entity_tracker);
        entity_manager.on_entity_spawn = Some(Box::new(move |entity: Arc<Mutex<dyn EntityBehavior + Send>>| {
            let is_player = entity.lock().unwrap().base().r#type == EntityType::Player;
            if is_player {
                spawn_tracker.lock().unwrap().add_player(&entity);
                return;
            }
            spawn_tracker.lock().unwrap().track_entity(&entity);
        }));

        let despawn_tracker = Arc::clone(entity_tracker);
        entity_manager.on_entity_despawn = Some(Box::new(move |entity: Arc<Mutex<dyn EntityBehavior + Send>>| {
            let is_player = entity.lock().unwrap().base().r#type == EntityType::Player;
            if is_player {
                despawn_tracker.lock().unwrap().remove_player(&entity);
                return;
            }
            despawn_tracker.lock().unwrap().untrack_entity(&entity);
        }));
    }

    // Chunks are ready to load at this point.
    fn load_spawn_chunks(world: &Arc<Mutex<WorldManager>>, total_spawn_chunks: i32) {
        let spawn_chunk_distance: i32 = 9;
        let mut start = Instant::now();
        loop {
            let mut loaded_chunks = 0;
            // Force gen these chunks AS FAST AS POSSIBLE
            {
                let mut world_guard = world.lock().unwrap();
                world_guard.pump_pipeline(&[]);
                world_guard.pool.wait();
                world_guard.drain_gen_queue();
                let region_manager = world_guard.region_manager.clone();
                if let Some(region_manager) = region_manager {
                    region_manager.lock().unwrap().iopool.wait();
                }
                world_guard.drain_load_queue();
                world_guard.populate_ready();
                let mut light_manager = std::mem::take(&mut world_guard.light_manager);
                light_manager.process_light_queue(&mut world_guard, i32::MAX);
                world_guard.light_manager = light_manager;
            }

            {
                let world_guard = world.lock().unwrap();
                for dx in -spawn_chunk_distance..=spawn_chunk_distance {
                    for dz in -spawn_chunk_distance..=spawn_chunk_distance {
                        let p = Int32_2::new((world_guard.spawn_point.x >> 4) + dx, (world_guard.spawn_point.z >> 4) + dz);
                        if let Some(chunk) = world_guard.chunks.get(&p) {
                            if chunk.lock().unwrap().state_load() >= ChunkState::Generated {
                                loaded_chunks += 1;
                            }
                        }
                    }
                }
            }

            // Update load percentage every second
            if start.elapsed().as_secs_f32() >= 1.0 {
                let percent_loaded = ((loaded_chunks as f32 / total_spawn_chunks as f32) * 100.0) as i32;
                global_logger().info(format!("Loading spawn.. {percent_loaded}%\n"));
                start = Instant::now();
            }

            // Have we loaded all the spawn chunks?
            if loaded_chunks >= total_spawn_chunks {
                break;
            }
        }
        global_logger().info("Loading spawn.. 100%\n");
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}
