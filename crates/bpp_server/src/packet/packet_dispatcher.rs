/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
use bpp_shared::enums::network::packet_ids::{self, PacketId};
use bpp_shared::logger::logger::global_logger;
use bpp_shared::networking::packets::{self, PacketBehavior};
use bpp_shared::world::world::WorldManager;

use crate::packet::handle_packet;
use crate::player_conn::player_session::PlayerSession;
use crate::server::Server;

// Dispatches a single already-identified incoming packet to its handler.
pub fn dispatch(packet_id: PacketId, session: &mut PlayerSession, session_world: &mut WorldManager, server: &mut Server) -> bool {
    match packet_id {
        packet_ids::KEEP_ALIVE => {
            let mut pkt = packets::KeepAlive::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::keep_alive(&mut pkt, session);
        }
        packet_ids::CHAT_MESSAGE => {
            let mut pkt = packets::ChatMessage::new();
            pkt.deserialize(&mut session.stream);
            let mut transfer_dimension = |_s: &mut PlayerSession| {};
            handle_packet::chat_message(&mut pkt, session, session_world, &mut transfer_dimension, server);
        }
        packet_ids::SET_TIME => {
            let mut pkt = packets::SetTime::new();
            pkt.deserialize(&mut session.stream);
        }
        packet_ids::INTERACT_WITH_ENTITY => {
            let mut pkt = packets::InteractWithEntity::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::interact_with_entity(&mut pkt, session, session_world);
        }
        packet_ids::RESPAWN => {
            let mut pkt = packets::Respawn::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::respawn(&mut pkt, session);
        }
        packet_ids::PLAYER_MOVEMENT => {
            let mut pkt = packets::PlayerMovement::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::player_movement(&mut pkt, session);
        }
        packet_ids::PLAYER_POSITION => {
            let mut pkt = packets::PlayerPosition::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::player_position(&mut pkt, session);
        }
        packet_ids::PLAYER_ROTATION => {
            let mut pkt = packets::PlayerRotation::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::player_rotation(&mut pkt, session);
        }
        packet_ids::PLAYER_POSITION_AND_ROTATION => {
            let mut pkt = packets::PlayerPositionAndRotation::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::player_position_and_rotation(&mut pkt, session);
        }
        packet_ids::MINE_BLOCK => {
            let mut pkt = packets::MineBlock::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::mine_block(&mut pkt, session, session_world, &mut server.players);
        }
        packet_ids::PLACE_BLOCK => {
            let mut pkt = packets::PlaceBlock::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::place_block(&mut pkt, session, session_world, &mut server.game_runtime);
        }
        packet_ids::SET_HOTBAR_SLOT => {
            let mut pkt = packets::SetHotbarSlot::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::set_hotbar_slot(&mut pkt, session);
        }
        packet_ids::INTERACT_WITH_BLOCK => {
            let mut pkt = packets::InteractWithBlock::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::interact_with_block(&mut pkt, session, session_world);
        }
        packet_ids::ANIMATION => {
            let mut pkt = packets::Animation::new();
            pkt.deserialize(&mut session.stream);
            let entity_tracker =
                if session.dimension == 0 { &server.overworld_entity_tracker } else { &server.hell_entity_tracker };
            let mut entity_tracker_guard = entity_tracker.lock().unwrap();
            handle_packet::animation(&mut pkt, session, &mut entity_tracker_guard);
        }
        packet_ids::PLAYER_ACTION => {
            let mut pkt = packets::PlayerAction::new();
            pkt.deserialize(&mut session.stream);
            let entity_tracker =
                if session.dimension == 0 { &server.overworld_entity_tracker } else { &server.hell_entity_tracker };
            let mut entity_tracker_guard = entity_tracker.lock().unwrap();
            handle_packet::player_action(&mut pkt, session, &mut entity_tracker_guard);
        }
        packet_ids::PLAYER_INPUT => {
            let mut pkt = packets::PlayerInput::new();
            pkt.deserialize(&mut session.stream);
        }
        packet_ids::CLOSE_CONTAINER => {
            let mut pkt = packets::CloseContainer::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::close_container(&mut pkt, session);
        }
        packet_ids::CLICK_SLOT => {
            let mut pkt = packets::ClickSlot::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::click_slot(&mut pkt, session, session_world, &server.game_runtime);
        }
        packet_ids::CONTAINER_TRANSACTION => {
            let mut pkt = packets::ContainerTransaction::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::container_transaction(&mut pkt, session);
        }
        packet_ids::UPDATE_SIGN => {
            let mut pkt = packets::UpdateSign::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::update_sign(&mut pkt, session, session_world);
        }
        packet_ids::DISCONNECT => {
            let mut pkt = packets::Disconnect::new();
            pkt.deserialize(&mut session.stream);
            handle_packet::disconnect(&mut pkt, session);
            return false; // session is dead; stop processing
        }
        _ => {
            global_logger().warn(format!("UNHANDLED packet 0x{:x}\n", packet_id.0));
            let conn_state_manager = std::mem::take(&mut server.conn_state_manager);
            conn_state_manager.disconnect_player(session, "Unknown packet", server);
            server.conn_state_manager = conn_state_manager;
            return false;
        }
    }
    true
}
