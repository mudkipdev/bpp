/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

// The IDs for all the packets. These names were adopted from and
// decided by the people running the Technical Beta-Wiki.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PacketId(pub u8);

pub const KEEP_ALIVE: PacketId = PacketId(0x00);
pub const LOGIN: PacketId = PacketId(0x01);
pub const PRE_LOGIN: PacketId = PacketId(0x02);
pub const CHAT_MESSAGE: PacketId = PacketId(0x03);
pub const SET_TIME: PacketId = PacketId(0x04);
pub const SET_EQUIPMENT: PacketId = PacketId(0x05);
pub const SET_SPAWN_POSITION: PacketId = PacketId(0x06);
pub const INTERACT_WITH_ENTITY: PacketId = PacketId(0x07);
pub const SET_HEALTH: PacketId = PacketId(0x08);
pub const RESPAWN: PacketId = PacketId(0x09);
pub const PLAYER_MOVEMENT: PacketId = PacketId(0x0A);
pub const PLAYER_POSITION: PacketId = PacketId(0x0B);
pub const PLAYER_ROTATION: PacketId = PacketId(0x0C);
pub const PLAYER_POSITION_AND_ROTATION: PacketId = PacketId(0x0D);
pub const MINE_BLOCK: PacketId = PacketId(0x0E);
pub const PLACE_BLOCK: PacketId = PacketId(0x0F);
pub const SET_HOTBAR_SLOT: PacketId = PacketId(0x10);
pub const INTERACT_WITH_BLOCK: PacketId = PacketId(0x11);
pub const ANIMATION: PacketId = PacketId(0x12);
pub const PLAYER_ACTION: PacketId = PacketId(0x13);
pub const SPAWN_PLAYER: PacketId = PacketId(0x14);
pub const SPAWN_ITEM: PacketId = PacketId(0x15);
pub const COLLECT_ITEM: PacketId = PacketId(0x16);
pub const SPAWN_OBJECT: PacketId = PacketId(0x17);
pub const SPAWN_MOB: PacketId = PacketId(0x18);
pub const SPAWN_PAINTING: PacketId = PacketId(0x19);
pub const PLAYER_INPUT: PacketId = PacketId(0x1B); // Unused, only implemented on the Client
pub const ENTITY_VELOCITY: PacketId = PacketId(0x1C);
pub const DESPAWN_ENTITY: PacketId = PacketId(0x1D);
pub const ENTITY_MOVEMENT: PacketId = PacketId(0x1E); // Unused, in practice
pub const ENTITY_POSITION: PacketId = PacketId(0x1F);
pub const ENTITY_ROTATION: PacketId = PacketId(0x20);
pub const ENTITY_POSITION_AND_ROTATION: PacketId = PacketId(0x21);
pub const TELEPORT_ENTITY: PacketId = PacketId(0x22);
pub const ENTITY_EVENT: PacketId = PacketId(0x26);
pub const ADD_PASSENGER: PacketId = PacketId(0x27);
pub const ENTITY_METADATA: PacketId = PacketId(0x28);
pub const SET_CHUNK_VISIBILITY: PacketId = PacketId(0x32);
pub const CHUNK: PacketId = PacketId(0x33);
pub const SET_MULTIPLE_BLOCKS: PacketId = PacketId(0x34);
pub const SET_BLOCK: PacketId = PacketId(0x35);
pub const BLOCK_EVENT: PacketId = PacketId(0x36);
pub const EXPLOSION: PacketId = PacketId(0x3C);
pub const WORLD_EVENT: PacketId = PacketId(0x3D);
pub const GAME_EVENT: PacketId = PacketId(0x46);
pub const LIGHTNING_BOLT: PacketId = PacketId(0x47);
pub const OPEN_CONTAINER: PacketId = PacketId(0x64);
pub const CLOSE_CONTAINER: PacketId = PacketId(0x65);
pub const CLICK_SLOT: PacketId = PacketId(0x66);
pub const SET_SLOT: PacketId = PacketId(0x67);
pub const FILL_CONTAINER: PacketId = PacketId(0x68);
pub const CONTAINER_DATA: PacketId = PacketId(0x69);
pub const CONTAINER_TRANSACTION: PacketId = PacketId(0x6A);
pub const UPDATE_SIGN: PacketId = PacketId(0x82);
pub const ITEM_DATA: PacketId = PacketId(0x83);
pub const INCREMENT_STATISTIC: PacketId = PacketId(0xC8);
pub const DISCONNECT: PacketId = PacketId(0xFF);
