/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use crate::base_structs::Block;
use crate::base_types::{
    EntityId, ItemDamage, ItemId, MapId, NetworkSlotId, TickTime, TransactionId, WindowId,
};
use crate::constants::{CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::enums::blocks::BlockType;
use crate::enums::dimensions::Dimension;
use crate::enums::items;
use crate::enums::network::packet_data;
use crate::enums::network::packet_data::entity_metadata::DataEntry;
use crate::enums::network::packet_ids;
use crate::enums::network::packet_ids::PacketId;
use crate::inventory::item_stack::ItemStack;
use crate::networking::network_stream::NetworkStream;
use crate::numeric_structs::{Float2, Int8_2, Int8_3, Int16_3, Int32_2, Int32_3, SlimInt3, TriNumber, Vec3};

// This class serves as a nice, convenient wrapper
// around the networking packets

// NOTE: The base packet should never be used directly!!
// Only public so that packets can be passed through functions
pub struct BasePacket {
    pub id: PacketId,
}

impl BasePacket {
    pub fn new(id: PacketId) -> Self {
        Self { id }
    }
}

pub trait PacketBehavior {
    fn base(&self) -> &BasePacket;
    fn base_mut(&mut self) -> &mut BasePacket;

    fn serialize(&self, stream: &mut NetworkStream);
    fn deserialize(&mut self, stream: &mut NetworkStream);
}

fn dimension_from_i8(value: i8) -> Dimension {
    match value {
        -1 => Dimension::Nether,
        _ => Dimension::Overworld,
    }
}

// Used to keep the connection alive
pub struct KeepAlive {
    pub base: BasePacket,
}

impl KeepAlive {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::KEEP_ALIVE),
        }
    }
}

impl PacketBehavior for KeepAlive {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
    }

    // NOTE: Reading the packet id is enough to deserialize it
    fn deserialize(&mut self, _stream: &mut NetworkStream) {}
}

// Used to finalize the connection
pub struct Login {
    pub base: BasePacket,
    // NOTE: This assumes that EntityId is always an int32_t
    pub entity_id: EntityId,
    pub username: String,
    pub world_seed: i64,
    pub dimension: Dimension,
}

impl Login {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::LOGIN),
            entity_id: EntityId(0),
            username: String::new(),
            world_seed: 0,
            dimension: Dimension::Overworld,
        }
    }
}

impl PacketBehavior for Login {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_string16(&self.username);
        stream.write_i64(self.world_seed);
        stream.write_i8(self.dimension as i8);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.username = stream.read_string16();
        self.world_seed = stream.read_i64();
        self.dimension = dimension_from_i8(stream.read_i8());
    }
}

// Used to initialize to connection
pub struct PreLogin {
    pub base: BasePacket,
    pub username: String,
}

impl PreLogin {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::PRE_LOGIN),
            username: String::new(),
        }
    }
}

impl PacketBehavior for PreLogin {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_string16(&self.username);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.username = stream.read_string16();
    }
}

// Holds a chat message
pub struct ChatMessage {
    pub base: BasePacket,
    pub message: String,
}

impl ChatMessage {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::CHAT_MESSAGE),
            message: String::new(),
        }
    }
}

impl PacketBehavior for ChatMessage {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_string16(&self.message);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.message = stream.read_string16();
    }
}

// Holds the current time
pub struct SetTime {
    pub base: BasePacket,
    pub time: TickTime,
}

impl SetTime {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::SET_TIME),
            time: 0,
        }
    }
}

impl PacketBehavior for SetTime {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i64(self.time);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.time = stream.read_i64();
    }
}

// Defines a players equipment
pub struct SetEquipment {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub inventory_slot: NetworkSlotId,
    pub item_id: ItemId,
    pub item_metadata: ItemDamage,
}

impl SetEquipment {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::SET_EQUIPMENT),
            entity_id: EntityId(0),
            inventory_slot: NetworkSlotId(0),
            item_id: ItemId(0),
            item_metadata: 0,
        }
    }
}

impl PacketBehavior for SetEquipment {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_i16(self.inventory_slot.0);
        stream.write_i16(self.item_id.0);
        stream.write_i16(self.item_metadata);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.inventory_slot = NetworkSlotId(stream.read_i16());
        self.item_id = ItemId(stream.read_i16());
        self.item_metadata = stream.read_i16();
    }
}

// Defines where the compass points
pub struct SetSpawnPosition {
    pub base: BasePacket,
    pub position: Int32_3,
}

impl SetSpawnPosition {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::SET_SPAWN_POSITION),
            position: Int32_3::default(),
        }
    }
}

impl PacketBehavior for SetSpawnPosition {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.position.x);
        stream.write_i32(self.position.y);
        stream.write_i32(self.position.z);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.position.x = stream.read_i32();
        self.position.y = stream.read_i32();
        self.position.z = stream.read_i32();
    }
}

// Used to convey who interacted with who
pub struct InteractWithEntity {
    pub base: BasePacket,
    pub source_entity_id: EntityId,
    pub target_entity_id: EntityId,
    pub attack: bool, // Usually sent when left-clicking
}

impl InteractWithEntity {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::INTERACT_WITH_ENTITY),
            source_entity_id: EntityId(0),
            target_entity_id: EntityId(0),
            attack: false,
        }
    }
}

impl PacketBehavior for InteractWithEntity {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.source_entity_id.0);
        stream.write_i32(self.target_entity_id.0);
        stream.write_bool(self.attack);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.source_entity_id = EntityId(stream.read_i32());
        self.target_entity_id = EntityId(stream.read_i32());
        self.attack = stream.read_bool();
    }
}

// Defines a players health
pub struct SetHealth {
    pub base: BasePacket,
    pub health: i16,
}

impl SetHealth {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::SET_HEALTH),
            health: 0,
        }
    }
}

impl PacketBehavior for SetHealth {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i16(self.health);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.health = stream.read_i16();
    }
}

// Defines a players health
pub struct Respawn {
    pub base: BasePacket,
    pub dimension: Dimension,
}

impl Respawn {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::RESPAWN),
            dimension: Dimension::Overworld,
        }
    }
}

impl PacketBehavior for Respawn {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i8(self.dimension as i8);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.dimension = dimension_from_i8(stream.read_i8());
    }
}

// Base Packet for player movement packets
pub struct PlayerMovement {
    pub base: BasePacket,
    pub on_ground: bool,
}

impl PlayerMovement {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::PLAYER_MOVEMENT),
            on_ground: false,
        }
    }
}

impl PacketBehavior for PlayerMovement {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_bool(self.on_ground);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.on_ground = stream.read_bool();
    }
}

// Defines the players position
pub struct PlayerPosition {
    pub base: BasePacket,
    pub position: Vec3,
    pub camera_y: f64,
    pub on_ground: bool,
}

impl PlayerPosition {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::PLAYER_POSITION),
            position: Vec3::default(),
            camera_y: 0.0,
            on_ground: false,
        }
    }
}

impl PacketBehavior for PlayerPosition {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_f64(self.position.x);
        stream.write_f64(self.position.y);
        stream.write_f64(self.camera_y);
        stream.write_f64(self.position.z);
        stream.write_bool(self.on_ground);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.position.x = stream.read_f64();
        self.position.y = stream.read_f64();
        self.camera_y = stream.read_f64();
        self.position.z = stream.read_f64();
        self.on_ground = stream.read_bool();
    }
}

// Defines the players rotation
pub struct PlayerRotation {
    pub base: BasePacket,
    pub rotation: Float2, // wire order: yaw first
    pub on_ground: bool,
}

impl PlayerRotation {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::PLAYER_ROTATION),
            rotation: Float2::default(),
            on_ground: false,
        }
    }
}

impl PacketBehavior for PlayerRotation {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_f32(self.rotation.x);
        stream.write_f32(self.rotation.y);
        stream.write_bool(self.on_ground);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.rotation.x = stream.read_f32();
        self.rotation.y = stream.read_f32();
        self.on_ground = stream.read_bool();
    }
}

// Defines the players position and rotation
pub struct PlayerPositionAndRotation {
    pub base: BasePacket,
    pub position: Vec3,
    pub camera_y: f64,
    pub rotation: Float2, // wire order: yaw first
    pub on_ground: bool,
}

impl PlayerPositionAndRotation {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::PLAYER_POSITION_AND_ROTATION),
            position: Vec3::default(),
            camera_y: 0.0,
            rotation: Float2::default(),
            on_ground: false,
        }
    }
}

impl PacketBehavior for PlayerPositionAndRotation {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_f64(self.position.x);
        stream.write_f64(self.position.y);
        stream.write_f64(self.camera_y);
        stream.write_f64(self.position.z);
        stream.write_f32(self.rotation.x);
        stream.write_f32(self.rotation.y);
        stream.write_bool(self.on_ground);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.position.x = stream.read_f64();
        self.position.y = stream.read_f64();
        self.camera_y = stream.read_f64();
        self.position.z = stream.read_f64();
        self.rotation.x = stream.read_f32();
        self.rotation.y = stream.read_f32();
        self.on_ground = stream.read_bool();
    }
}

// Information on how far along the player is with breaking a block
pub struct MineBlock {
    pub base: BasePacket,
    pub status: packet_data::MineStatus,
    pub position: SlimInt3<i8>,
    pub face: packet_data::FaceDirection,
}

impl MineBlock {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::MINE_BLOCK),
            status: packet_data::MineStatus::default(),
            position: SlimInt3::new(0, 0, 0),
            face: packet_data::FaceDirection::default(),
        }
    }
}

impl PacketBehavior for MineBlock {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_u8(self.status.0);
        stream.write_i32(self.position.x);
        stream.write_i8(self.position.y);
        stream.write_i32(self.position.z);
        stream.write_i8(self.face.0);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.status = packet_data::MineStatus(stream.read_u8());
        self.position.x = stream.read_i32();
        self.position.y = stream.read_i8();
        self.position.z = stream.read_i32();
        self.face = packet_data::FaceDirection(stream.read_i8());
    }
}

// Information on where a player is placing a block
pub struct PlaceBlock {
    pub base: BasePacket,
    pub position: SlimInt3<i8>,
    pub face: packet_data::FaceDirection,
    pub item: ItemStack,
}

impl PlaceBlock {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::PLACE_BLOCK),
            position: SlimInt3::new(0, 0, 0),
            face: packet_data::FaceDirection::default(),
            item: ItemStack::default(),
        }
    }
}

impl PacketBehavior for PlaceBlock {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.position.x);
        stream.write_i8(self.position.y);
        stream.write_i32(self.position.z);
        stream.write_i8(self.face.0);
        stream.write_i16(self.item.id.0);
        stream.write_i8(self.item.count);
        stream.write_i16(self.item.data);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.position.x = stream.read_i32();
        self.position.y = stream.read_i8();
        self.position.z = stream.read_i32();
        self.face = packet_data::FaceDirection(stream.read_i8());
        self.item.id = ItemId(stream.read_i16());
        if self.item.id.0 >= 0 {
            self.item.count = stream.read_i8();
            self.item.data = stream.read_i16();
        }
    }
}

// The clients active hotbar slot
pub struct SetHotbarSlot {
    pub base: BasePacket,
    pub slot: NetworkSlotId,
}

impl SetHotbarSlot {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::SET_HOTBAR_SLOT),
            slot: NetworkSlotId(0),
        }
    }
}

impl PacketBehavior for SetHotbarSlot {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i16(self.slot.0);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.slot = NetworkSlotId(stream.read_i16());
    }
}

// Interactions with blocks
pub struct InteractWithBlock {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub interaction_id: packet_data::BlockInteraction,
    pub position: SlimInt3<i8>,
}

impl InteractWithBlock {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::INTERACT_WITH_BLOCK),
            entity_id: EntityId(0),
            interaction_id: packet_data::BlockInteraction::default(),
            position: SlimInt3::new(0, 0, 0),
        }
    }
}

impl PacketBehavior for InteractWithBlock {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_u8(self.interaction_id.0);
        stream.write_i32(self.position.x);
        stream.write_i8(self.position.y);
        stream.write_i32(self.position.z);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.interaction_id = packet_data::BlockInteraction(stream.read_u8());
        self.position.x = stream.read_i32();
        self.position.y = stream.read_i8();
        self.position.z = stream.read_i32();
    }
}

// Informs of the desired animation
pub struct Animation {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub animation: packet_data::Animation,
}

impl Animation {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::ANIMATION),
            entity_id: EntityId(0),
            animation: packet_data::Animation::default(),
        }
    }
}

impl PacketBehavior for Animation {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_i8(self.animation.0);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.animation = packet_data::Animation(stream.read_i8());
    }
}

// Used for simple actions, like sneaking
pub struct PlayerAction {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub action: packet_data::PlayerAction,
}

impl PlayerAction {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::PLAYER_ACTION),
            entity_id: EntityId(0),
            action: packet_data::PlayerAction::default(),
        }
    }
}

impl PacketBehavior for PlayerAction {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_i8(self.action.0);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.action = packet_data::PlayerAction(stream.read_i8());
    }
}

// Used for spawning other players in the world
pub struct SpawnPlayer {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub username: String,
    pub q_position: Int32_3,
    pub q_rotation: Int8_2, // wire order: yaw first
    pub held_item_id: ItemId,
}

impl SpawnPlayer {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::SPAWN_PLAYER),
            entity_id: EntityId(0),
            username: String::new(),
            q_position: Int32_3::default(),
            q_rotation: Int8_2::default(),
            held_item_id: ItemId(0),
        }
    }
}

impl PacketBehavior for SpawnPlayer {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_string16(&self.username);
        stream.write_i32(self.q_position.x);
        stream.write_i32(self.q_position.y);
        stream.write_i32(self.q_position.z);
        stream.write_i8(self.q_rotation.x);
        stream.write_i8(self.q_rotation.y);
        stream.write_i16(self.held_item_id.0);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.username = stream.read_string16();
        self.q_position.x = stream.read_i32();
        self.q_position.y = stream.read_i32();
        self.q_position.z = stream.read_i32();
        self.q_rotation.x = stream.read_i8();
        self.q_rotation.y = stream.read_i8();
        self.held_item_id = ItemId(stream.read_i16());
    }
}

// Used for spawning items in the world
pub struct SpawnItem {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub item: ItemStack,
    pub q_position: Int32_3,
    pub q_rotation: Int8_3,
}

impl SpawnItem {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::SPAWN_ITEM),
            entity_id: EntityId(0),
            item: ItemStack::default(),
            q_position: Int32_3::default(),
            q_rotation: Int8_3::default(),
        }
    }
}

impl PacketBehavior for SpawnItem {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_i16(self.item.id.0);
        stream.write_i8(self.item.count);
        stream.write_i16(self.item.data);
        stream.write_i32(self.q_position.x);
        stream.write_i32(self.q_position.y);
        stream.write_i32(self.q_position.z);
        stream.write_i8(self.q_rotation.x);
        stream.write_i8(self.q_rotation.y);
        stream.write_i8(self.q_rotation.z);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.item.id = ItemId(stream.read_i16());
        self.item.count = stream.read_i8();
        self.item.data = stream.read_i16();
        self.q_position.x = stream.read_i32();
        self.q_position.y = stream.read_i32();
        self.q_position.z = stream.read_i32();
        self.q_rotation.x = stream.read_i8();
        self.q_rotation.y = stream.read_i8();
        self.q_rotation.z = stream.read_i8();
    }
}

// Used for collecting items visually
pub struct CollectItem {
    pub base: BasePacket,
    pub item_entity_id: EntityId,
    pub collector_entity_id: EntityId,
}

impl CollectItem {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::COLLECT_ITEM),
            item_entity_id: EntityId(0),
            collector_entity_id: EntityId(0),
        }
    }
}

impl PacketBehavior for CollectItem {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.item_entity_id.0);
        stream.write_i32(self.collector_entity_id.0);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.item_entity_id = EntityId(stream.read_i32());
        self.collector_entity_id = EntityId(stream.read_i32());
    }
}

// Used for spawning objects in the world
pub struct SpawnObject {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub object_type: packet_data::ObjectType,
    pub q_position: Int32_3,
    pub owner_entity_id: EntityId,
    pub q_velocity: Int16_3,
}

impl SpawnObject {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::SPAWN_OBJECT),
            entity_id: EntityId(0),
            object_type: packet_data::ObjectType::default(),
            q_position: Int32_3::default(),
            owner_entity_id: EntityId(0),
            q_velocity: Int16_3::default(),
        }
    }
}

impl PacketBehavior for SpawnObject {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_i8(self.object_type.0);
        stream.write_i32(self.q_position.x);
        stream.write_i32(self.q_position.y);
        stream.write_i32(self.q_position.z);
        stream.write_i32(self.owner_entity_id.0);
        if self.owner_entity_id.0 != 0 {
            stream.write_i16(self.q_velocity.x);
            stream.write_i16(self.q_velocity.y);
            stream.write_i16(self.q_velocity.z);
        }
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.object_type = packet_data::ObjectType(stream.read_i8());
        self.q_position.x = stream.read_i32();
        self.q_position.y = stream.read_i32();
        self.q_position.z = stream.read_i32();
        self.owner_entity_id = EntityId(stream.read_i32());
        if self.owner_entity_id.0 != 0 {
            self.q_velocity.x = stream.read_i16();
            self.q_velocity.y = stream.read_i16();
            self.q_velocity.z = stream.read_i16();
        }
    }
}

// Used for spawning mobs in the world
pub struct SpawnMob {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub mob_type: packet_data::MobType,
    pub q_position: Int32_3,
    pub q_rotation: Int8_2, // wire order: yaw first
    pub metadata: Vec<DataEntry>,
}

impl SpawnMob {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::SPAWN_MOB),
            entity_id: EntityId(0),
            mob_type: packet_data::MobType::default(),
            q_position: Int32_3::default(),
            q_rotation: Int8_2::default(),
            metadata: Vec::new(),
        }
    }
}

impl PacketBehavior for SpawnMob {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_i8(self.mob_type.0);
        stream.write_i32(self.q_position.x);
        stream.write_i32(self.q_position.y);
        stream.write_i32(self.q_position.z);
        stream.write_i8(self.q_rotation.x);
        stream.write_i8(self.q_rotation.y);
        stream.write_entity_metadata(&self.metadata);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.mob_type = packet_data::MobType(stream.read_i8());
        self.q_position.x = stream.read_i32();
        self.q_position.y = stream.read_i32();
        self.q_position.z = stream.read_i32();
        self.q_rotation.x = stream.read_i8();
        self.q_rotation.y = stream.read_i8();
        stream.read_entity_metadata(&mut self.metadata);
    }
}

// Used for spawning paintings in the world
pub struct SpawnPainting {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub title: String,
    pub position: Int32_3, // Block position
    pub direction: packet_data::PaintingDirection,
}

impl SpawnPainting {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::SPAWN_PAINTING),
            entity_id: EntityId(0),
            title: String::new(),
            position: Int32_3::default(),
            direction: packet_data::PaintingDirection::default(),
        }
    }
}

impl PacketBehavior for SpawnPainting {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_string16(&self.title);
        stream.write_i32(self.position.x);
        stream.write_i32(self.position.y);
        stream.write_i32(self.position.z);
        stream.write_i32(self.direction.0);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.title = stream.read_string16();
        self.position.x = stream.read_i32();
        self.position.y = stream.read_i32();
        self.position.z = stream.read_i32();
        self.direction = packet_data::PaintingDirection(stream.read_i32());
    }
}

// Unused, but exists for sending raw player input to the client/server
pub struct PlayerInput {
    pub base: BasePacket,
    pub direction: Float2,
    pub rotation: Float2,
    pub jumping: bool,
    pub sneaking: bool,
}

impl PlayerInput {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::PLAYER_INPUT),
            direction: Float2::default(),
            rotation: Float2::default(),
            jumping: false,
            sneaking: false,
        }
    }
}

impl PacketBehavior for PlayerInput {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_f32(self.direction.x);
        stream.write_f32(self.direction.y);
        stream.write_f32(self.rotation.x);
        stream.write_f32(self.rotation.y);
        stream.write_bool(self.jumping);
        stream.write_bool(self.sneaking);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.direction.x = stream.read_f32();
        self.direction.y = stream.read_f32();
        self.rotation.x = stream.read_f32();
        self.rotation.y = stream.read_f32();
        self.jumping = stream.read_bool();
        self.sneaking = stream.read_bool();
    }
}

// Used to update an entities velocity
pub struct EntityVelocity {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub velocity: Int16_3,
}

impl EntityVelocity {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::ENTITY_VELOCITY),
            entity_id: EntityId(0),
            velocity: Int16_3::default(),
        }
    }
}

impl PacketBehavior for EntityVelocity {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_i16(self.velocity.x);
        stream.write_i16(self.velocity.y);
        stream.write_i16(self.velocity.z);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.velocity.x = stream.read_i16();
        self.velocity.y = stream.read_i16();
        self.velocity.z = stream.read_i16();
    }
}

// Used to despawn an entity
pub struct DespawnEntity {
    pub base: BasePacket,
    pub entity_id: EntityId,
}

impl DespawnEntity {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::DESPAWN_ENTITY),
            entity_id: EntityId(0),
        }
    }
}

impl PacketBehavior for DespawnEntity {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
    }
}

// Unused, Base Packet for entity movement packets
pub struct EntityMovement {
    pub base: BasePacket,
}

impl EntityMovement {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::ENTITY_MOVEMENT),
        }
    }
}

impl PacketBehavior for EntityMovement {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
    }

    fn deserialize(&mut self, _stream: &mut NetworkStream) {}
}

// Used for setting an entitys relative position
pub struct EntityPosition {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub qr_position: Int8_3,
}

impl EntityPosition {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::ENTITY_POSITION),
            entity_id: EntityId(0),
            qr_position: Int8_3::default(),
        }
    }
}

impl PacketBehavior for EntityPosition {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_i8(self.qr_position.x);
        stream.write_i8(self.qr_position.y);
        stream.write_i8(self.qr_position.z);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.qr_position.x = stream.read_i8();
        self.qr_position.y = stream.read_i8();
        self.qr_position.z = stream.read_i8();
    }
}

// Used for setting an entitys rotation
pub struct EntityRotation {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub q_rotation: Int8_2, // wire order: yaw first
}

impl EntityRotation {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::ENTITY_ROTATION),
            entity_id: EntityId(0),
            q_rotation: Int8_2::default(),
        }
    }
}

impl PacketBehavior for EntityRotation {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_i8(self.q_rotation.x);
        stream.write_i8(self.q_rotation.y);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.q_rotation.x = stream.read_i8();
        self.q_rotation.y = stream.read_i8();
    }
}

// Used for setting an entitys relative position and rotation
pub struct EntityPositionAndRotation {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub qr_position: Int8_3,
    pub q_rotation: Int8_2, // wire order: yaw first
}

impl EntityPositionAndRotation {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::ENTITY_POSITION_AND_ROTATION),
            entity_id: EntityId(0),
            qr_position: Int8_3::default(),
            q_rotation: Int8_2::default(),
        }
    }
}

impl PacketBehavior for EntityPositionAndRotation {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_i8(self.qr_position.x);
        stream.write_i8(self.qr_position.y);
        stream.write_i8(self.qr_position.z);
        stream.write_i8(self.q_rotation.x);
        stream.write_i8(self.q_rotation.y);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.qr_position.x = stream.read_i8();
        self.qr_position.y = stream.read_i8();
        self.qr_position.z = stream.read_i8();
        self.q_rotation.x = stream.read_i8();
        self.q_rotation.y = stream.read_i8();
    }
}

// Used for setting an entitys absolute position
pub struct TeleportEntity {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub position: Int32_3,
    pub rotation: Int8_2, // wire order: yaw first
}

impl TeleportEntity {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::TELEPORT_ENTITY),
            entity_id: EntityId(0),
            position: Int32_3::default(),
            rotation: Int8_2::default(),
        }
    }
}

impl PacketBehavior for TeleportEntity {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_i32(self.position.x);
        stream.write_i32(self.position.y);
        stream.write_i32(self.position.z);
        stream.write_i8(self.rotation.x);
        stream.write_i8(self.rotation.y);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.position.x = stream.read_i32();
        self.position.y = stream.read_i32();
        self.position.z = stream.read_i32();
        self.rotation.x = stream.read_i8();
        self.rotation.y = stream.read_i8();
    }
}

// Used for some entity animations
pub struct EntityEvent {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub action: packet_data::EntityEvent,
}

impl EntityEvent {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::ENTITY_EVENT),
            entity_id: EntityId(0),
            action: packet_data::EntityEvent::default(),
        }
    }
}

impl PacketBehavior for EntityEvent {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_i8(self.action.0);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.action = packet_data::EntityEvent(stream.read_i8());
    }
}

// Used for mounting and dismounting entities
pub struct AddPassenger {
    pub base: BasePacket,
    pub passenger_entity_id: EntityId,
    pub vehicle_entity_id: EntityId,
}

impl AddPassenger {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::ADD_PASSENGER),
            passenger_entity_id: EntityId(0),
            vehicle_entity_id: EntityId(0),
        }
    }
}

impl PacketBehavior for AddPassenger {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.passenger_entity_id.0);
        stream.write_i32(self.vehicle_entity_id.0);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.passenger_entity_id = EntityId(stream.read_i32());
        self.vehicle_entity_id = EntityId(stream.read_i32());
    }
}

// Used for mounting and dismounting entities
pub struct EntityMetadata {
    pub base: BasePacket,
    pub entity_id: EntityId,
    pub metadata: Vec<DataEntry>,
}

impl EntityMetadata {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::ENTITY_METADATA),
            entity_id: EntityId(0),
            metadata: Vec::new(),
        }
    }
}

// TODO: Ideally this'd immediately read/write
// the relevant data for the entity behind the ID,
// but for now we'll just read it into the metadata vector

impl PacketBehavior for EntityMetadata {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_entity_metadata(&self.metadata);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        stream.read_entity_metadata(&mut self.metadata);
    }
}

// Tells the client to allocate or free a chunk slot. Must be sent before ChunkData
pub struct SetChunkVisibility {
    pub base: BasePacket,
    pub pos: Int32_2,
    pub visible: bool,
}

impl SetChunkVisibility {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::SET_CHUNK_VISIBILITY),
            pos: Int32_2::default(),
            visible: false,
        }
    }
}

impl PacketBehavior for SetChunkVisibility {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.pos.x);
        stream.write_i32(*self.pos.z());
        stream.write_bool(self.visible);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.pos.x = stream.read_i32();
        *self.pos.z_mut() = stream.read_i32();
        self.visible = stream.read_bool();
    }
}

// Sends compressed chunk data; always preceded by SetChunkVisibility
pub struct ChunkData {
    pub base: BasePacket,
    pub pos: SlimInt3<i16>,
    pub size: TriNumber<u8>,
    pub compressed_data: Vec<u8>,
}

impl ChunkData {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::CHUNK),
            pos: SlimInt3::new(0, 0, 0),
            size: TriNumber::new((CHUNK_WIDTH - 1) as u8, (CHUNK_HEIGHT - 1) as u8, (CHUNK_WIDTH - 1) as u8),
            compressed_data: Vec::new(),
        }
    }
}

impl PacketBehavior for ChunkData {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.pos.x);
        stream.write_i16(self.pos.y);
        stream.write_i32(self.pos.z);
        stream.write_u8(self.size.x);
        stream.write_u8(self.size.y);
        stream.write_u8(self.size.z);
        stream.write_i32(self.compressed_data.len() as i32);
        stream.write_bytes(&self.compressed_data);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.pos.x = stream.read_i32();
        self.pos.y = stream.read_i16();
        self.pos.z = stream.read_i32();
        self.size.x = stream.read_u8();
        self.size.y = stream.read_u8();
        self.size.z = stream.read_u8();
        let length = stream.read_i32();
        self.compressed_data = vec![0u8; length as usize];
        stream.read_bytes(&mut self.compressed_data);
    }
}

// Used to set multiple blocks in a small area
pub struct SetMultipleBlocks {
    pub base: BasePacket,
    pub chunk_position: Int32_2,
    pub number_of_blocks: i16,
    pub block_coordinates: Vec<i16>,
    pub block_types: Vec<BlockType>,
    pub block_metadata: Vec<i8>, // Nibbles
}

impl SetMultipleBlocks {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::SET_MULTIPLE_BLOCKS),
            chunk_position: Int32_2::default(),
            number_of_blocks: 0,
            block_coordinates: Vec::new(),
            block_types: Vec::new(),
            block_metadata: Vec::new(),
        }
    }
}

impl PacketBehavior for SetMultipleBlocks {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.chunk_position.x);
        stream.write_i32(*self.chunk_position.z());
        stream.write_i16(self.number_of_blocks);
        for i in 0..self.number_of_blocks {
            stream.write_i16(self.block_coordinates[i as usize]);
        }
        for i in 0..self.number_of_blocks {
            stream.write_i8(self.block_types[i as usize].0);
        }
        for i in 0..self.number_of_blocks {
            stream.write_i8(self.block_metadata[i as usize]);
        }
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.chunk_position.x = stream.read_i32();
        *self.chunk_position.z_mut() = stream.read_i32();
        self.number_of_blocks = stream.read_i16();
        self.block_coordinates = vec![0i16; self.number_of_blocks as usize];
        self.block_types = vec![BlockType::default(); self.number_of_blocks as usize];
        self.block_metadata = vec![0i8; self.number_of_blocks as usize];
        for i in 0..self.number_of_blocks {
            self.block_coordinates[i as usize] = stream.read_i16();
        }
        for i in 0..self.number_of_blocks {
            self.block_types[i as usize] = BlockType(stream.read_i8());
        }
        for i in 0..self.number_of_blocks {
            self.block_metadata[i as usize] = stream.read_i8();
        }
    }
}

// Used to set a singular block
pub struct SetBlock {
    pub base: BasePacket,
    pub position: SlimInt3<i8>,
    pub block: Block,
}

impl SetBlock {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::SET_BLOCK),
            position: SlimInt3::new(0, 0, 0),
            block: Block::default(),
        }
    }
}

impl PacketBehavior for SetBlock {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.position.x);
        stream.write_i8(self.position.y);
        stream.write_i32(self.position.z);
        stream.write_i8(self.block.r#type.0);
        stream.write_u8(self.block.data);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.position.x = stream.read_i32();
        self.position.y = stream.read_i8();
        self.position.z = stream.read_i32();
        self.block.r#type = BlockType(stream.read_i8());
        self.block.data = stream.read_u8();
    }
}

// Used to set a singular block
pub struct BlockEvent {
    pub base: BasePacket,
    pub position: SlimInt3<i8>,
    pub instrument_state: i8,
    pub pitch_direction: i8,
}

impl BlockEvent {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::BLOCK_EVENT),
            position: SlimInt3::new(0, 0, 0),
            instrument_state: 0,
            pitch_direction: 0,
        }
    }

    pub fn instrument(&self) -> packet_data::NoteInstrument {
        packet_data::NoteInstrument(self.instrument_state as u8)
    }

    pub fn pitch(&self) -> packet_data::NotePitch {
        packet_data::NotePitch(self.pitch_direction as u8)
    }

    pub fn state(&self) -> packet_data::PistonState {
        packet_data::PistonState(self.instrument_state)
    }

    pub fn direction(&self) -> packet_data::PistonDirection {
        packet_data::PistonDirection(self.pitch_direction)
    }
}

impl PacketBehavior for BlockEvent {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.position.x);
        stream.write_i8(self.position.y);
        stream.write_i32(self.position.z);
        stream.write_i8(self.instrument_state);
        stream.write_i8(self.pitch_direction);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.position.x = stream.read_i32();
        self.position.y = stream.read_i8();
        self.position.z = stream.read_i32();
        self.instrument_state = stream.read_i8();
        self.pitch_direction = stream.read_i8();
    }
}

// Used for explosions
pub struct Explosion {
    pub base: BasePacket,
    pub position: Vec3,
    pub radius: f32,
    pub number_of_destroyed_blocks: i32,
    pub destroyed_blocks: Vec<i8>,
}

impl Explosion {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::EXPLOSION),
            position: Vec3::default(),
            radius: 0.0,
            number_of_destroyed_blocks: 0,
            destroyed_blocks: Vec::new(),
        }
    }
}

impl PacketBehavior for Explosion {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_f64(self.position.x);
        stream.write_f64(self.position.y);
        stream.write_f64(self.position.z);
        stream.write_f32(self.radius);
        stream.write_i32(self.destroyed_blocks.len() as i32);
        let bytes: Vec<u8> = self.destroyed_blocks.iter().map(|&value| value as u8).collect();
        stream.write_bytes(&bytes);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.position.x = stream.read_f64();
        self.position.y = stream.read_f64();
        self.position.z = stream.read_f64();
        self.radius = stream.read_f32();
        self.number_of_destroyed_blocks = stream.read_i32();
        let mut bytes = vec![0u8; self.number_of_destroyed_blocks as usize];
        stream.read_bytes(&mut bytes);
        self.destroyed_blocks = bytes.into_iter().map(|value| value as i8).collect();
    }
}

// Used to trigger world events, such as sound effects
pub struct WorldEvent {
    pub base: BasePacket,
    pub event_id: packet_data::WorldEvent,
    pub position: SlimInt3<i8>,
    pub data: i32,
}

impl WorldEvent {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::WORLD_EVENT),
            event_id: packet_data::WorldEvent::default(),
            position: SlimInt3::new(0, 0, 0),
            data: 0,
        }
    }
}

impl PacketBehavior for WorldEvent {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.position.x);
        stream.write_i32(self.position.z);
        stream.write_i8(self.position.y);
        stream.write_i32(self.event_id.0);
        stream.write_i32(self.data);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.position.x = stream.read_i32();
        self.position.z = stream.read_i32();
        self.position.y = stream.read_i8();
        self.event_id = packet_data::WorldEvent(stream.read_i32());
        self.data = stream.read_i32();
    }
}

// Used to trigger global game events, such as rain
pub struct GameEvent {
    pub base: BasePacket,
    pub event_id: packet_data::GameEvent,
}

impl GameEvent {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::GAME_EVENT),
            event_id: packet_data::GameEvent::default(),
        }
    }
}

impl PacketBehavior for GameEvent {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i8(self.event_id.0);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.event_id = packet_data::GameEvent(stream.read_i8());
    }
}

// Used to spawn a lightning bolt
pub struct LightningBolt {
    pub base: BasePacket,
    pub entity_id: EntityId,
    // This is only ever "1", which means lightning
    pub entity_type: i8,
    pub position: Int32_3,
}

impl LightningBolt {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::LIGHTNING_BOLT),
            entity_id: EntityId(0),
            entity_type: 1,
            position: Int32_3::default(),
        }
    }
}

impl PacketBehavior for LightningBolt {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.entity_id.0);
        stream.write_i8(self.entity_type);
        stream.write_i32(self.position.x);
        stream.write_i32(self.position.y);
        stream.write_i32(self.position.z);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.entity_id = EntityId(stream.read_i32());
        self.entity_type = stream.read_i8();
        self.position.x = stream.read_i32();
        self.position.y = stream.read_i32();
        self.position.z = stream.read_i32();
    }
}

// Used for signaling when a container is opened
pub struct OpenContainer {
    pub base: BasePacket,
    pub window_id: WindowId,
    pub window_type: packet_data::WindowType,
    pub title: String, // This is String8!!
    pub slot_count: i8,
}

impl OpenContainer {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::OPEN_CONTAINER),
            window_id: WindowId(0),
            window_type: packet_data::WindowType::default(),
            title: String::new(),
            slot_count: 0,
        }
    }
}

impl PacketBehavior for OpenContainer {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i8(self.window_id.0);
        stream.write_i8(self.window_type.0);
        stream.write_string8(&self.title);
        stream.write_i8(self.slot_count);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.window_id = WindowId(stream.read_i8());
        self.window_type = packet_data::WindowType(stream.read_i8());
        self.title = stream.read_string8();
        self.slot_count = stream.read_i8();
    }
}

// Used for signaling when a container was closed
pub struct CloseContainer {
    pub base: BasePacket,
    pub window_id: WindowId,
}

impl CloseContainer {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::CLOSE_CONTAINER),
            window_id: WindowId(0),
        }
    }
}

impl PacketBehavior for CloseContainer {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i8(self.window_id.0);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.window_id = WindowId(stream.read_i8());
    }
}

// Used for signaling when a slot was clicked
pub struct ClickSlot {
    pub base: BasePacket,
    pub window_id: WindowId,
    pub slot_id: NetworkSlotId,
    pub right_click: bool,
    pub transaction_id: TransactionId,
    pub shift: bool,
    pub item: ItemStack,
}

impl ClickSlot {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::CLICK_SLOT),
            window_id: WindowId(0),
            slot_id: NetworkSlotId(0),
            right_click: false,
            transaction_id: TransactionId(0),
            shift: false,
            item: ItemStack::default(),
        }
    }
}

impl PacketBehavior for ClickSlot {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i8(self.window_id.0);
        stream.write_i16(self.slot_id.0);
        stream.write_bool(self.right_click);
        stream.write_i16(self.transaction_id.0);
        stream.write_bool(self.shift);
        stream.write_i16(self.item.id.0);
        if self.item.id != items::INVALID {
            stream.write_i8(self.item.count);
            stream.write_i16(self.item.data);
        }
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.window_id = WindowId(stream.read_i8());
        self.slot_id = NetworkSlotId(stream.read_i16());
        self.right_click = stream.read_bool();
        self.transaction_id = TransactionId(stream.read_i16());
        self.shift = stream.read_bool();
        self.item.id = ItemId(stream.read_i16());
        if self.item.id != items::INVALID {
            self.item.count = stream.read_i8();
            self.item.data = stream.read_i16();
        }
    }
}

// Used for setting the contents of a slot
pub struct SetSlot {
    pub base: BasePacket,
    pub window_id: WindowId,
    pub slot_id: NetworkSlotId,
    pub item: ItemStack,
}

impl SetSlot {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::SET_SLOT),
            window_id: WindowId(0),
            slot_id: NetworkSlotId(0),
            item: ItemStack::default(),
        }
    }
}

impl PacketBehavior for SetSlot {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i8(self.window_id.0);
        stream.write_i16(self.slot_id.0);
        stream.write_i16(self.item.id.0);
        if self.item.id != items::INVALID {
            stream.write_i8(self.item.count);
            stream.write_i16(self.item.data);
        }
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.window_id = WindowId(stream.read_i8());
        self.slot_id = NetworkSlotId(stream.read_i16());
        self.item.id = ItemId(stream.read_i16());
        if self.item.id != items::INVALID {
            self.item.count = stream.read_i8();
            self.item.data = stream.read_i16();
        }
    }
}

// Possibly we do this by passing in the whole inventory?
// Used for filling a container with data
pub struct FillContainer {
    pub base: BasePacket,
    pub window_id: WindowId,
    pub items: Vec<ItemStack>,
}

impl FillContainer {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::FILL_CONTAINER),
            window_id: WindowId(0),
            items: Vec::new(),
        }
    }
}

impl PacketBehavior for FillContainer {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i8(self.window_id.0);
        stream.write_i16(self.items.len() as i16);
        for item in &self.items {
            stream.write_i16(item.id.0);
            if item.id != items::INVALID {
                stream.write_i8(item.count);
                stream.write_i16(item.data);
            }
        }
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.window_id = WindowId(stream.read_i8());
        let number_of_slots = stream.read_i16() as usize;
        self.items = vec![ItemStack::default(); number_of_slots];
        for i in 0..number_of_slots {
            self.items[i].id = ItemId(stream.read_i16());
            if self.items[i].id != items::INVALID {
                self.items[i].count = stream.read_i8();
                self.items[i].data = stream.read_i16();
            }
        }
    }
}

pub struct ContainerDataEntry {
    pub r#type: packet_data::ContainerDataType,
    pub value: i16,
}

impl Default for ContainerDataEntry {
    fn default() -> Self {
        Self {
            r#type: packet_data::ContainerDataType::default(),
            value: 0,
        }
    }
}

// Used for setting data for containers, such as furnace progress
pub struct ContainerData {
    pub base: BasePacket,
    pub window_id: WindowId,
    pub container_data: ContainerDataEntry,
}

impl ContainerData {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::CONTAINER_DATA),
            window_id: WindowId(0),
            container_data: ContainerDataEntry::default(),
        }
    }
}

impl PacketBehavior for ContainerData {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i8(self.window_id.0);
        stream.write_i16(self.container_data.r#type.0);
        stream.write_i16(self.container_data.value);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.window_id = WindowId(stream.read_i8());
        self.container_data.r#type = packet_data::ContainerDataType(stream.read_i16());
        self.container_data.value = stream.read_i16();
    }
}

// Used for checking if the performed transaction was valid and got through successfully
pub struct ContainerTransaction {
    pub base: BasePacket,
    pub window_id: WindowId,
    pub transaction_id: TransactionId,
    pub accepted: bool,
}

impl ContainerTransaction {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::CONTAINER_TRANSACTION),
            window_id: WindowId(0),
            transaction_id: TransactionId(0),
            accepted: false,
        }
    }
}

impl PacketBehavior for ContainerTransaction {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i8(self.window_id.0);
        stream.write_i16(self.transaction_id.0);
        stream.write_bool(self.accepted);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.window_id = WindowId(stream.read_i8());
        self.transaction_id = TransactionId(stream.read_i16());
        self.accepted = stream.read_bool();
    }
}

// Use for updating the text on signs
pub struct UpdateSign {
    pub base: BasePacket,
    pub position: SlimInt3<i16>,
    pub lines: [String; 4],
}

impl UpdateSign {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::UPDATE_SIGN),
            position: SlimInt3::new(0, 0, 0),
            lines: [String::new(), String::new(), String::new(), String::new()],
        }
    }
}

impl PacketBehavior for UpdateSign {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.position.x);
        stream.write_i16(self.position.y);
        stream.write_i32(self.position.z);
        stream.write_string16(&self.lines[0]);
        stream.write_string16(&self.lines[1]);
        stream.write_string16(&self.lines[2]);
        stream.write_string16(&self.lines[3]);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.position.x = stream.read_i32();
        self.position.y = stream.read_i16();
        self.position.z = stream.read_i32();
        self.lines[0] = stream.read_string16();
        self.lines[1] = stream.read_string16();
        self.lines[2] = stream.read_string16();
        self.lines[3] = stream.read_string16();
    }
}

// Used for updating custom item data, only used by maps
pub struct ItemData {
    pub base: BasePacket,
    pub item_id: ItemId,
    pub map_id: MapId,
    pub data: Vec<u8>,
}

impl ItemData {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::ITEM_DATA),
            item_id: ItemId(0),
            map_id: MapId(0),
            data: Vec::new(),
        }
    }
}

impl PacketBehavior for ItemData {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i16(self.item_id.0);
        stream.write_i16(self.map_id.0);
        stream.write_u8(self.data.len() as u8);
        stream.write_bytes(&self.data);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.item_id = ItemId(stream.read_i16());
        self.map_id = MapId(stream.read_i16());
        let size = stream.read_u8();
        self.data = vec![0u8; size as usize];
        stream.read_bytes(&mut self.data);
    }
}

// Used for changing the value of a statistic
pub struct IncrementStatistic {
    pub base: BasePacket,
    pub statistic_id: i32, // TODO: Replace with Enum
    pub amount: i8,
}

impl IncrementStatistic {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::INCREMENT_STATISTIC),
            statistic_id: 0,
            amount: 0,
        }
    }
}

impl PacketBehavior for IncrementStatistic {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_i32(self.statistic_id);
        stream.write_i8(self.amount);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.statistic_id = stream.read_i32();
        self.amount = stream.read_i8();
    }
}

// Used for disconnecting with a disconnect reason
pub struct Disconnect {
    pub base: BasePacket,
    pub reason: String,
}

impl Disconnect {
    pub fn new() -> Self {
        Self {
            base: BasePacket::new(packet_ids::DISCONNECT),
            reason: String::new(),
        }
    }
}

impl PacketBehavior for Disconnect {
    fn base(&self) -> &BasePacket {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn serialize(&self, stream: &mut NetworkStream) {
        stream.write_u8(self.base.id.0);
        stream.write_string16(&self.reason);
    }

    fn deserialize(&mut self, stream: &mut NetworkStream) {
        self.reason = stream.read_string16();
    }
}
