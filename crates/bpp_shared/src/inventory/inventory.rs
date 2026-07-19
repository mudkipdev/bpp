/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/
use crate::base_types::{NbtSlotId, NetworkSlotId};
use crate::enums::items;
use crate::inventory::item_stack::ItemStack;
use crate::items::item_properties;

// Inventory
#[derive(Clone)]
pub struct Inventory {
    pub name: String,
    pub slots: Vec<ItemStack>,
    pub is_modified: bool,
}

impl Inventory {
    pub fn new(size: usize) -> Self {
        Inventory {
            name: "Inventory".to_string(),
            slots: vec![ItemStack::default(); size],
            is_modified: false,
        }
    }
}

pub trait InventoryBehavior {
    fn base(&self) -> &Inventory;
    fn base_mut(&mut self) -> &mut Inventory;

    fn get_size_inventory(&self) -> i32 {
        self.base().slots.len() as i32
    }

    fn get_network_slot_id(&self, slot: NbtSlotId) -> NetworkSlotId {
        if slot.value() < 0 || slot.value() as i32 >= self.base().slots.len() as i32 {
            return NetworkSlotId(-1);
        }
        NetworkSlotId(slot.value() as i16)
    }

    fn get_stack_in_slot(&mut self, slot: i32) -> Option<&mut ItemStack> {
        if slot < 0 || slot >= self.base().slots.len() as i32 {
            return None;
        }
        let idx = slot as usize;
        if self.base().slots[idx].id == items::INVALID {
            return None;
        }
        Some(&mut self.base_mut().slots[idx])
    }

    fn decrease_stack_size(&mut self, slot: i32, count: i32) -> ItemStack {
        if slot < 0
            || slot >= self.base().slots.len() as i32
            || self.base().slots[slot as usize].id == items::INVALID
        {
            return ItemStack::default();
        }
        let idx = slot as usize;
        let stack = self.base().slots[idx];
        let taken = if stack.count as i32 <= count {
            self.base_mut().slots[idx] = ItemStack::default();
            stack
        } else {
            let taken = ItemStack {
                id: stack.id,
                count: count as i8,
                data: stack.data,
            };
            self.base_mut().slots[idx].count = (stack.count as i32 - count) as i8;
            taken
        };
        self.on_inventory_changed();
        taken
    }

    fn set_inventory_slot_contents(&mut self, slot: i32, stack: Option<&ItemStack>) {
        if slot < 0 || slot >= self.base().slots.len() as i32 {
            return;
        }
        self.base_mut().slots[slot as usize] = stack.copied().unwrap_or_default();
        self.on_inventory_changed();
    }

    fn get_inventory_name(&self) -> &str {
        &self.base().name
    }

    fn on_inventory_changed(&mut self) {
        self.base_mut().is_modified = true;
    }

    fn clear_slot(&mut self, slot: i32) {
        if slot < 0 || slot >= self.base().slots.len() as i32 {
            return;
        }
        self.base_mut().slots[slot as usize] = ItemStack::default();
        self.on_inventory_changed();
    }

    // Take in the original item stack, try and merge it with our inventory. Returns if it was successful
    // Start slot and end slot are inclusive
    fn merge_item_stack_in_inventory(
        &mut self,
        stack: &mut ItemStack,
        reverse: bool,
        start_slot: i32,
        end_slot: i32,
    ) -> bool {
        let start = start_slot;
        let end = if end_slot == -1 {
            self.get_size_inventory() - 1
        } else {
            end_slot
        };

        // Try and merge into an already existing stack of the same type if this item is stackable
        if item_properties::is_stackable(stack.id) {
            let order: Vec<i32> = if reverse {
                (start..=end).rev().collect()
            } else {
                (start..=end).collect()
            };
            for i in order {
                let slot = match self.get_stack_in_slot(i) {
                    Some(slot) => slot,
                    None => continue,
                };
                if slot.id == stack.id && slot.data == stack.data {
                    let max_stack = item_properties::get_max_stack(slot.id);
                    // Don't try and merge into an already maxed out stack
                    if slot.count as i32 >= max_stack {
                        continue;
                    }

                    // Add the stacks together and do some checks to make sure we don't overflow
                    let space = max_stack - slot.count as i32;
                    let to_move = space.min(stack.count as i32);

                    slot.count += to_move as i8;
                    stack.count -= to_move as i8;

                    self.on_inventory_changed();
                    if stack.count == 0 {
                        return true;
                    }
                }
            }
        }

        // We couldn't merge into existing items so just try and find an empty slot
        let order: Vec<i32> = if reverse {
            (start..=end).rev().collect()
        } else {
            (start..=end).collect()
        };
        for i in order {
            if self.base().slots[i as usize].id == items::INVALID {
                self.base_mut().slots[i as usize] = ItemStack {
                    id: stack.id,
                    count: stack.count,
                    data: stack.data,
                };
                stack.id = items::INVALID;
                stack.data = 0;
                stack.count = 0;
                self.on_inventory_changed();
                return true;
            }
        }
        false // Give up
    }
}

impl InventoryBehavior for Inventory {
    fn base(&self) -> &Inventory {
        self
    }
    fn base_mut(&mut self) -> &mut Inventory {
        self
    }
}
