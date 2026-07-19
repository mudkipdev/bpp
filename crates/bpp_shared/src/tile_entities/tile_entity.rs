/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
use std::collections::HashMap;
use std::sync::{Mutex, Weak};

use crate::enums::items;
use crate::inventory::inventories::{InventoryChest, InventoryDispenser, InventoryFurnace};
use crate::nbt::nbt::{TAG_COMPOUND, Tag};
use crate::numeric_structs::Int3;
use crate::world::chunk::Chunk;

// I hate doing inheritance but its simple to do for this
pub struct TileEntity {
    pub id: String,
    pub position: Int3, // Global coordinates
    pub can_tick: bool,
    pub chunk: Weak<Mutex<Chunk>>, // The chunk this tile entity is in; may not be best practice to have this as a raw pointer but it should be fine since the chunk will always exist while the tile entity exists
}

impl TileEntity {
    pub fn new(id: String, position: Int3) -> Self {
        TileEntity {
            id,
            position,
            can_tick: false,
            chunk: Weak::new(),
        }
    }
}

pub trait TileEntityBehavior {
    fn base(&self) -> &TileEntity;
    fn base_mut(&mut self) -> &mut TileEntity;

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn tick(&mut self) {}

    fn serialize(&self) -> Tag {
        let base = self.base();

        let id = Tag::String { name: "id".to_string(), string_value: base.id.clone() };
        let x = Tag::Int { name: "x".to_string(), int_value: base.position.x };
        let y = Tag::Int { name: "y".to_string(), int_value: base.position.y };
        let z = Tag::Int { name: "z".to_string(), int_value: base.position.z };

        let mut compound = HashMap::new();
        compound.insert("id".to_string(), id);
        compound.insert("x".to_string(), x);
        compound.insert("y".to_string(), y);
        compound.insert("z".to_string(), z);

        Tag::Compound { name: String::new(), compound }
    }
}

// Chest
pub struct TileEntityChest {
    pub base: TileEntity,
    pub inventory: InventoryChest,
}

impl TileEntityChest {
    pub fn new(position: Int3) -> Self {
        TileEntityChest {
            base: TileEntity::new("Chest".to_string(), position),
            inventory: InventoryChest::new(),
        }
    }
}

impl TileEntityBehavior for TileEntityChest {
    fn base(&self) -> &TileEntity {
        &self.base
    }

    fn base_mut(&mut self) -> &mut TileEntity {
        &mut self.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn tick(&mut self) {
        if let Some(chunk) = self.base.chunk.upgrade() {
            if self.inventory.base.is_modified {
                chunk.lock().unwrap().is_modified = true;
                self.inventory.base.is_modified = false;
            }
        }
    }

    fn serialize(&self) -> Tag {
        let base = self.base();

        let id = Tag::String { name: "id".to_string(), string_value: base.id.clone() };
        let x = Tag::Int { name: "x".to_string(), int_value: base.position.x };
        let y = Tag::Int { name: "y".to_string(), int_value: base.position.y };
        let z = Tag::Int { name: "z".to_string(), int_value: base.position.z };

        // Construct our inventory
        let mut list = Vec::new();
        let mut current_slot: i8 = 0;
        for stack in &self.inventory.base.slots {
            if stack.id != items::INVALID {
                let count = Tag::Byte { name: "Count".to_string(), byte_value: stack.count };
                let damage = Tag::Short { name: "Damage".to_string(), short_value: stack.data };
                let id_tag = Tag::Short { name: "id".to_string(), short_value: stack.id.value() };
                let slot = Tag::Byte { name: "Slot".to_string(), byte_value: current_slot };

                let mut item_compound = HashMap::new();
                item_compound.insert("Count".to_string(), count);
                item_compound.insert("Damage".to_string(), damage);
                item_compound.insert("id".to_string(), id_tag);
                item_compound.insert("Slot".to_string(), slot);

                list.push(Tag::Compound { name: String::new(), compound: item_compound });
            }
            current_slot += 1;
        }
        let items_tag = Tag::List { name: "Items".to_string(), list_type: TAG_COMPOUND, list };

        let mut compound = HashMap::new();
        compound.insert("Items".to_string(), items_tag);
        compound.insert("id".to_string(), id);
        compound.insert("x".to_string(), x);
        compound.insert("y".to_string(), y);
        compound.insert("z".to_string(), z);

        Tag::Compound { name: String::new(), compound }
    }
}

// Furnace
pub struct TileEntityFurnace {
    pub base: TileEntity,
    pub inventory: InventoryFurnace,
}

impl TileEntityFurnace {
    pub fn new(position: Int3) -> Self {
        let mut base = TileEntity::new("Furnace".to_string(), position);
        base.can_tick = true;

        TileEntityFurnace { base, inventory: InventoryFurnace::new() }
    }
}

impl TileEntityBehavior for TileEntityFurnace {
    fn base(&self) -> &TileEntity {
        &self.base
    }

    fn base_mut(&mut self) -> &mut TileEntity {
        &mut self.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn tick(&mut self) {
        if let Some(chunk) = self.base.chunk.upgrade() {
            if self.inventory.base.is_modified {
                chunk.lock().unwrap().is_modified = true;
                self.inventory.base.is_modified = false;
            }
        }
    }

    fn serialize(&self) -> Tag {
        let base = self.base();

        let id = Tag::String { name: "id".to_string(), string_value: base.id.clone() };
        let x = Tag::Int { name: "x".to_string(), int_value: base.position.x };
        let y = Tag::Int { name: "y".to_string(), int_value: base.position.y };
        let z = Tag::Int { name: "z".to_string(), int_value: base.position.z };

        // Construct our inventory
        let mut list = Vec::new();
        let mut current_slot: i8 = 0;
        for stack in &self.inventory.base.slots {
            if stack.id != items::INVALID {
                let count = Tag::Byte { name: "Count".to_string(), byte_value: stack.count };
                let damage = Tag::Short { name: "Damage".to_string(), short_value: stack.data };
                let id_tag = Tag::Short { name: "id".to_string(), short_value: stack.id.value() };
                let slot = Tag::Byte { name: "Slot".to_string(), byte_value: current_slot };

                let mut item_compound = HashMap::new();
                item_compound.insert("Count".to_string(), count);
                item_compound.insert("Damage".to_string(), damage);
                item_compound.insert("id".to_string(), id_tag);
                item_compound.insert("Slot".to_string(), slot);

                list.push(Tag::Compound { name: String::new(), compound: item_compound });
            }
            current_slot += 1;
        }
        let items_tag = Tag::List { name: "Items".to_string(), list_type: TAG_COMPOUND, list };

        let mut compound = HashMap::new();
        compound.insert("Items".to_string(), items_tag);
        compound.insert("id".to_string(), id);
        compound.insert("x".to_string(), x);
        compound.insert("y".to_string(), y);
        compound.insert("z".to_string(), z);

        Tag::Compound { name: String::new(), compound }
    }
}

// Dispenser (Trap)
pub struct TileEntityDispenser {
    pub base: TileEntity,
    pub inventory: InventoryDispenser,
}

impl TileEntityDispenser {
    pub fn new(position: Int3) -> Self {
        TileEntityDispenser {
            base: TileEntity::new("Trap".to_string(), position),
            inventory: InventoryDispenser::new(),
        }
    }
}

impl TileEntityBehavior for TileEntityDispenser {
    fn base(&self) -> &TileEntity {
        &self.base
    }

    fn base_mut(&mut self) -> &mut TileEntity {
        &mut self.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn tick(&mut self) {
        if let Some(chunk) = self.base.chunk.upgrade() {
            if self.inventory.base.is_modified {
                chunk.lock().unwrap().is_modified = true;
                self.inventory.base.is_modified = false;
            }
        }
    }

    fn serialize(&self) -> Tag {
        let base = self.base();

        let id = Tag::String { name: "id".to_string(), string_value: base.id.clone() };
        let x = Tag::Int { name: "x".to_string(), int_value: base.position.x };
        let y = Tag::Int { name: "y".to_string(), int_value: base.position.y };
        let z = Tag::Int { name: "z".to_string(), int_value: base.position.z };

        // Construct our inventory
        let mut list = Vec::new();
        let mut current_slot: i8 = 0;
        for stack in &self.inventory.base.slots {
            if stack.id != items::INVALID {
                let count = Tag::Byte { name: "Count".to_string(), byte_value: stack.count };
                let damage = Tag::Short { name: "Damage".to_string(), short_value: stack.data };
                let id_tag = Tag::Short { name: "id".to_string(), short_value: stack.id.value() };
                let slot = Tag::Byte { name: "Slot".to_string(), byte_value: current_slot };

                let mut item_compound = HashMap::new();
                item_compound.insert("Count".to_string(), count);
                item_compound.insert("Damage".to_string(), damage);
                item_compound.insert("id".to_string(), id_tag);
                item_compound.insert("Slot".to_string(), slot);

                list.push(Tag::Compound { name: String::new(), compound: item_compound });
            }
            current_slot += 1;
        }
        let items_tag = Tag::List { name: "Items".to_string(), list_type: TAG_COMPOUND, list };

        let mut compound = HashMap::new();
        compound.insert("Items".to_string(), items_tag);
        compound.insert("id".to_string(), id);
        compound.insert("x".to_string(), x);
        compound.insert("y".to_string(), y);
        compound.insert("z".to_string(), z);

        Tag::Compound { name: String::new(), compound }
    }
}

// Sign
pub struct TileEntitySign {
    pub base: TileEntity,
    pub text1: String,
    pub text2: String,
    pub text3: String,
    pub text4: String,
}

impl TileEntitySign {
    pub fn new(position: Int3) -> Self {
        TileEntitySign {
            base: TileEntity::new("Sign".to_string(), position),
            text1: String::new(),
            text2: String::new(),
            text3: String::new(),
            text4: String::new(),
        }
    }
}

impl TileEntityBehavior for TileEntitySign {
    fn base(&self) -> &TileEntity {
        &self.base
    }

    fn base_mut(&mut self) -> &mut TileEntity {
        &mut self.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn serialize(&self) -> Tag {
        let base = self.base();

        let id = Tag::String { name: "id".to_string(), string_value: base.id.clone() };
        let x = Tag::Int { name: "x".to_string(), int_value: base.position.x };
        let y = Tag::Int { name: "y".to_string(), int_value: base.position.y };
        let z = Tag::Int { name: "z".to_string(), int_value: base.position.z };
        let text1 = Tag::String { name: "Text1".to_string(), string_value: self.text1.clone() };
        let text2 = Tag::String { name: "Text2".to_string(), string_value: self.text2.clone() };
        let text3 = Tag::String { name: "Text3".to_string(), string_value: self.text3.clone() };
        let text4 = Tag::String { name: "Text4".to_string(), string_value: self.text4.clone() };

        let mut compound = HashMap::new();
        compound.insert("Text1".to_string(), text1);
        compound.insert("Text2".to_string(), text2);
        compound.insert("Text3".to_string(), text3);
        compound.insert("Text4".to_string(), text4);
        compound.insert("id".to_string(), id);
        compound.insert("x".to_string(), x);
        compound.insert("y".to_string(), y);
        compound.insert("z".to_string(), z);

        Tag::Compound { name: String::new(), compound }
    }
}

// MobSpawner
pub struct TileEntityMobSpawner {
    pub base: TileEntity,
    pub entity_id: String,
    pub delay: i16,
}

impl TileEntityMobSpawner {
    pub fn new(position: Int3) -> Self {
        let mut base = TileEntity::new("MobSpawner".to_string(), position);
        base.can_tick = true;

        TileEntityMobSpawner { base, entity_id: String::new(), delay: 0 }
    }
}

impl TileEntityBehavior for TileEntityMobSpawner {
    fn base(&self) -> &TileEntity {
        &self.base
    }

    fn base_mut(&mut self) -> &mut TileEntity {
        &mut self.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn serialize(&self) -> Tag {
        let base = self.base();

        let id = Tag::String { name: "id".to_string(), string_value: base.id.clone() };
        let x = Tag::Int { name: "x".to_string(), int_value: base.position.x };
        let y = Tag::Int { name: "y".to_string(), int_value: base.position.y };
        let z = Tag::Int { name: "z".to_string(), int_value: base.position.z };
        let entity_id = Tag::String { name: "EntityId".to_string(), string_value: self.entity_id.clone() };
        let delay = Tag::Short { name: "Delay".to_string(), short_value: self.delay };

        let mut compound = HashMap::new();
        compound.insert("EntityId".to_string(), entity_id);
        compound.insert("Delay".to_string(), delay);
        compound.insert("id".to_string(), id);
        compound.insert("x".to_string(), x);
        compound.insert("y".to_string(), y);
        compound.insert("z".to_string(), z);

        Tag::Compound { name: String::new(), compound }
    }
}
