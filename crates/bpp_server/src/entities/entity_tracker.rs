/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, Weak};

use bpp_shared::base_types::{EntityId, TickTime};
use bpp_shared::entities::entity::{Entity, EntityBehavior};
use bpp_shared::entities::entity_item::ItemEntity;
use bpp_shared::enums::entities::EntityType;
use bpp_shared::enums::items;
use bpp_shared::helpers::java::java_math::MathHelper;
use bpp_shared::logger::logger::global_logger;
use bpp_shared::networking::packets::{
    DespawnEntity, EntityPosition, EntityPositionAndRotation, EntityRotation, EntityVelocity,
    PacketBehavior, SpawnItem, SpawnPlayer, TeleportEntity,
};
use bpp_shared::numeric_structs::{Int8_2, Int8_3, Int16_3, Int32_3, Vec3};

use crate::server::Server;

// Entity tracker so we can send entity updates to the right players. This is server side only annoyingly enough.
// I am not entirely happy with how this is done but notch demands we have several packet types for each type of entity
pub struct TrackingProfile {
    pub range: i32,
    pub update_frequency: i32, // ticks between movement-sync packets
    pub send_velocity: bool,
}

impl Default for TrackingProfile {
    fn default() -> Self {
        Self {
            range: 0,
            update_frequency: 0,
            send_velocity: false,
        }
    }
}

pub struct TrackedEntry {
    pub entity: Weak<Mutex<dyn EntityBehavior + Send>>,
    pub profile: TrackingProfile,
    pub last_encoded_pos: Int32_3,
    pub last_broadcast_motion: Vec3,
    pub last_encoded_yaw: i32,
    pub last_encoded_pitch: i32,
    pub update_counter: i32,
    pub ticks_since_teleport: i32,
    pub visible_to: HashSet<EntityId>, // what player ids can see this entity
}

impl Default for TrackedEntry {
    fn default() -> Self {
        Self {
            entity: Weak::<Mutex<ItemEntity>>::new(),
            profile: TrackingProfile::default(),
            last_encoded_pos: Int32_3::default(),
            last_broadcast_motion: Vec3::default(),
            last_encoded_yaw: 0,
            last_encoded_pitch: 0,
            update_counter: 0,
            ticks_since_teleport: 0,
            visible_to: HashSet::new(),
        }
    }
}

pub struct EntityTracker {
    pub server: Weak<Mutex<Server>>,

    pub tracked_entities: HashMap<EntityId, TrackedEntry>,
    pub player_ids: HashSet<EntityId>,

    pub force_teleport_ticks: TickTime, // 20 seconds
}

impl EntityTracker {
    pub fn new() -> Self {
        Self {
            server: Weak::new(),
            tracked_entities: HashMap::new(),
            player_ids: HashSet::new(),
            force_teleport_ticks: 400,
        }
    }

    // Update each player instance so entities properly despawn and spawn for them
    pub fn tick(&mut self) {
        let server_weak = self.server.clone();
        let force_teleport_ticks = self.force_teleport_ticks;

        let mut dead_this_tick: Vec<EntityId> = Vec::new();

        for (&entity_id, entry) in self.tracked_entities.iter() {
            let is_dead = match entry.entity.upgrade() {
                Some(entity) => entity.lock().unwrap().base().is_dead,
                None => true,
            };
            if is_dead {
                dead_this_tick.push(entity_id);
            }
        }

        for entity_id in dead_this_tick {
            let visible_to = self
                .tracked_entities
                .get(&entity_id)
                .expect("Entity not found")
                .visible_to
                .clone();
            for viewer_id in visible_to {
                let session = {
                    let server = server_weak.upgrade().expect("server dropped");
                    let server_guard = server.lock().unwrap();
                    server_guard.get_session_by_id(viewer_id)
                };
                let mut pkt = DespawnEntity::new();
                pkt.entity_id = entity_id;
                pkt.serialize(&mut session.lock().unwrap().stream);
            }
            for other_entry in self.tracked_entities.values_mut() {
                other_entry.visible_to.remove(&entity_id);
            }
            self.tracked_entities.remove(&entity_id);
            self.player_ids.remove(&entity_id);
        }

        // Despawn pass / update
        for entry in self.tracked_entities.values_mut() {
            Self::update(&server_weak, force_teleport_ticks, entry); // Determine what packets to send
        }

        let positions: HashMap<EntityId, (f64, f64)> = self
            .tracked_entities
            .iter()
            .filter_map(|(&id, entry)| {
                entry.entity.upgrade().map(|entity| {
                    let guard = entity.lock().unwrap();
                    let base = guard.base();
                    (id, (base.pos_x, base.pos_z))
                })
            })
            .collect();

        for (&entity_id, entry) in self.tracked_entities.iter_mut() {
            let (entry_x, entry_z) = match positions.get(&entity_id) {
                Some(&pos) => pos,
                None => continue,
            };
            let range = f64::from(entry.profile.range);

            let mut to_despawn: Vec<EntityId> = Vec::new();
            entry.visible_to.retain(|&player_id| {
                let (player_x, player_z) = match positions.get(&player_id) {
                    Some(&pos) => pos,
                    None => return false,
                };

                let distance_to = f64::abs(f64::max(
                    f64::abs(entry_x - player_x),
                    f64::abs(entry_z - player_z),
                ));
                if distance_to > range {
                    to_despawn.push(player_id);
                    false
                } else {
                    true
                }
            });

            for player_id in to_despawn {
                let session = {
                    let server = server_weak.upgrade().expect("server dropped");
                    let server_guard = server.lock().unwrap();
                    server_guard.get_session_by_id(player_id)
                };
                let mut pkt = DespawnEntity::new();
                pkt.entity_id = entity_id;
                pkt.serialize(&mut session.lock().unwrap().stream);
            }
        }

        // Spawn pass
        let player_ids: Vec<EntityId> = self.player_ids.iter().copied().collect();
        for player_id in player_ids {
            let player_entry = self
                .tracked_entities
                .get(&player_id)
                .expect("Entity not found");
            let player_entity = player_entry.entity.upgrade().expect("entity dropped");
            let player_pos = {
                let guard = player_entity.lock().unwrap();
                let base = guard.base();
                (base.pos_x, base.pos_z)
            };

            let entity_ids: Vec<EntityId> = self.tracked_entities.keys().copied().collect();
            for entity_id in entity_ids {
                if entity_id == player_id {
                    continue;
                }

                let (entity_pos_x, entity_pos_z, entity_type, range, already_visible) = {
                    let entry = match self.tracked_entities.get(&entity_id) {
                        Some(entry) => entry,
                        None => continue,
                    };
                    let entity_arc = match entry.entity.upgrade() {
                        Some(entity) => entity,
                        None => continue,
                    };
                    let guard = entity_arc.lock().unwrap();
                    let base = guard.base();
                    (
                        base.pos_x,
                        base.pos_z,
                        base.r#type,
                        entry.profile.range,
                        entry.visible_to.contains(&player_id),
                    )
                };

                let distance_to = f64::abs(f64::max(
                    f64::abs(entity_pos_x - player_pos.0),
                    f64::abs(entity_pos_z - player_pos.1),
                ));
                if distance_to > f64::from(range) || already_visible {
                    continue;
                }

                let session = {
                    let server = server_weak.upgrade().expect("server dropped");
                    let server_guard = server.lock().unwrap();
                    server_guard.get_session_by_id(player_id)
                };

                match entity_type {
                    EntityType::Item => {
                        let entry = self
                            .tracked_entities
                            .get(&entity_id)
                            .expect("Entity not found");
                        let entity_arc = entry.entity.upgrade().expect("entity dropped");
                        let guard = entity_arc.lock().unwrap();
                        let base = guard.base();
                        let item_entity = guard
                            .as_any()
                            .downcast_ref::<ItemEntity>()
                            .expect("EntityType::Item entity was not an ItemEntity");

                        let mut pkt = SpawnItem::new();
                        pkt.entity_id = base.id;
                        pkt.item = item_entity.item_stack;
                        pkt.q_position = Int32_3::new(
                            Self::quantize_position(base.pos_x),
                            Self::quantize_position(base.pos_y),
                            Self::quantize_position(base.pos_z),
                        );
                        // For some reason notch decided this should be a convoluted way of getting the initial spawn velocity
                        let quantize_spawn_velocity = |v: f64| -> i8 { (v * 128.0) as i8 };
                        pkt.q_rotation = Int8_3::new(
                            quantize_spawn_velocity(base.motion_x),
                            quantize_spawn_velocity(base.motion_y),
                            quantize_spawn_velocity(base.motion_z),
                        );
                        drop(guard);
                        pkt.serialize(&mut session.lock().unwrap().stream);
                    }
                    EntityType::Player => {
                        let entry = self
                            .tracked_entities
                            .get(&entity_id)
                            .expect("Entity not found");
                        let entity_arc = entry.entity.upgrade().expect("entity dropped");
                        let guard = entity_arc.lock().unwrap();
                        let base = guard.base();

                        let mut pkt = SpawnPlayer::new();
                        pkt.entity_id = base.id;
                        pkt.held_item_id = items::NONE;
                        pkt.q_position = Int32_3::new(
                            Self::quantize_position(base.pos_x),
                            Self::quantize_position(base.pos_y),
                            Self::quantize_position(base.pos_z),
                        );
                        pkt.q_rotation = Int8_2::new(
                            Self::quantize_rotation(base.rotation_yaw) as i8,
                            Self::quantize_rotation(base.rotation_pitch) as i8,
                        );
                        let id = base.id;
                        drop(guard);

                        // To prevent bad behavior when we share a name with another entity
                        let username = {
                            let server = server_weak.upgrade().expect("server dropped");
                            let server_guard = server.lock().unwrap();
                            server_guard.get_username_by_entity_id(id)
                        };
                        pkt.username = username;
                        pkt.serialize(&mut session.lock().unwrap().stream);
                    }
                    _ => {}
                }
                // TODO: Implement other types
                if let Some(entry) = self.tracked_entities.get_mut(&entity_id) {
                    entry.visible_to.insert(player_id);
                }
            }
        }
    }

    pub fn quantize_velocity(v: f64) -> i16 {
        let clamp = 3.9;
        let mut v = v;
        if v < -clamp {
            v = -clamp;
        }
        if v > clamp {
            v = clamp;
        }
        (v * 8000.0) as i16
    }

    pub fn quantize_position(p: f64) -> i32 {
        MathHelper::floor_double(p * 32.0)
    }

    pub fn quantize_rotation(r: f32) -> i32 {
        MathHelper::floor_float(r * 256.0 / 360.0)
    }

    pub fn track_entity(&mut self, entity: &Arc<Mutex<dyn EntityBehavior + Send>>) {
        let (id, entry) = {
            let guard = entity.lock().unwrap();
            let base = guard.base();
            let profile = self.get_tracking_profile(base);
            let current_motion = Vec3::new(base.motion_x, base.motion_y, base.motion_z);

            let entry = TrackedEntry {
                entity: Arc::downgrade(entity),
                profile,
                last_encoded_pos: Int32_3::new(
                    Self::quantize_position(base.pos_x),
                    Self::quantize_position(base.pos_y),
                    Self::quantize_position(base.pos_z),
                ),
                last_broadcast_motion: current_motion,
                last_encoded_pitch: Self::quantize_rotation(base.rotation_pitch),
                last_encoded_yaw: Self::quantize_rotation(base.rotation_yaw),
                ..TrackedEntry::default()
            };
            (base.id, entry)
        };

        self.tracked_entities.insert(id, entry);
        self.tick();
    }

    pub fn untrack_entity(&mut self, _entity: &Arc<Mutex<dyn EntityBehavior + Send>>) {
        self.tick();
    }

    pub fn add_player(&mut self, player: &Arc<Mutex<dyn EntityBehavior + Send>>) {
        let (id, entry) = {
            let guard = player.lock().unwrap();
            let base = guard.base();
            let profile = self.get_tracking_profile(base);
            let current_motion = Vec3::new(base.motion_x, base.motion_y, base.motion_z);

            let entry = TrackedEntry {
                entity: Arc::downgrade(player),
                profile,
                last_encoded_pos: Int32_3::new(
                    Self::quantize_position(base.pos_x),
                    Self::quantize_position(base.pos_y),
                    Self::quantize_position(base.pos_z),
                ),
                last_broadcast_motion: current_motion,
                last_encoded_pitch: Self::quantize_rotation(base.rotation_pitch),
                last_encoded_yaw: Self::quantize_rotation(base.rotation_yaw),
                ..TrackedEntry::default()
            };
            (base.id, entry)
        };

        self.tracked_entities.insert(id, entry);
        self.player_ids.insert(id);
        self.tick();
    }

    pub fn remove_player(&mut self, _player: &Arc<Mutex<dyn EntityBehavior + Send>>) {
        self.tick();
    }

    pub fn send_packet_to_players_in_tracked_entry(
        server: &Weak<Mutex<Server>>,
        pkt: &dyn PacketBehavior,
        tracked_entry: &TrackedEntry,
    ) {
        for &player_id in &tracked_entry.visible_to {
            let session = {
                let server = server.upgrade().expect("server dropped");
                let server_guard = server.lock().unwrap();
                server_guard.get_session_by_id(player_id)
            };
            pkt.serialize(&mut session.lock().unwrap().stream);
        }
    }

    pub fn send_packet_to_viewers(&mut self, pkt: &dyn PacketBehavior, id: EntityId) {
        let player_ids: Vec<EntityId> = self.player_ids.iter().copied().collect();
        for player_id in player_ids {
            let visible = self
                .get_tracker_for_entity_id(player_id)
                .visible_to
                .contains(&id);
            if visible {
                let session = {
                    let server = self.server.upgrade().expect("server dropped");
                    let server_guard = server.lock().unwrap();
                    server_guard.get_session_by_id(player_id)
                };
                pkt.serialize(&mut session.lock().unwrap().stream);
            }
        }
    }

    pub fn get_tracker_for_entity_id(&mut self, id: EntityId) -> &mut TrackedEntry {
        self.tracked_entities
            .get_mut(&id)
            .expect("Entity not found")
    }

    pub fn update(
        server: &Weak<Mutex<Server>>,
        force_teleport_ticks: TickTime,
        tracked_entry: &mut TrackedEntry,
    ) {
        let entity = tracked_entry.entity.upgrade().expect("entity dropped");
        let current_position = {
            let guard = entity.lock().unwrap();
            let base = guard.base();
            Vec3::new(base.pos_x, base.pos_y, base.pos_z)
        };

        // Dirty flag gets checked every tick
        let velocity_changed = entity.lock().unwrap().base().velocity_changed;
        if velocity_changed {
            entity.lock().unwrap().base_mut().velocity_changed = false;
            let motion = {
                let guard = entity.lock().unwrap();
                let base = guard.base();
                Vec3::new(base.motion_x, base.motion_y, base.motion_z)
            };
            tracked_entry.last_broadcast_motion = motion;
            let id = entity.lock().unwrap().base().id;
            let mut pkt = EntityVelocity::new();
            pkt.entity_id = id;
            pkt.velocity = Int16_3::new(
                Self::quantize_velocity(motion.x),
                Self::quantize_velocity(motion.y),
                Self::quantize_velocity(motion.z),
            );
            Self::send_packet_to_players_in_tracked_entry(server, &pkt, tracked_entry);
        }

        tracked_entry.ticks_since_teleport += 1;
        tracked_entry.update_counter += 1;

        let needs_movement_update = tracked_entry.update_counter
            >= tracked_entry.profile.update_frequency
            || i64::from(tracked_entry.ticks_since_teleport) >= force_teleport_ticks;

        if needs_movement_update {
            tracked_entry.update_counter = 0;

            // The threshold-based velocity check
            if tracked_entry.profile.send_velocity {
                let current_motion = {
                    let guard = entity.lock().unwrap();
                    let base = guard.base();
                    Vec3::new(base.motion_x, base.motion_y, base.motion_z)
                };
                let last_motion = tracked_entry.last_broadcast_motion;
                let dmx = current_motion.x - last_motion.x;
                let dmy = current_motion.y - last_motion.y;
                let dmz = current_motion.z - last_motion.z;
                let delta_sq = dmx * dmx + dmy * dmy + dmz * dmz;
                let motion_threshold = 0.02;

                let needs_velocity_update = delta_sq > motion_threshold * motion_threshold
                    || (delta_sq > 0.0
                        && current_motion.x == 0.0
                        && current_motion.y == 0.0
                        && current_motion.z == 0.0);

                if needs_velocity_update {
                    tracked_entry.last_broadcast_motion = current_motion;
                    let id = entity.lock().unwrap().base().id;
                    let mut pkt = EntityVelocity::new();
                    pkt.entity_id = id;
                    pkt.velocity = Int16_3::new(
                        Self::quantize_velocity(current_motion.x),
                        Self::quantize_velocity(current_motion.y),
                        Self::quantize_velocity(current_motion.z),
                    );
                    Self::send_packet_to_players_in_tracked_entry(server, &pkt, tracked_entry);
                }
            }

            let (rotation_yaw, rotation_pitch, id) = {
                let guard = entity.lock().unwrap();
                let base = guard.base();
                (base.rotation_yaw, base.rotation_pitch, base.id)
            };

            let qx = Self::quantize_position(current_position.x);
            let qy = Self::quantize_position(current_position.y);
            let qz = Self::quantize_position(current_position.z);
            let q_yaw = Self::quantize_rotation(rotation_yaw);
            let q_pitch = Self::quantize_rotation(rotation_pitch);

            let dx = qx - tracked_entry.last_encoded_pos.x;
            let dy = qy - tracked_entry.last_encoded_pos.y;
            let dz = qz - tracked_entry.last_encoded_pos.z;

            let needs_tp = dx < -128
                || dx >= 128
                || dy < -128
                || dy >= 128
                || dz < -128
                || dz >= 128
                || i64::from(tracked_entry.ticks_since_teleport) >= force_teleport_ticks;

            if needs_tp {
                tracked_entry.ticks_since_teleport = 0;

                // resyncs the entity position
                {
                    let mut guard = entity.lock().unwrap();
                    let base = guard.base_mut();
                    base.pos_x = f64::from(qx) / 32.0;
                    base.pos_y = f64::from(qy) / 32.0;
                    base.pos_z = f64::from(qz) / 32.0;
                    base.rebuild_collider();
                }

                let mut pkt = TeleportEntity::new();
                pkt.entity_id = id;
                pkt.position = Int32_3::new(qx, qy, qz);
                pkt.rotation = Int8_2::new(q_yaw as i8, q_pitch as i8);
                Self::send_packet_to_players_in_tracked_entry(server, &pkt, tracked_entry);
                tracked_entry.last_encoded_pos = Int32_3::new(qx, qy, qz);
                tracked_entry.last_encoded_yaw = q_yaw;
                tracked_entry.last_encoded_pitch = q_pitch;
            } else {
                let needs_rel_move = dx != 0 || dy != 0 || dz != 0;
                let needs_rot = q_yaw != tracked_entry.last_encoded_yaw
                    || q_pitch != tracked_entry.last_encoded_pitch;

                if needs_rel_move && needs_rot {
                    let mut pkt = EntityPositionAndRotation::new();
                    pkt.qr_position = Int8_3::new(dx as i8, dy as i8, dz as i8);
                    pkt.q_rotation = Int8_2::new(q_yaw as i8, q_pitch as i8);
                    pkt.entity_id = id;
                    Self::send_packet_to_players_in_tracked_entry(server, &pkt, tracked_entry);
                    tracked_entry.last_encoded_pos = Int32_3::new(qx, qy, qz);
                    tracked_entry.last_encoded_yaw = q_yaw;
                    tracked_entry.last_encoded_pitch = q_pitch;
                    return;
                }
                if needs_rel_move {
                    let mut pkt = EntityPosition::new();
                    pkt.qr_position = Int8_3::new(dx as i8, dy as i8, dz as i8);
                    pkt.entity_id = id;
                    Self::send_packet_to_players_in_tracked_entry(server, &pkt, tracked_entry);
                    tracked_entry.last_encoded_pos = Int32_3::new(qx, qy, qz);
                    return;
                }
                if needs_rot {
                    let mut pkt = EntityRotation::new();
                    pkt.q_rotation = Int8_2::new(q_yaw as i8, q_pitch as i8);
                    pkt.entity_id = id;
                    Self::send_packet_to_players_in_tracked_entry(server, &pkt, tracked_entry);
                    tracked_entry.last_encoded_yaw = q_yaw;
                    tracked_entry.last_encoded_pitch = q_pitch;
                }
            }
        }
    }

    // With my strict goal of keeping strict separation we cannot put this as a virtual in the actual entity class itself
    pub fn get_tracking_profile(&self, entity: &Entity) -> TrackingProfile {
        let r#type = entity.r#type;
        match r#type {
            EntityType::None => TrackingProfile {
                range: 0,
                update_frequency: 0,
                send_velocity: false,
            },
            EntityType::Player => TrackingProfile {
                range: 512,
                update_frequency: 2,
                send_velocity: false,
            },
            EntityType::Fish => TrackingProfile {
                range: 64,
                update_frequency: 5,
                send_velocity: true,
            },
            EntityType::Arrow => TrackingProfile {
                range: 64,
                update_frequency: 20,
                send_velocity: false,
            },
            EntityType::Fireball => TrackingProfile {
                range: 64,
                update_frequency: 10,
                send_velocity: false,
            },
            EntityType::ThrownSnowball | EntityType::ThrownEgg => TrackingProfile {
                range: 64,
                update_frequency: 10,
                send_velocity: true,
            },
            EntityType::Item => TrackingProfile {
                range: 64,
                update_frequency: 20,
                send_velocity: true,
            },
            EntityType::Minecart | EntityType::Boat => TrackingProfile {
                range: 160,
                update_frequency: 5,
                send_velocity: true,
            },
            EntityType::Squid => TrackingProfile {
                range: 160,
                update_frequency: 3,
                send_velocity: true,
            },
            EntityType::Chicken
            | EntityType::Cow
            | EntityType::Pig
            | EntityType::Sheep
            | EntityType::Wolf
            | EntityType::Zombie
            | EntityType::ZombiePigman
            | EntityType::Skeleton
            | EntityType::Creeper
            | EntityType::Spider
            | EntityType::Ghast
            | EntityType::Slime
            | EntityType::GiantZombie => TrackingProfile {
                range: 160,
                update_frequency: 3,
                send_velocity: false,
            },
            EntityType::LitTnt => TrackingProfile {
                range: 160,
                update_frequency: 10,
                send_velocity: true,
            },
            EntityType::FallingSand | EntityType::FallingGravel => TrackingProfile {
                range: 160,
                update_frequency: 20,
                send_velocity: true,
            },
            EntityType::Painting => {
                // Paintings never move so there's nothing to resync
                TrackingProfile {
                    range: 160,
                    update_frequency: i32::MAX,
                    send_velocity: false,
                }
            }
            _ => {
                let profile = TrackingProfile {
                    range: 0,
                    update_frequency: 0,
                    send_velocity: false,
                };
                return profile;

                global_logger().warn(format!(
                    "EntityTracker: no tracking profile for entity type '{}'\n",
                    r#type as i32
                ));
            }
        }
    }
}

impl Default for EntityTracker {
    fn default() -> Self {
        Self::new()
    }
}
