/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::sync::{Arc, Mutex, Weak};

use bpp_shared::base_types::EntityId;
use bpp_shared::constants::PLAYER_EYE_HEIGHT;
use bpp_shared::entities::entity::{Entity, EntityBehavior};
use bpp_shared::entities::entity_item::ItemEntity;
use bpp_shared::entities::entity_player::{PlayerEntity, PlayerEntityBehavior};
use bpp_shared::enums::items;
use bpp_shared::helpers::java::java_math::JavaMath;
use bpp_shared::inventory::item_stack::ItemStack;
use bpp_shared::networking::packets::{CollectItem, PacketBehavior};
use bpp_shared::numeric_structs::Vec3;

use crate::player_conn::player_session::PlayerSession;

pub struct EntityMPPlayer {
    pub base: PlayerEntity,
    pub session: Option<Weak<Mutex<PlayerSession>>>,
}

impl EntityMPPlayer {
    pub fn new() -> Self {
        EntityMPPlayer {
            base: PlayerEntity::new(),
            session: None,
        }
    }
}

impl Default for EntityMPPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityBehavior for EntityMPPlayer {
    fn base(&self) -> &Entity {
        &self.base.base
    }

    fn base_mut(&mut self) -> &mut Entity {
        &mut self.base.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    // We ignore physics for the player entity and just grab what the client tells us
    fn tick(&mut self) {
        let session = match self.session.as_ref().and_then(Weak::upgrade) {
            Some(session) => session,
            None => return,
        };
        let (claimed, rotation) = {
            let guard = session.lock().unwrap();
            (guard.position.pos, guard.rotation)
        };

        self.base.base.pos_x = claimed.x;
        self.base.base.pos_y = claimed.y;
        self.base.base.pos_z = claimed.z;
        self.base.base.rotation_yaw = rotation.x;
        self.base.base.rotation_pitch = rotation.y;

        self.base.base.rebuild_collider();

        // Tell entities we collided with them
        if let Some(world) = self.base.base.world.upgrade() {
            let collider_copy = self.base.base.collider.expand(1.0, 0.0, 1.0);
            let entities_colliding_with = world
                .lock()
                .unwrap()
                .entity_manager
                .get_entities_within_aabb_excluding(collider_copy, self.base.base.id);
            for entity in entities_colliding_with {
                let mut guard = entity.lock().unwrap();
                let collided =
                    guard.base().collider.intersects(&collider_copy) && !guard.base().is_dead;
                if collided {
                    guard.on_collide_with_player(&mut self.base);
                }
            }
        }
    }
}

impl PlayerEntityBehavior for EntityMPPlayer {
    fn pickup_item(&mut self, stack: &mut ItemStack, entity_id: EntityId) -> bool {
        let session = self
            .session
            .as_ref()
            .and_then(Weak::upgrade)
            .expect("no session");
        let mut guard = session.lock().unwrap();

        if guard.inventory.pickup_item(stack) {
            let mut pkt = CollectItem::new();
            pkt.collector_entity_id = self.base.base.id;
            pkt.item_entity_id = entity_id;
            let entity_tracker = guard
                .entity_tracker
                .upgrade()
                .expect("entity tracker dropped");
            entity_tracker
                .lock()
                .unwrap()
                .send_packet_to_viewers(&pkt, self.base.base.id);
            pkt.serialize(&mut guard.stream);
            return true;
        }

        false
    }

    // This works over a copy of your item, it doesn't remove or decrement it !!!
    fn drop_item(&mut self, stack: ItemStack) -> bool {
        if stack.id == items::INVALID || stack.count <= 0 {
            return false;
        }

        // Create the item entity
        let position = Vec3::new(
            self.base.base.pos_x,
            self.base.base.pos_y - 0.3 + PLAYER_EYE_HEIGHT,
            self.base.base.pos_z,
        );
        let mut item_entity = ItemEntity::new(position);
        item_entity.item_stack = stack;
        item_entity.pickup_cooldown = 40; // So we don't pick it up instantly
        item_entity.base.dim = self.base.base.dim;

        // Give ourselves some random velocity based on look direction
        let mut velocity: f32 = 0.3;
        item_entity.base.motion_x = f64::from(
            -(self.base.base.rotation_yaw / 180.0 * JavaMath::PI_FLOAT).sin()
                * (self.base.base.rotation_pitch / 180.0 * JavaMath::PI_FLOAT).cos()
                * velocity,
        );
        item_entity.base.motion_z = f64::from(
            (self.base.base.rotation_yaw / 180.0 * JavaMath::PI_FLOAT).cos()
                * (self.base.base.rotation_pitch / 180.0 * JavaMath::PI_FLOAT).cos()
                * velocity,
        );
        item_entity.base.motion_y = f64::from(
            -(self.base.base.rotation_pitch / 180.0 * JavaMath::PI_FLOAT).sin() * velocity + 0.1,
        );

        // Add a little bit of randomness
        velocity = 0.02;
        let angle = self.base.base.rand.next_float() * JavaMath::PI_FLOAT * 2.0;
        velocity *= self.base.base.rand.next_float();
        item_entity.base.motion_x += f64::from(angle.cos() * velocity);
        item_entity.base.motion_y +=
            f64::from((self.base.base.rand.next_float() - self.base.base.rand.next_float()) * 0.1);
        item_entity.base.motion_z += f64::from(angle.sin() * velocity);

        // Register our item with the world
        let world = self.base.base.world.upgrade().expect("world dropped");
        world
            .lock()
            .unwrap()
            .entity_manager
            .add_entity(Arc::new(Mutex::new(item_entity)), EntityId(-1));
        true
    }
}
