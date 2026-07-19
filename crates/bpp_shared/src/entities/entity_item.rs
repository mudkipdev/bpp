/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::collections::HashMap;

use crate::base_types::{ItemId, TickTime};
use crate::blocks::block_properties;
use crate::blocks::materials::Material;
use crate::entities::entity::{Entity, EntityBehavior, entity_tick};
use crate::entities::entity_player::{PlayerEntity, PlayerEntityBehavior};
use crate::enums::entities::EntityType;
use crate::helpers::aabb::AABB;
use crate::helpers::java::java_math::MathHelper;
use crate::inventory::item_stack::ItemStack;
use crate::nbt::nbt::Tag;
use crate::numeric_structs::{Int3, Vec2, Vec3};

pub struct ItemEntity {
    pub base: Entity,
    pub item_stack: ItemStack,
    pub health: i8,
    pub pickup_cooldown: i8,
}

impl ItemEntity {
    pub fn new(position: Vec3) -> Self {
        let mut base = Entity::new();
        base.r#type = EntityType::Item;
        base.width = 0.25;
        base.height = 0.25;
        base.y_offset = 0.125; // height / 2

        // Set the initial position of the item entity
        base.teleport(position, Vec2::new(0.0, 0.0));

        // This stuff is mostly randomized
        base.rotation_yaw = (base.rand.next_double() * 360.0) as f32;
        base.motion_x = base.rand.next_double() * 0.2 - 0.1;
        base.motion_y = 0.2;
        base.motion_z = base.rand.next_double() * 0.2 - 0.1;

        ItemEntity { base, item_stack: ItemStack::default(), health: 5, pickup_cooldown: 10 }
    }
}

impl EntityBehavior for ItemEntity {
    fn base(&self) -> &Entity {
        &self.base
    }

    fn base_mut(&mut self) -> &mut Entity {
        &mut self.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_fluid_collider(&self) -> AABB {
        // Returns the collider we use to compare if we are in a fluid
        self.base.collider
    }

    fn get_lava_collider(&self) -> AABB {
        // Returns the collider we use to detect if we are in lava
        self.base.collider
    }

    fn on_collide_with_player(&mut self, entity: &mut PlayerEntity) {
        if self.pickup_cooldown != 0 {
            return;
        }
        entity.pickup_item(&mut self.item_stack, self.base.id);
        if self.item_stack.count <= 0 {
            self.base.is_dead = true;
        }
    }

    fn tick(&mut self) {
        // Item entities have differing physics
        entity_tick(self);
        self.pickup_cooldown -= 1;
        self.pickup_cooldown = self.pickup_cooldown.max(0);

        self.base.motion_y -= 0.04;

        let world = self.base.world.upgrade().expect("world dropped");
        let material = {
            let world_guard = world.lock().unwrap();
            world_guard.get_material(Int3::new(
                MathHelper::floor_double(self.base.pos_x),
                MathHelper::floor_double(self.base.pos_y),
                MathHelper::floor_double(self.base.pos_z),
            ))
        };
        if material == Material::lava() {
            self.base.motion_y = 0.2;
            self.base.motion_x = f64::from((self.base.rand.next_float() - self.base.rand.next_float()) * 0.2);
            self.base.motion_z = f64::from((self.base.rand.next_float() - self.base.rand.next_float()) * 0.2);
        }

        let pos = Vec3::new(self.base.pos_x, (self.base.collider.min_y + self.base.collider.max_y) / 2.0, self.base.pos_z);
        self.push_out_of_blocks(pos);
        let motion = Vec3::new(self.base.motion_x, self.base.motion_y, self.base.motion_z);
        self.r#move(motion);

        let mut horizontal_drag: f32 = 0.98;
        if self.base.on_ground {
            horizontal_drag = 0.58800006;

            // Look up the block below us
            let bx = MathHelper::floor_double(self.base.pos_x);
            let by = MathHelper::floor_double(self.base.collider.min_y) - 1;
            let bz = MathHelper::floor_double(self.base.pos_z);
            let block_id = {
                let world_guard = world.lock().unwrap();
                world_guard.get_block_id(Int3::new(bx, by, bz))
            };
            self.base.below_block = block_properties::block_properties()[block_id.0 as u8 as usize];

            if block_id.0 > 0 {
                horizontal_drag = self.base.below_block.slipperiness * 0.98;
            }
        }

        self.base.motion_x *= f64::from(horizontal_drag);
        self.base.motion_y *= 0.9800000190734863;
        self.base.motion_z *= f64::from(horizontal_drag);

        // Bounce when we land
        if self.base.on_ground {
            self.base.motion_y *= -0.5;
        }

        if self.base.ticks_existed >= 6000 {
            self.base.is_dead = true;
        }
    }

    fn serialize_to_nbt(&mut self) -> Option<Tag> {
        let mut tag = self.base.serialize_to_nbt()?;

        // Our additions
        let health = Tag::Short { name: "Health".to_string(), short_value: self.health as i16 };
        let age = Tag::Short { name: "Age".to_string(), short_value: self.base.ticks_existed as i16 };

        // Construct the item nbt
        let id = Tag::Short { name: "id".to_string(), short_value: self.item_stack.id.value() };
        let count = Tag::Byte { name: "Count".to_string(), byte_value: self.item_stack.count };
        let damage = Tag::Short { name: "Damage".to_string(), short_value: self.item_stack.data };
        let mut item_compound = HashMap::new();
        item_compound.insert("id".to_string(), id);
        item_compound.insert("Count".to_string(), count);
        item_compound.insert("Damage".to_string(), damage);
        let item = Tag::Compound { name: "Item".to_string(), compound: item_compound };

        // Add our additions to the base tag
        if let Tag::Compound { compound, .. } = &mut tag {
            compound.insert("Health".to_string(), health);
            compound.insert("Age".to_string(), age);
            compound.insert("Item".to_string(), item);
        }

        Some(tag)
    }

    fn load_from_nbt(&mut self, nbt: &mut Tag) {
        self.base.load_from_nbt(nbt);

        // Load item specific stuff
        self.health = nbt.get("Health").get_short() as i8;
        self.base.ticks_existed = nbt.get("Age").get_short() as TickTime;

        // Load our item
        let item_compound = nbt.get("Item").get_compound();
        self.item_stack = ItemStack {
            id: ItemId(item_compound.get("id").expect("missing id").get_short()),
            count: item_compound.get("Count").expect("missing Count").get_byte(),
            data: item_compound.get("Damage").expect("missing Damage").get_short(),
        };
    }

    fn attack_entity_from(&mut self, entity: Option<&Entity>, damage: i32) -> bool {
        self.base.attack_entity_from(entity, damage);
        self.health -= damage as i8;
        if self.health <= 0 {
            self.base.is_dead = true;
        }
        false
    }
}
