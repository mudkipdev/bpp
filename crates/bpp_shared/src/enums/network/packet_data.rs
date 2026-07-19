/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use crate::inventory::item_stack::ItemStack;
use crate::numeric_structs::Int32_3;

// Used by the Mine Block Packet (0x0E)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MineStatus(pub u8);

pub const DIGGING_STARTED: MineStatus = MineStatus(0);
pub const DIGGING_FINISHED: MineStatus = MineStatus(2);
pub const DROPPED_ITEM: MineStatus = MineStatus(4);

// Used by Mine and Place Block Packets (0x0E and 0x0F)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FaceDirection(pub i8);

pub const INVALID_USE: FaceDirection = FaceDirection(-1);
pub const Y_MINUS: FaceDirection = FaceDirection(0);
pub const Y_PLUS: FaceDirection = FaceDirection(1);
pub const Z_MINUS: FaceDirection = FaceDirection(2);
pub const Z_PLUS: FaceDirection = FaceDirection(3);
pub const X_MINUS: FaceDirection = FaceDirection(4);
pub const X_PLUS: FaceDirection = FaceDirection(5);

// Used by the Interact with Block Packet (0x11)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockInteraction(pub u8);

pub const SLEEPING: BlockInteraction = BlockInteraction(0);

// Used by the Animation Packet (0x12)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Animation(pub i8);

pub const NONE: Animation = Animation(0);
// The player swings their arm (e.g. when attacking or using an item)
pub const PUNCH: Animation = Animation(1);
// While this still works as intended in b1.7.3,
// the server does not send it.
// Instead Entity Event (0x26) is used
pub const DAMAGE: Animation = Animation(2);
// The player is forced to leave the bed
pub const LEAVE_BED: Animation = Animation(3);
// An animation that seems to no longer
// have any connected functionality in b1.7.3
pub const UNUSED: Animation = Animation(4);

// Used by the Player Action Packet (0x13)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerAction(pub i8);

pub const START_SNEAKING: PlayerAction = PlayerAction(1);
pub const STOP_SNEAKING: PlayerAction = PlayerAction(2);
pub const STOP_SLEEPING: PlayerAction = PlayerAction(3);

// Used by the Spawn Object Packet (0x17)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectType(pub i8);

pub const BOAT: ObjectType = ObjectType(1);
pub const MINECART: ObjectType = ObjectType(10);
pub const STORAGE_MINECART: ObjectType = ObjectType(11);
pub const FURNACE_MINECART: ObjectType = ObjectType(12);
pub const LIT_TNT: ObjectType = ObjectType(50);
pub const ARROW: ObjectType = ObjectType(60);
pub const THROWN_SNOWBALL: ObjectType = ObjectType(61);
pub const THROWN_EGG: ObjectType = ObjectType(62);
pub const FALLING_SAND: ObjectType = ObjectType(70);
pub const FALLING_GRAVEL: ObjectType = ObjectType(71);
pub const FISHING_BOBBER: ObjectType = ObjectType(90);

// Used by the Spawn Mob Packet (0x18)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MobType(pub i8);

pub const CREEPER: MobType = MobType(50);
pub const SKELETON: MobType = MobType(51);
pub const SPIDER: MobType = MobType(52);
pub const GIANT_ZOMBIE: MobType = MobType(53);
pub const ZOMBIE: MobType = MobType(54);
pub const SLIME: MobType = MobType(55);
pub const GHAST: MobType = MobType(56);
pub const ZOMBIE_PIGMAN: MobType = MobType(57);
pub const PIG: MobType = MobType(90);
pub const SHEEP: MobType = MobType(91);
pub const COW: MobType = MobType(92);
pub const CHICKEN: MobType = MobType(93);
pub const SQUID: MobType = MobType(94);
pub const WOLF: MobType = MobType(95);

// Used by the Spawn Painting Packet (0x19)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PaintingDirection(pub i32);

pub const MINUS_Z: PaintingDirection = PaintingDirection(0);
pub const MINUS_X: PaintingDirection = PaintingDirection(1);
pub const PLUS_Z: PaintingDirection = PaintingDirection(2);
pub const PLUS_X: PaintingDirection = PaintingDirection(3);

// Used by the Entity Event Packet (0x26)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityEvent(pub i8);

pub const HURT: EntityEvent = EntityEvent(2);
pub const DEATH: EntityEvent = EntityEvent(3);
// Wolf specific events
pub const SMOKE_PARTICLES: EntityEvent = EntityEvent(6);
pub const HEART_PARTICLES: EntityEvent = EntityEvent(7);
pub const START_SHAKING: EntityEvent = EntityEvent(8);

// Only used for the Entity Metadata pseudo-VM
pub mod entity_metadata {
    use super::{ItemStack, Int32_3};

    pub const END: u8 = 0x7F;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Type(pub u8);

    pub const BYTE: Type = Type(0);
    pub const SHORT: Type = Type(1);
    pub const INTEGER: Type = Type(2);
    pub const FLOAT: Type = Type(3);
    pub const STRING: Type = Type(4);
    pub const ITEM: Type = Type(5);
    pub const COORDINATES: Type = Type(6);

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Flags(pub u8);

    pub const BURNING: Flags = Flags(0);
    pub const SNEAKING: Flags = Flags(1);
    pub const RIDING: Flags = Flags(2);

    pub struct DataEntry {
        pub r#type: Type,
        pub index: u8,

        pub value: Value,
    }

    pub enum Value {
        Byte(i8),
        Short(i16),
        Integer(i32),
        Float(f32),
        String(String),
        Item(ItemStack),
        Coordinates(Int32_3),
    }
}

// Used by Block Action Packet (0x36)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoteInstrument(pub u8);

pub const HARP: NoteInstrument = NoteInstrument(0);
pub const BASS: NoteInstrument = NoteInstrument(1);
pub const SNARE_DRUM: NoteInstrument = NoteInstrument(2);
pub const HI_HAT: NoteInstrument = NoteInstrument(3);
pub const BASS_DRUM: NoteInstrument = NoteInstrument(4);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NotePitch(pub u8);

pub const LOW_F_SHARP: NotePitch = NotePitch(0);
pub const LOW_G: NotePitch = NotePitch(1);
pub const LOW_G_SHARP: NotePitch = NotePitch(2);
pub const LOW_A: NotePitch = NotePitch(3);
pub const LOW_A_SHARP: NotePitch = NotePitch(4);
pub const LOW_B: NotePitch = NotePitch(5);
pub const LOW_C: NotePitch = NotePitch(6);
pub const LOW_C_SHARP: NotePitch = NotePitch(7);
pub const LOW_D: NotePitch = NotePitch(8);
pub const LOW_D_SHARP: NotePitch = NotePitch(9);
pub const LOW_E: NotePitch = NotePitch(10);
pub const LOW_F: NotePitch = NotePitch(11);
pub const HIGH_F_SHARP: NotePitch = NotePitch(12);
pub const HIGH_G: NotePitch = NotePitch(13);
pub const HIGH_G_SHARP: NotePitch = NotePitch(14);
pub const HIGH_A: NotePitch = NotePitch(15);
pub const HIGH_A_SHARP: NotePitch = NotePitch(16);
pub const HIGH_B: NotePitch = NotePitch(17);
pub const HIGH_C: NotePitch = NotePitch(18);
pub const HIGH_C_SHARP: NotePitch = NotePitch(19);
pub const HIGH_D: NotePitch = NotePitch(20);
pub const HIGH_D_SHARP: NotePitch = NotePitch(21);
pub const HIGH_E: NotePitch = NotePitch(22);
pub const HIGH_F: NotePitch = NotePitch(23);
pub const LAST_F_SHARP: NotePitch = NotePitch(24);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PistonState(pub i8);

pub const EXTEND: PistonState = PistonState(0);
pub const RETRACT: PistonState = PistonState(1);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PistonDirection(pub i8);

pub const DOWN: PistonDirection = PistonDirection(0);
pub const UP: PistonDirection = PistonDirection(1);
pub const EAST: PistonDirection = PistonDirection(2);
pub const WEST: PistonDirection = PistonDirection(3);
pub const NORTH: PistonDirection = PistonDirection(4);
pub const SOUTH: PistonDirection = PistonDirection(5);

// Used by the World Event Packet (0x3D)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldEvent(pub i32);

// Button click sound
pub const CLICK2: WorldEvent = WorldEvent(1000);
// Alt. button click sound
pub const CLICK1: WorldEvent = WorldEvent(1001);
// Bow shooting sound
pub const BOW_FIRE: WorldEvent = WorldEvent(1002);
// Door opening/closing sound
pub const DOOR_TOGGLE: WorldEvent = WorldEvent(1003);
// Extinguish fire sound
pub const EXTINGUISH: WorldEvent = WorldEvent(1004);
// Record playing sound, requires music disc item id as parameter
pub const RECORD_PLAY: WorldEvent = WorldEvent(1005);
// Smoke particle effect, requires index for a position
pub const SMOKE: WorldEvent = WorldEvent(2000);
// Block breaking particle effect, requires block id
pub const BLOCK_BREAK: WorldEvent = WorldEvent(2001);

// Used by Game Event Packet (0x46)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GameEvent(pub i8);

pub const INVALID_BED: GameEvent = GameEvent(0);
pub const START_RAINING: GameEvent = GameEvent(1);
pub const STOP_RAINING: GameEvent = GameEvent(2);

// Used by Open Container (0x64)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowType(pub i8);

pub const CHEST: WindowType = WindowType(0);
pub const CRAFTING_TABLE: WindowType = WindowType(1);
pub const FURNACE: WindowType = WindowType(2);
pub const DISPENSER: WindowType = WindowType(3);

// Used by Container Data (0x69)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContainerDataType(pub i16);

pub const SMELTING_PROGRESS: ContainerDataType = ContainerDataType(0);
pub const FUEL_REMAINING: ContainerDataType = ContainerDataType(1);
pub const FUEL_DURATION: ContainerDataType = ContainerDataType(2);
