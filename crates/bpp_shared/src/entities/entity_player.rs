/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use crate::base_types::EntityId;
use crate::entities::entity::{Entity, EntityBehavior};
use crate::enums::entities::EntityType;
use crate::inventory::item_stack::ItemStack;

pub struct PlayerEntity {
    pub base: Entity,
}

impl PlayerEntity {
    pub fn new() -> Self {
        let mut base = Entity::new();
        base.r#type = EntityType::Player;
        base.has_physics = false;
        base.width = 0.6;
        base.height = 1.8;
        base.step_height = 0.5;

        PlayerEntity { base }
    }
}

impl Default for PlayerEntity {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityBehavior for PlayerEntity {
    fn base(&self) -> &Entity {
        &self.base
    }

    fn base_mut(&mut self) -> &mut Entity {
        &mut self.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub trait PlayerEntityBehavior: EntityBehavior {
    fn pickup_item(&mut self, _stack: &mut ItemStack, _entity_id: EntityId) -> bool {
        true
    }

    fn drop_item(&mut self, _stack: ItemStack) -> bool {
        true
    }
}

impl PlayerEntityBehavior for PlayerEntity {}
