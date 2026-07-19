/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::sync::OnceLock;

use crate::base_types::{ItemAmount, ItemDamage, ItemId};
use crate::blocks::materials::Material;
use crate::entities::entity::Entity;
use crate::enums::network::packet_data::FaceDirection;
use crate::helpers::aabb::{AABB, CollisionShape};
use crate::helpers::java::java_random::Random;
use crate::numeric_structs::{Int3, Vec3};
use crate::world::world::WorldManager;

// Some fluid specific stuff
pub fn get_fluid_percent_air(meta: u8) -> f32 {
    let mut meta = meta;
    if meta >= 8 {
        meta = 0;
    }

    (meta as f32 + 1.0) / 9.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StepSound {
    Stone, // default, also metal (different pitch)
    Wood,
    Gravel,
    Grass,
    Sand,
    Cloth,
    Glass,
}

impl Default for StepSound {
    fn default() -> Self {
        StepSound::Stone
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BlockProperties {
    pub material: Material,
    pub step_sound: StepSound,

    pub light_emission: u8, // 0-15
    pub light_opacity: u8,  // 0 = transparent, 255 = fully opaque
    pub tick_rate: i32,

    pub hardness: f32,         // -1 = unbreakable (bedrock)
    pub resistance: f32,       // blast resistance
    pub slipperiness: f32,     // default friction, ice = 0.98f
    pub particle_gravity: f32, // how fast break particles fall

    pub is_collidable: bool,
    pub is_opaque_cube: bool,
    pub is_normal_cube: bool,
    pub render_as_normal_block: bool,
    pub ticks_on_load: bool,
    pub can_block_grass: bool,
    pub notify_neighbors_on_meta_change: bool,
    pub notify_self_on_meta_change: bool,
    pub enable_stats: bool, // false = breaking doesn't count for achievements
}

impl Default for BlockProperties {
    fn default() -> Self {
        Self {
            material: Material::rock(),
            step_sound: StepSound::Stone,
            light_emission: 0,
            light_opacity: 255,
            tick_rate: 10,
            hardness: 1.0,
            resistance: 5.0,
            slipperiness: 0.6,
            particle_gravity: 1.0,
            is_collidable: true,
            is_opaque_cube: true,
            is_normal_cube: true,
            render_as_normal_block: true,
            ticks_on_load: false,
            can_block_grass: true,
            notify_neighbors_on_meta_change: true,
            notify_self_on_meta_change: true,
            enable_stats: true,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct BlockBehavior {
    // Called when we need to get the AABB for the selection box
    pub get_selection_box: Option<fn(u8) -> AABB>,

    // Called when we need to check for ray intersections for selection
    pub get_ray_bounds: Option<fn(u8) -> AABB>,

    // Called when we need to check the collision of this block
    pub get_collider: Option<fn(u8) -> CollisionShape>,

    // Called each random tick if ticksOnLoad = true
    pub on_tick: Option<fn(&mut WorldManager, Int3, u8, &mut Random)>,

    // Called when block is placed by world gen or setBlock
    pub on_block_added: Option<fn(&mut WorldManager, Int3)>,

    // Called when block is removed
    pub on_block_removal: Option<fn(&mut WorldManager, Int3)>,

    // Called when a neighboring block changes
    pub on_neighbor_block_change: Option<fn(&mut WorldManager, Int3)>,

    // Called when a player left-clicks the block (not breaks, just clicks)
    // pos is where that block that is interacted with is
    pub on_block_clicked: Option<fn(&mut WorldManager, Int3)>,

    // Called when a player right-clicks the block
    // Return true if we allow the player to still place their held block
    pub on_block_activated: Option<fn(&mut WorldManager, Int3) -> bool>,

    // Called when block is placed by a player
    pub on_block_placed: Option<fn(&mut WorldManager, Int3, &mut Entity, FaceDirection)>,

    // Called when player breaks the block
    pub on_block_destroyed_by_player: Option<fn(&mut WorldManager, Int3, &mut Entity)>,

    // Called when an explosion destroys the block
    pub on_block_destroyed_by_explosion: Option<fn(&mut WorldManager, Int3)>,

    // Called when an entity walks on top of the block
    pub on_entity_walking: Option<fn(&mut WorldManager, Int3, &mut Entity)>,

    // Called when an entity collides with the block (cactus damage, etc.)
    pub on_entity_collided_with_block: Option<fn(&mut WorldManager, Int3, &mut Entity)>,

    // Called when we need to find how this block would contribute to the push vector of an entity
    pub velocity_to_add_to_entity: Option<fn(&mut WorldManager, Int3, &mut Vec3)>,

    // What item/block this drops when broken
    pub id_dropped: Option<fn(u8, &mut Random) -> ItemId>,

    // The data value of the dropped item
    pub damage_dropped: Option<fn(u8) -> ItemDamage>,

    // How many items drop
    pub quantity_dropped: Option<fn(&mut Random) -> ItemAmount>,
}

// Indexed by block ID, populated by register_all()
static BLOCK_PROPERTIES: OnceLock<[BlockProperties; 256]> = OnceLock::new();
static BLOCK_BEHAVIORS: OnceLock<[BlockBehavior; 256]> = OnceLock::new();

pub fn block_properties() -> &'static [BlockProperties; 256] {
    BLOCK_PROPERTIES.get().expect("block_registration::register_all() must run before the block tables are read")
}

pub fn block_behaviors() -> &'static [BlockBehavior; 256] {
    BLOCK_BEHAVIORS.get().expect("block_registration::register_all() must run before the block tables are read")
}

// Called once at startup before anything reads from the tables
pub fn set_tables(properties: [BlockProperties; 256], behaviors: [BlockBehavior; 256]) {
    let _ = BLOCK_PROPERTIES.set(properties);
    let _ = BLOCK_BEHAVIORS.set(behaviors);
}
