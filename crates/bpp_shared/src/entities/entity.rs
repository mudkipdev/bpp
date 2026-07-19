/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::collections::HashMap;
use std::sync::{Mutex, Weak};

use crate::base_types::{EntityId, TickTime};
use crate::blocks::block_properties::{self, BlockProperties};
use crate::blocks::materials::Material;
use crate::entities::entity_manager::EntityManager;
use crate::entities::entity_player::PlayerEntity;
use crate::enums::dimensions::Dimension;
use crate::enums::entities::EntityType;
use crate::enums::network::packet_data::entity_metadata::DataEntry;
use crate::helpers::aabb::AABB;
use crate::helpers::java::java_math::MathHelper;
use crate::helpers::java::java_random::Random;
use crate::nbt::nbt::{TAG_DOUBLE, TAG_FLOAT, Tag};
use crate::numeric_structs::{Int3, Vec2, Vec3};
use crate::world::world::WorldManager;

// Constants pulled from the betaWiki!
// https://pixelbrush.dev/beta-wiki/entities/movement
// I <3 BETA WIKI!
pub const KNOCKBACK_VELOCITY_DAMPENING: f32 = 0.5;
pub const HORIZONTAL_KNOCKBACK: f32 = 0.4;
pub const VERTICAL_KNOCKBACK: f32 = 0.4;

pub const COBWEB_VERTICAL_DRAG: f32 = 0.05;
pub const COBWEB_HORIZONTAL_DRAG: f32 = 0.25;

pub const LADDER_MAX_HORIZONTAL: f32 = 0.15;
pub const LADDER_MAX_DESCENT: f32 = 0.15;
pub const LADDER_SNEAK_DESCENT: f32 = 0.0;
pub const LADDER_WALL_BOOST: f32 = 0.2;

pub const WATER_DRAG: f32 = 0.8;
pub const LAVA_DRAG: f32 = 0.5;
pub const FLUID_GRAVITY: f32 = 0.02;
pub const FLUID_ACCELERATION: f32 = 0.02;
pub const FLUID_WALL_BOOST: f32 = 0.3;

pub const GRAVITY: f32 = 0.08;
pub const VERTICAL_FRICTION: f32 = 0.98;
pub const HORIZONTAL_FRICTION: f32 = 0.91;
pub const JUMP_VELOCITY: f32 = 0.42;
pub const FLUID_JUMP_BOOST: f32 = 0.04;
pub const FALL_DAMAGE_FLOOR: f32 = 3.0;
pub const STEP_HEIGHT: f32 = 0.5;
pub const DEFAULT_BLOCK_SLIPPERINESS: f32 = 0.6;
pub const NORMAL_FRICTION_CUBED: f32 = 0.16277136;
pub const AIR_ACCELERATION: f32 = 0.02;
pub const INPUT_DECAY: f32 = 0.98;
pub const SNEAK_SPEED_MODIFIER: f32 = 0.3;

pub struct Entity {
    // For randomness
    pub rand: Random,

    // Entity type because notch split stuff into multiple packets based on type
    pub r#type: EntityType,

    // World pointer
    pub world: Weak<Mutex<WorldManager>>,

    // Identity
    pub id: EntityId, // -1 = not yet spawned
    pub is_dead: bool,
    pub ticks_existed: TickTime,
    pub dim: Dimension,

    // Riding
    pub riding_entity: Option<Weak<Mutex<dyn EntityBehavior + Send>>>,
    pub ridden_by_entity: Option<Weak<Mutex<dyn EntityBehavior + Send>>>,

    // Position
    pub pos_x: f64,
    pub pos_y: f64,
    pub pos_z: f64,

    // Velocity
    pub motion_x: f64,
    pub motion_y: f64,
    pub motion_z: f64,
    pub velocity_changed: bool,

    // Look direction
    pub rotation_yaw: f32,
    pub rotation_pitch: f32,

    // Collision
    pub collider: AABB,
    pub bucket_pos: Int3, // The bucket this entity is currently in (for spatial partitioning)
    pub below_block: BlockProperties,

    // Width/height of the collision box in blocks.
    pub width: f32,
    pub height: f32,

    // Vertical offset from posY down to the bottom of the bounding box
    pub y_offset: f32,

    // How high a block face this entity can step onto without jumping.
    pub step_height: f32,

    // Collision state
    pub on_ground: bool,
    pub collided: bool,
    pub collided_horizontally: bool,
    pub collided_vertically: bool,
    pub sneaking: bool,
    pub jumping: bool,

    // Movement / environment state
    pub has_physics: bool,
    pub in_web: bool, // Inside a cobweb
    pub in_water: bool,
    pub in_lava: bool,
    pub on_ladder: bool,

    pub fall_distance: f32,

    pub next_step_distance: i32,

    // Accumulated walk distance this tick (unused rn its mostly for the client)
    pub distance_walked_modified: f32,
    pub y_size: f32,

    pub move_forward: f32, // Forward/backward input axis
    pub move_strafe: f32,  // Left/right input axis

    // Fire
    pub fire: i32,               // Ticks remaining on fire; 0 = not on fire
    pub in_fire: bool,           // Currently touching a fire/lava block
    pub fire_resistance: i32,    // Ticks of immunity after catching fire
    pub is_immune_to_fire: bool, // Total fire immunity

    // Combat
    pub been_attacked: bool,
    pub hurt_resistant_time: i32, // Invincibility frames after being hit
    pub attacked_at_yaw: f32,     // Yaw from which the last attack came

    // Spawning
    pub prevent_entity_spawning: bool,
    pub is_first_update: bool, // True only on the very first tick

    // Air
    pub max_air: i32,
    pub air: i32,
}

impl Entity {
    pub fn new() -> Self {
        let mut entity = Self {
            rand: Random::new(),
            r#type: EntityType::None,
            world: Weak::new(),
            id: EntityId(-1),
            is_dead: false,
            ticks_existed: 0,
            dim: Dimension::Overworld,
            riding_entity: None,
            ridden_by_entity: None,
            pos_x: 0.0,
            pos_y: 0.0,
            pos_z: 0.0,
            motion_x: 0.0,
            motion_y: 0.0,
            motion_z: 0.0,
            velocity_changed: false,
            rotation_yaw: 0.0,
            rotation_pitch: 0.0,
            collider: AABB::default(),
            bucket_pos: Int3::new(0, 0, 0),
            below_block: BlockProperties::default(),
            width: 0.6,
            height: 1.8,
            y_offset: 0.0,
            step_height: 0.5,
            on_ground: false,
            collided: false,
            collided_horizontally: false,
            collided_vertically: false,
            sneaking: false,
            jumping: false,
            has_physics: true,
            in_web: false,
            in_water: false,
            in_lava: false,
            on_ladder: false,
            fall_distance: 0.0,
            next_step_distance: 0,
            distance_walked_modified: 0.0,
            y_size: 0.0,
            move_forward: 0.0,
            move_strafe: 0.0,
            fire: 0,
            in_fire: false,
            fire_resistance: 1,
            is_immune_to_fire: false,
            been_attacked: false,
            hurt_resistant_time: 0,
            attacked_at_yaw: 0.0,
            prevent_entity_spawning: false,
            is_first_update: true,
            max_air: 300,
            air: 300,
        };
        entity.rebuild_collider();
        entity
    }

    pub fn rebuild_collider(&mut self) {
        let half_width = f64::from(self.width) / 2.0;
        let bottom = self.pos_y - f64::from(self.y_offset) + f64::from(self.y_size);
        self.collider = AABB {
            min_x: self.pos_x - half_width,
            min_y: bottom,
            min_z: self.pos_z - half_width,
            max_x: self.pos_x + half_width,
            max_y: bottom + f64::from(self.height),
            max_z: self.pos_z + half_width,
        };
    }

    pub fn teleport(&mut self, newpos: Vec3, newrot: Vec2) {
        self.pos_x = newpos.x;
        self.pos_y = newpos.y;
        self.pos_z = newpos.z;
        self.rotation_yaw = newrot.x as f32;
        self.rotation_pitch = newrot.y as f32;
        self.y_size = 0.0;
        self.rebuild_collider();
    }

    pub fn attack_entity_from(&mut self, _entity: Option<&Entity>, _damage: i32) -> bool {
        self.been_attacked = true;
        self.velocity_changed = true;
        false
    }

    pub fn serialize_to_nbt(&self) -> Option<Tag> {
        // Get our string ID
        let string_id = EntityManager::get_entity_nbt_id(self.r#type)?;

        // Save position and rotation / velocity
        let pos = Tag::List {
            name: "Pos".to_string(),
            list_type: TAG_DOUBLE,
            list: vec![
                Tag::Double { name: String::new(), double_value: self.pos_x },
                Tag::Double { name: String::new(), double_value: self.pos_y },
                Tag::Double { name: String::new(), double_value: self.pos_z },
            ],
        };

        let motion = Tag::List {
            name: "Motion".to_string(),
            list_type: TAG_DOUBLE,
            list: vec![
                Tag::Double { name: String::new(), double_value: self.motion_x },
                Tag::Double { name: String::new(), double_value: self.motion_y },
                Tag::Double { name: String::new(), double_value: self.motion_z },
            ],
        };

        let rotation = Tag::List {
            name: "Rotation".to_string(),
            list_type: TAG_FLOAT,
            list: vec![
                Tag::Float { name: String::new(), float_value: self.rotation_yaw },
                Tag::Float { name: String::new(), float_value: self.rotation_pitch },
            ],
        };

        let fall_distance = Tag::Float { name: "FallDistance".to_string(), float_value: self.fall_distance };
        let fire = Tag::Short { name: "Fire".to_string(), short_value: self.fire as i16 };
        let air = Tag::Short { name: "Air".to_string(), short_value: self.air as i16 };
        let on_ground = Tag::Byte { name: "OnGround".to_string(), byte_value: self.on_ground as i8 };

        // Get our string ID
        let id = Tag::String { name: "id".to_string(), string_value: string_id.to_string() };

        // Link together our compound
        let mut compound = HashMap::new();
        compound.insert("Pos".to_string(), pos);
        compound.insert("Motion".to_string(), motion);
        compound.insert("Rotation".to_string(), rotation);
        compound.insert("FallDistance".to_string(), fall_distance);
        compound.insert("Fire".to_string(), fire);
        compound.insert("Air".to_string(), air);
        compound.insert("OnGround".to_string(), on_ground);
        compound.insert("id".to_string(), id);

        Some(Tag::Compound { name: String::new(), compound })
    }

    pub fn load_from_nbt(&mut self, nbt: &mut Tag) {
        let motion = nbt.get("Motion").get_list();
        let pos = nbt.get("Pos").get_list();
        let rotation = nbt.get("Rotation").get_list();

        self.motion_x = motion[0].get_double();
        self.motion_y = motion[1].get_double();
        self.motion_z = motion[2].get_double();

        self.pos_x = pos[0].get_double();
        self.pos_y = pos[1].get_double();
        self.pos_z = pos[2].get_double();

        self.rotation_yaw = rotation[0].get_float();
        self.rotation_pitch = rotation[1].get_float();

        self.air = nbt.get("Air").get_short() as i32;
        self.on_ground = nbt.get("OnGround").get_byte() != 0;
        self.fall_distance = nbt.get("FallDistance").get_float();
        self.fire = nbt.get("Fire").get_short() as i32;

        self.rebuild_collider();
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self::new()
    }
}

pub trait EntityBehavior {
    fn base(&self) -> &Entity;
    fn base_mut(&mut self) -> &mut Entity;

    fn as_any(&self) -> &dyn std::any::Any;

    // Encode Entity info into relevant Metadata
    fn encode_metadata(&mut self, _metadata: &[DataEntry]) {}

    // Apply Metadata to Entity
    fn decode_metadata(&mut self, _metadata: &[DataEntry]) {}

    fn tick(&mut self) {
        entity_tick(self);
    }

    fn attack_entity_from(&mut self, entity: Option<&Entity>, damage: i32) -> bool {
        self.base_mut().attack_entity_from(entity, damage)
    }

    fn get_fluid_collider(&self) -> AABB {
        // Returns the collider we use to compare if we are in a fluid
        self.base().collider.expand(0.0, -0.4, 0.0)
    }

    fn get_lava_collider(&self) -> AABB {
        // Returns the collider we use to detect if we are in lava
        self.base().collider.expand(-0.1, -0.4, -0.1)
    }

    fn push_out_of_blocks(&mut self, pos: Vec3) -> bool {
        let bx = MathHelper::floor_double(pos.x);
        let by = MathHelper::floor_double(pos.y);
        let bz = MathHelper::floor_double(pos.z);
        let frac_x = pos.x - f64::from(bx);
        let frac_y = pos.y - f64::from(by);
        let frac_z = pos.z - f64::from(bz);

        let world = self.base().world.upgrade().expect("world dropped");

        if world.lock().unwrap().is_block_normal_cube(Int3::new(bx, by, bz)) {
            let open_neg_x = !world.lock().unwrap().is_block_normal_cube(Int3::new(bx - 1, by, bz));
            let open_pos_x = !world.lock().unwrap().is_block_normal_cube(Int3::new(bx + 1, by, bz));
            let open_neg_y = !world.lock().unwrap().is_block_normal_cube(Int3::new(bx, by - 1, bz));
            let open_pos_y = !world.lock().unwrap().is_block_normal_cube(Int3::new(bx, by + 1, bz));
            let open_neg_z = !world.lock().unwrap().is_block_normal_cube(Int3::new(bx, by, bz - 1));
            let open_pos_z = !world.lock().unwrap().is_block_normal_cube(Int3::new(bx, by, bz + 1));

            let mut direction: i8 = -1;
            let mut closest = 9999.0;

            if open_neg_x && frac_x < closest {
                closest = frac_x;
                direction = 0;
            }
            if open_pos_x && 1.0 - frac_x < closest {
                closest = 1.0 - frac_x;
                direction = 1;
            }
            if open_neg_y && frac_y < closest {
                closest = frac_y;
                direction = 2;
            }
            if open_pos_y && 1.0 - frac_y < closest {
                closest = 1.0 - frac_y;
                direction = 3;
            }
            if open_neg_z && frac_z < closest {
                closest = frac_z;
                direction = 4;
            }
            if open_pos_z && 1.0 - frac_z < closest {
                closest = 1.0 - frac_z;
                direction = 5;
            }

            let base = self.base_mut();
            let push_speed: f32 = base.rand.next_float() * 0.2 + 0.1;
            if direction == 0 {
                base.motion_x = f64::from(-push_speed);
            }
            if direction == 1 {
                base.motion_x = f64::from(push_speed);
            }
            if direction == 2 {
                base.motion_y = f64::from(-push_speed);
            }
            if direction == 3 {
                base.motion_y = f64::from(push_speed);
            }
            if direction == 4 {
                base.motion_z = f64::from(-push_speed);
            }
            if direction == 5 {
                base.motion_z = f64::from(push_speed);
            }
        }

        false
    }

    fn on_collide_with_player(&mut self, _entity: &mut PlayerEntity) {}

    fn apply_knockback(&mut self, direction: Vec3) {
        let base = self.base_mut();
        base.motion_x *= f64::from(KNOCKBACK_VELOCITY_DAMPENING);
        base.motion_y *= f64::from(KNOCKBACK_VELOCITY_DAMPENING);
        base.motion_z *= f64::from(KNOCKBACK_VELOCITY_DAMPENING);
        base.motion_x -= direction.x * f64::from(HORIZONTAL_KNOCKBACK);
        base.motion_z -= direction.z * f64::from(HORIZONTAL_KNOCKBACK);
        let clamped = (base.motion_y + f64::from(VERTICAL_KNOCKBACK)) as f32;
        base.motion_y = f64::from(clamped.min(VERTICAL_KNOCKBACK));
        base.velocity_changed = true;
    }

    fn apply_input(&mut self, strafe: f32, forward: f32, acceleration: f32) {
        let mut strafe = strafe;
        let mut forward = forward;
        let mut length = (strafe * strafe + forward * forward).sqrt();

        if length < 0.01 {
            return;
        }

        if length < 1.0 {
            length = 1.0;
        }

        strafe /= length;
        forward /= length;

        let base = self.base_mut();
        base.motion_x += f64::from((strafe * base.rotation_yaw.cos() - forward * base.rotation_yaw.sin()) * acceleration);
        base.motion_z += f64::from((forward * base.rotation_yaw.cos() + strafe * base.rotation_yaw.sin()) * acceleration);
    }

    fn r#move(&mut self, movement: Vec3) {
        let mut movement = movement;
        self.base_mut().y_size *= 0.4;

        if self.base().in_web {
            self.base_mut().in_web = false;
            movement.x *= f64::from(COBWEB_HORIZONTAL_DRAG);
            movement.y *= f64::from(COBWEB_VERTICAL_DRAG);
            movement.z *= f64::from(COBWEB_HORIZONTAL_DRAG);
            let base = self.base_mut();
            base.motion_x = 0.0;
            base.motion_y = 0.0;
            base.motion_z = 0.0;
        }

        let mut original = movement;
        let original_collider = self.base().collider;
        let clamp_sneak = self.base().on_ground && self.base().sneaking;

        let world = self.base().world.upgrade().expect("world dropped");

        if clamp_sneak {
            const STEP: f64 = 0.05;
            let collider = self.base().collider;
            let ground_below = |dx: f64, dz: f64| -> bool {
                !world.lock().unwrap().get_colliding_bounding_boxes(collider.offset(dx, -1.0, dz)).is_empty()
            };

            // Clamp on the X and Z axes to avoid falling off edges while sneaking
            while movement.x != 0.0 && !ground_below(movement.x, 0.0) {
                if movement.x < STEP && movement.x >= -STEP {
                    movement.x = 0.0;
                } else if movement.x > 0.0 {
                    movement.x -= STEP;
                } else {
                    movement.x += STEP;
                }
            }
            while movement.z != 0.0 && !ground_below(0.0, movement.z) {
                if movement.z < STEP && movement.z >= -STEP {
                    movement.z = 0.0;
                } else if movement.z > 0.0 {
                    movement.z -= STEP;
                } else {
                    movement.z += STEP;
                }
            }

            // Update our og values so step up logic uses the correct position
            original.x = movement.x;
            original.z = movement.z;
        }

        let swept_collider = {
            let collider = self.base().collider;
            world.lock().unwrap().get_colliding_bounding_boxes(collider.add_coord(movement.x, movement.y, movement.z))
        };

        // Resolve Y first
        for col in &swept_collider {
            movement.y = col.calculate_y_offset(&self.base().collider, movement.y);
        }
        self.base_mut().collider = self.base().collider.offset(0.0, movement.y, 0.0);

        // Check if we are on ground or landed this tick
        let can_step_up = self.base().on_ground || (original.y != movement.y && original.y < 0.0);

        // Resolve X
        for col in &swept_collider {
            movement.x = col.calculate_x_offset(&self.base().collider, movement.x);
        }
        self.base_mut().collider = self.base().collider.offset(movement.x, 0.0, 0.0);

        // Resolve Z
        for col in &swept_collider {
            movement.z = col.calculate_z_offset(&self.base().collider, movement.z);
        }
        self.base_mut().collider = self.base().collider.offset(0.0, 0.0, movement.z);

        self.base_mut().collided_horizontally = original.x != movement.x || original.z != movement.z;

        if self.base().step_height > 0.0
            && can_step_up
            && (clamp_sneak || self.base().y_size < 0.05)
            && self.base().collided_horizontally
        {
            let step_up_movement = movement;
            movement = Vec3::new(original.x, f64::from(self.base().step_height), original.z);

            let resolved_collider = self.base().collider;
            self.base_mut().collider = original_collider;

            let step_up_swept_collider = {
                let collider = self.base().collider;
                world.lock().unwrap().get_colliding_bounding_boxes(collider.add_coord(movement.x, movement.y, movement.z))
            };

            // Resolve Y first
            for col in &step_up_swept_collider {
                movement.y = col.calculate_y_offset(&self.base().collider, movement.y);
            }
            self.base_mut().collider = self.base().collider.offset(0.0, movement.y, 0.0);

            // Resolve X
            for col in &step_up_swept_collider {
                movement.x = col.calculate_x_offset(&self.base().collider, movement.x);
            }
            self.base_mut().collider = self.base().collider.offset(movement.x, 0.0, 0.0);

            // Resolve Z
            for col in &step_up_swept_collider {
                movement.z = col.calculate_z_offset(&self.base().collider, movement.z);
            }
            self.base_mut().collider = self.base().collider.offset(0.0, 0.0, movement.z);

            // Snap down
            let mut down_y = -f64::from(self.base().step_height);
            for col in &step_up_swept_collider {
                down_y = col.calculate_y_offset(&self.base().collider, down_y);
            }
            self.base_mut().collider = self.base().collider.offset(0.0, down_y, 0.0);

            // Keep whichever collision path moved further horizontally
            if step_up_movement.x * step_up_movement.x + step_up_movement.z * step_up_movement.z
                > movement.x * movement.x + movement.z * movement.z
            {
                movement = step_up_movement;
                self.base_mut().collider = resolved_collider;
            } else {
                movement.y += (resolved_collider.min_y - original_collider.min_y) - f64::from(self.base().step_height);
                let frac = resolved_collider.min_y - self.base().collider.min_y;
                if frac > 0.0 {
                    self.base_mut().y_size += (frac + 0.01) as f32;
                }
            }
        }

        // Derive our current position from our collider
        let collider = self.base().collider;
        let base = self.base_mut();
        base.pos_x = (collider.min_x + collider.max_x) / 2.0;
        base.pos_y = collider.min_y + f64::from(base.y_offset) - f64::from(base.y_size);
        base.pos_z = (collider.min_z + collider.max_z) / 2.0;

        base.collided_horizontally = original.x != movement.x || original.z != movement.z;
        base.collided_vertically = original.y != movement.y;
        base.on_ground = original.y != movement.y && original.y < 0.0;
        base.collided = base.collided_horizontally || base.collided_vertically;

        if original.x != movement.x {
            base.motion_x = 0.0;
        }
        if original.y != movement.y {
            base.motion_y = 0.0;
        }
        if original.z != movement.z {
            base.motion_z = 0.0;
        }

        self.update_fall_state(movement.y as f32);

        // Scan each block this entity overlaps so we can trigger collided with code
        let collider = self.base().collider;
        let min_x = MathHelper::floor_double(collider.min_x + 0.001);
        let min_y = MathHelper::floor_double(collider.min_y + 0.001);
        let min_z = MathHelper::floor_double(collider.min_z + 0.001);
        let max_x = MathHelper::floor_double(collider.max_x - 0.001);
        let max_y = MathHelper::floor_double(collider.max_y - 0.001);
        let max_z = MathHelper::floor_double(collider.max_z - 0.001);

        let valid = world.lock().unwrap().aabb_in_valid_chunks(AABB {
            min_x: f64::from(min_x),
            min_y: f64::from(min_y),
            min_z: f64::from(min_z),
            max_x: f64::from(max_x),
            max_y: f64::from(max_y),
            max_z: f64::from(max_z),
        });

        if valid {
            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    for z in min_z..=max_z {
                        let block_id = world.lock().unwrap().get_block_id(Int3::new(x, y, z));
                        let index = if block_id.0 < 0 { 0usize } else { block_id.0 as u8 as usize };
                        let function = block_properties::block_behaviors()[index].on_entity_collided_with_block;
                        if block_id.0 > 0 {
                            if let Some(function) = function {
                                let mut world_guard = world.lock().unwrap();
                                function(&mut *world_guard, Int3::new(x, y, z), self.base_mut());
                            }
                        }
                    }
                }
            }
        }
    }

    fn deal_damage(&mut self, _amount: i32) {}

    fn update_fall_state(&mut self, moved_y: f32) {
        if self.base().on_ground {
            if self.base().fall_distance > FALL_DAMAGE_FLOOR {
                let amount = (self.base().fall_distance - FALL_DAMAGE_FLOOR).ceil() as i32;
                self.deal_damage(amount);
            }
            self.base_mut().fall_distance = 0.0;
        } else if moved_y < 0.0 {
            self.base_mut().fall_distance -= moved_y;
        }
    }

    fn serialize_to_nbt(&mut self) -> Option<Tag> {
        self.base().serialize_to_nbt()
    }

    fn load_from_nbt(&mut self, nbt: &mut Tag) {
        self.base_mut().load_from_nbt(nbt)
    }
}

impl EntityBehavior for Entity {
    fn base(&self) -> &Entity {
        self
    }

    fn base_mut(&mut self) -> &mut Entity {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn entity_tick<T: EntityBehavior + ?Sized>(entity: &mut T) {
    entity.base_mut().ticks_existed += 1;

    let riding_dead = entity
        .base()
        .riding_entity
        .as_ref()
        .map(|weak| weak.upgrade().map(|arc| arc.lock().unwrap().base().is_dead).unwrap_or(true));
    if riding_dead == Some(true) {
        entity.base_mut().riding_entity = None;
    }

    // Returns if we are in water and applies a push to our entity
    let fluid_collider = entity.get_fluid_collider();
    let world = entity.base().world.upgrade().expect("world dropped");
    let in_water = world.lock().unwrap().handle_fluid_acceleration(fluid_collider, Material::water(), entity.base_mut());

    if in_water {
        let base = entity.base_mut();
        base.fall_distance = 0.0;
        base.in_water = true;
        base.fire = 0;
    } else {
        entity.base_mut().in_water = false;
    }

    // If we are in fire decrement the fire
    if entity.base().fire > 0 {
        if entity.base().is_immune_to_fire {
            let base = entity.base_mut();
            base.fire -= 4;
            base.fire = base.fire.max(0);
        } else {
            if entity.base().fire % 20 == 0 {
                entity.attack_entity_from(None, 1);
            }
            entity.base_mut().fire -= 1;
        }
    }

    // Returns if we are in lava
    let lava_collider = entity.get_lava_collider();
    let is_in_lava = world.lock().unwrap().is_material_in_aabb(lava_collider, Material::lava());
    if is_in_lava {
        if !entity.base().is_immune_to_fire {
            entity.attack_entity_from(None, 4);
            entity.base_mut().fire = 600;
        }
    }

    // Kill our entity if its below the world
    if entity.base().pos_y < -64.0 {
        entity.base_mut().is_dead = true;
    }

    entity.base_mut().is_first_update = false;
}
