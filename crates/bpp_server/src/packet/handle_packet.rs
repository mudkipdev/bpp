/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 * Copyright (c) 2026, jwaxy <jwaxy.is-a.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
use std::sync::{Arc, Mutex};

use bpp_shared::base_types::{EntityId, NetworkSlotId, WindowId};
use bpp_shared::blocks::{block_properties, block_registration};
use bpp_shared::entities::entity::EntityBehavior;
use bpp_shared::entities::entity_item::ItemEntity;
use bpp_shared::entities::entity_player::PlayerEntityBehavior;
use bpp_shared::enums::blocks::{BLOCK_AIR, BlockType};
use bpp_shared::enums::network::packet_data;
use bpp_shared::inventory::inventory::InventoryBehavior;
use bpp_shared::inventory::inventory_interaction::InventoryInteractionBehavior;
use bpp_shared::inventory::item_stack::ItemStack;
use bpp_shared::items::item_properties;
use bpp_shared::logger::logger::global_logger;
use bpp_shared::networking::packets::{self, PacketBehavior};
use bpp_shared::numeric_structs::{Int3, SlimInt3, Vec3};
use bpp_shared::runtime::Runtime;
use bpp_shared::world::world::WorldManager;

use crate::blocks::server_block_behaviors;
use crate::commands::command_manager::CommandManager;
use crate::entities::entity_tracker::EntityTracker;
use crate::packet::packet_utils;
use crate::player_conn::player_session::{ActiveContainer, PlayerSession};
use crate::server::Server;

pub fn keep_alive(_pkt: &mut packets::KeepAlive, session: &mut PlayerSession) {
    let ka = packets::KeepAlive::new();
    ka.serialize(&mut session.stream);
}

pub fn chat_message(
    pkt: &mut packets::ChatMessage,
    session: &mut PlayerSession,
    world: &mut WorldManager,
    transfer_dimension: &mut dyn FnMut(&mut PlayerSession),
    server: &mut Server,
) {
    if !pkt.message.is_empty() && pkt.message.starts_with('/') {
        CommandManager::parse(&mut pkt.message, session, world, transfer_dimension, server);
        return;
    }
    let broadcast = format!("<{}> {}", session.username, pkt.message);
    server.send_global_chat_message_from(&broadcast, session);
}

pub fn player_movement(_pkt: &mut packets::PlayerMovement, _session: &mut PlayerSession) {
    // onGround flag only, so no position update needed.
}

pub fn player_position(pkt: &mut packets::PlayerPosition, session: &mut PlayerSession) {
    session.position.pos = pkt.position;
}

pub fn player_rotation(pkt: &mut packets::PlayerRotation, session: &mut PlayerSession) {
    session.rotation.x = pkt.rotation.x;
    session.rotation.y = pkt.rotation.y;
}

pub fn player_position_and_rotation(pkt: &mut packets::PlayerPositionAndRotation, session: &mut PlayerSession) {
    session.position.pos = pkt.position;
    session.rotation.x = pkt.rotation.x;
    session.rotation.y = pkt.rotation.y;
}

// TODO: Move this elsewhere!!!!!
pub fn break_and_drop_block(world: &mut WorldManager, pos: &SlimInt3<i8>) {
    let block_id = world.get_block_id(Int3::new(pos.x, i32::from(pos.y), pos.z));
    let meta = world.get_metadata(Int3::new(pos.x, i32::from(pos.y), pos.z));
    world.set_block(Int3::new(pos.x, i32::from(pos.y), pos.z), BLOCK_AIR, 0);

    let drops = block_registration::get_block_drops(block_id, meta, &mut world.rand);

    for drop in drops {
        let mut drop_pos = Vec3::new(f64::from(pos.x), f64::from(pos.y), f64::from(pos.z));
        let offset: f32 = 0.7;
        drop_pos.x += f64::from((world.rand.next_float() * offset) + (1.0 - offset) * 0.5);
        drop_pos.y += f64::from((world.rand.next_float() * offset) + (1.0 - offset) * 0.5);
        drop_pos.z += f64::from((world.rand.next_float() * offset) + (1.0 - offset) * 0.5);
        let mut item = ItemEntity::new(drop_pos);
        item.item_stack = drop;
        world.entity_manager.add_entity(Arc::new(Mutex::new(item)), EntityId(-1));
    }
}

pub fn mine_block(
    pkt: &mut packets::MineBlock,
    session: &mut PlayerSession,
    world: &mut WorldManager,
    _players: &mut Vec<Arc<Mutex<PlayerSession>>>,
) {
    let packet_pos = Int3::new(pkt.position.x, i32::from(pkt.position.y), pkt.position.z);
    match pkt.status {
        packet_data::DIGGING_STARTED => {
            session.started_mining_at_tick = world.elapsed_ticks;
            let block_id = world.get_block_id(Int3::new(pkt.position.x, i32::from(pkt.position.y), pkt.position.z));
            session.last_targeted_block = block_id;

            if block_properties::block_properties()[session.last_targeted_block.0 as u8 as usize].hardness == 0.0 {
                break_and_drop_block(world, &pkt.position);
                return;
            }

            if let Some(function) =
                block_properties::block_behaviors()[session.last_targeted_block.0 as u8 as usize].on_block_clicked
            {
                function(world, packet_pos);
            }
        }
        packet_data::DIGGING_FINISHED => {
            if session.last_targeted_block != world.get_block_id(Int3::new(pkt.position.x, i32::from(pkt.position.y), pkt.position.z)) {
                return; // block changed while mining so we don't drop it
            }
            break_and_drop_block(world, &pkt.position);
        }
        packet_data::DROPPED_ITEM => {
            let held_stack = match session.inventory.get_held_item() {
                Some(stack) => stack,
                None => return,
            };

            let mut dropped_stack = *held_stack;
            dropped_stack.count = 1;

            if session.entity.as_ref().unwrap().lock().unwrap().drop_item(dropped_stack) {
                held_stack.decrement_count(1);
            }
        }
        _ => {}
    }
}

pub fn place_block(pkt: &mut packets::PlaceBlock, session: &mut PlayerSession, world: &mut WorldManager, game_runtime: &mut Runtime) {
    let position = Int3::new(pkt.position.x, i32::from(pkt.position.y), pkt.position.z);
    // Block interactions
    let block = world.get_block_id(position);

    // Function returns true if we can place a block after running the function
    if let Some(on_block_activated) = server_block_behaviors::block_behaviors()[block.0 as u8 as usize].on_block_activated {
        if !on_block_activated(world, position, session, game_runtime) {
            return;
        }
    }
    // The server didn't override our block's behavior so check the base behavior
    else if let Some(on_block_activated) = block_properties::block_behaviors()[block.0 as u8 as usize].on_block_activated {
        if !on_block_activated(world, position) {
            return;
        }
    }

    let held_item = match session.inventory.get_held_item() {
        Some(item) => item,
        None => return,
    };

    if pkt.face == packet_data::INVALID_USE {
        // Custom behaviour can be here if needed.
        return;
    }

    if !item_properties::is_valid(held_item.id) {
        // It's a block
        let mut place_position = position;
        if pkt.face == packet_data::Y_MINUS {
            place_position.y -= 1;
        }
        if pkt.face == packet_data::Y_PLUS {
            place_position.y += 1;
        }
        if pkt.face == packet_data::Z_MINUS {
            place_position.z -= 1;
        }
        if pkt.face == packet_data::Z_PLUS {
            place_position.z += 1;
        }
        if pkt.face == packet_data::X_MINUS {
            place_position.x -= 1;
        }
        if pkt.face == packet_data::X_PLUS {
            place_position.x += 1;
        }

        let block_id = BlockType(held_item.id.0 as i8);
        world.set_block(place_position, block_id, held_item.data as u8);
        let function = block_properties::block_behaviors()[block_id.0 as u8 as usize].on_block_placed;
        if let Some(function) = function {
            function(world, place_position, session.entity.as_ref().unwrap().lock().unwrap().base_mut(), pkt.face);
        }
        held_item.decrement_count(1);
    } else {
        // It's an item
        global_logger().info("Tried to use item\n");
        global_logger().info(format!("{position}\n"));
        let behavior = item_properties::item_behavior().lock().unwrap().get(&held_item.id).copied();
        if let Some(behavior) = behavior {
            if let Some(on_block_use) = behavior.on_block_use {
                global_logger().info(format!("Used on {position}\n"));
                on_block_use(world, position);
            }
        }
    }
}

pub fn set_hotbar_slot(pkt: &mut packets::SetHotbarSlot, session: &mut PlayerSession) {
    if pkt.slot.0 < 0 || pkt.slot.0 >= 9 {
        return;
    }
    session.inventory.active_hotbar_slot = i32::from(pkt.slot.0);
}

// Click handler
pub fn click_slot(pkt: &mut packets::ClickSlot, session: &mut PlayerSession, world: &mut WorldManager, game_runtime: &Runtime) {
    if session.inventory_locked {
        return;
    }
    session.pending_window_id = pkt.window_id;
    session.pending_transaction_id = pkt.transaction_id;

    // The player's inventory is handled seperate
    if pkt.window_id.0 == 0 {
        // Make sure what the client thinks and what we have line up
        let empty = ItemStack::default();
        let slot_item = match session.inventory.get_stack_in_slot(i32::from(pkt.slot_id.0)) {
            Some(item) => *item,
            None => empty,
        };
        if slot_item.id != pkt.item.id || slot_item.data != pkt.item.data || slot_item.count != pkt.item.count {
            let mut ct = packets::ContainerTransaction::new();
            ct.accepted = false;
            ct.transaction_id = session.pending_transaction_id;
            ct.window_id = pkt.window_id;
            ct.serialize(&mut session.stream);
            session.inventory_locked = true;

            // Reset the held cursor
            packet_utils::send_slot(session, WindowId(-1), NetworkSlotId(-1), Some(&empty));

            let inventory = session.inventory.base.clone();
            packet_utils::send_inventory(session, pkt.window_id, inventory);
            return;
        }

        // Everything lined up so go as normal
        if pkt.right_click {
            session.with_own_interaction(game_runtime, |i| i.on_right_click(i32::from(pkt.slot_id.0)));
            return;
        }
        if pkt.shift {
            session.with_own_interaction(game_runtime, |i| i.on_shift_click(i32::from(pkt.slot_id.0)));
            return;
        }
        session.with_own_interaction(game_runtime, |i| i.on_left_click(i32::from(pkt.slot_id.0)));
        return;
    }
    let empty = ItemStack::default();
    let slot_item = session
        .with_active_interaction(game_runtime, world, |i| i.inventory().get_stack_in_slot(i32::from(pkt.slot_id.0)).copied())
        .flatten()
        .unwrap_or(empty);
    if slot_item.id != pkt.item.id || slot_item.data != pkt.item.data || slot_item.count != pkt.item.count {
        let mut ct = packets::ContainerTransaction::new();
        ct.accepted = false;
        ct.transaction_id = session.pending_transaction_id;
        ct.window_id = pkt.window_id;
        ct.serialize(&mut session.stream);
        session.inventory_locked = true;

        // Reset the held cursor
        packet_utils::send_slot(session, WindowId(-1), NetworkSlotId(-1), Some(&empty));
        if let Some(inventory) = session.with_active_interaction(game_runtime, world, |i| i.inventory().base().clone()) {
            packet_utils::send_inventory(session, pkt.window_id, inventory);
        }
        return;
    }

    // Everything lined up so go as normal
    if pkt.right_click {
        session.with_active_interaction(game_runtime, world, |i| i.on_right_click(i32::from(pkt.slot_id.0)));
        return;
    }
    if pkt.shift {
        session.with_active_interaction(game_runtime, world, |i| i.on_shift_click(i32::from(pkt.slot_id.0)));
        return;
    }
    session.with_active_interaction(game_runtime, world, |i| i.on_left_click(i32::from(pkt.slot_id.0)));
}

pub fn close_container(pkt: &mut packets::CloseContainer, session: &mut PlayerSession) {
    if pkt.window_id.0 == 0 && session.entity.is_some() {
        // Drop the crafting grid items on inventory close
        for i in 1..=4usize {
            let stack = session.inventory.base.slots[i];
            session.entity.as_ref().unwrap().lock().unwrap().drop_item(stack);
            session.inventory.base.slots[i] = ItemStack::default();
        }
    }

    // Get rid of our active interaction and reset the window id
    session.active_container = ActiveContainer::None;
    session.open_window_id = WindowId(0);
}

// Client acknowledges a rejected transaction
pub fn container_transaction(pkt: &mut packets::ContainerTransaction, session: &mut PlayerSession) {
    if session.inventory_locked && pkt.window_id == session.pending_window_id && pkt.transaction_id == session.pending_transaction_id {
        session.inventory_locked = false;
    }
}

// Other handlers
pub fn interact_with_entity(pkt: &mut packets::InteractWithEntity, session: &mut PlayerSession, world: &mut WorldManager) {
    // Check if session entity and source entity match
    if pkt.source_entity_id != session.entity.as_ref().unwrap().lock().unwrap().base().id {
        return;
    }

    // Check if target entity exists
    let entity = match world.entity_manager.entities.get(pkt.target_entity_id.0 as usize) {
        Some(entity) => Arc::clone(entity),
        None => return,
    };

    let held_item_id = match session.inventory.get_held_item() {
        Some(item) => item.id,
        None => return,
    };

    // Get item behavior
    let behavior = match item_properties::item_behavior().lock().unwrap().get(&held_item_id).copied() {
        Some(behavior) => behavior,
        None => return,
    };

    if pkt.attack {
        if let Some(on_entity_attack) = behavior.on_entity_attack {
            on_entity_attack(entity.lock().unwrap().base_mut());
        }
    } else if let Some(on_entity_use) = behavior.on_entity_use {
        on_entity_use(entity.lock().unwrap().base_mut());
    }
}

pub fn interact_with_block(_pkt: &mut packets::InteractWithBlock, _session: &mut PlayerSession, _world: &mut WorldManager) {}

pub fn animation(pkt: &mut packets::Animation, session: &mut PlayerSession, entity_tracker: &mut EntityTracker) {
    // Broadcast what we were sent to players who can see this player
    let mut anim = packets::Animation::new();
    anim.entity_id = session.entity.as_ref().unwrap().lock().unwrap().base().id;
    anim.animation = pkt.animation;
    entity_tracker.send_packet_to_viewers(&anim, anim.entity_id);
}

pub fn player_action(_pkt: &mut packets::PlayerAction, _session: &mut PlayerSession, _entity_tracker: &mut EntityTracker) {
    // Broadcast what we were sent to players who can see this player
}

pub fn respawn(_pkt: &mut packets::Respawn, _session: &mut PlayerSession) {
    // TODO: reset position, health, send spawn chunks
}

pub fn update_sign(_pkt: &mut packets::UpdateSign, _session: &mut PlayerSession, _world: &mut WorldManager) {
    // TODO: write sign text to world, broadcast to nearby clients
}

pub fn disconnect(_pkt: &mut packets::Disconnect, session: &mut PlayerSession) {
    session.stream.set_connected(false);
}
