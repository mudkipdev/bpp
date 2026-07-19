/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 * Copyright (c) 2026, jwaxy <jwaxy.is-a.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/

use crate::base_types::{NbtSlotId, NetworkSlotId};
use crate::enums::items;
use crate::helpers::java::java_random::Random;
use crate::inventory::inventory::{Inventory, InventoryBehavior};
use crate::inventory::item_stack::ItemStack;
use crate::items::item_properties;
use crate::logger::logger::global_logger;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvMap {
    Armor,
    Inventory,
    Hotbar,
    CraftingArea,
    CraftingResult,
    Invalid,
}

// Network format (The rest of the inventories are self explanatory this is the only one that is semi-convoluted):
// Slots 5 -> 8 are for armor
// Slots 36 -> 44 are the hotbar
// Slots 9 -> 35 are the main inventory
// Slots 1 -> 4 are the crafting grid
// Slot 0 is the crafting result
pub struct InventoryPlayer {
    pub base: Inventory,
    pub active_hotbar_slot: i32,
    pub current_item: i32,
}

impl InventoryPlayer {
    pub fn new() -> Self {
        let mut base = Inventory::new(45);
        base.name = "Inventory".to_string();
        InventoryPlayer {
            base,
            active_hotbar_slot: 0,
            current_item: 0,
        }
    }

    pub fn get_current_item(&mut self) -> Option<&mut ItemStack> {
        if self.current_item < 0 || self.current_item >= 9 {
            return None;
        }
        let slot = self.current_item;
        self.get_stack_in_slot(slot)
    }

    pub fn get_held_item(&mut self) -> Option<&mut ItemStack> {
        if self.active_hotbar_slot < 0 || self.active_hotbar_slot >= 9 {
            return None;
        }
        let slot = self.active_hotbar_slot + 36;
        self.get_stack_in_slot(slot)
    }

    pub fn get_inventory_area_from_slot(&self, slot: i32) -> InvMap {
        if slot == 0 {
            return InvMap::CraftingResult;
        }
        if (1..=4).contains(&slot) {
            return InvMap::CraftingArea;
        }
        if (5..=8).contains(&slot) {
            return InvMap::Armor;
        }
        if (36..=44).contains(&slot) {
            return InvMap::Hotbar;
        }
        if (9..=35).contains(&slot) {
            return InvMap::Inventory;
        }
        global_logger().error(format!("Invalid Inventory area slot! ({slot})\n"));
        InvMap::Invalid // Fallback,
    }

    pub fn get_nbt_slot_id(&self, slot: NetworkSlotId) -> NbtSlotId {
        let slot = slot.value() as i32;
        if (9..=35).contains(&slot) {
            return NbtSlotId(slot as i8);
        }
        if (5..=8).contains(&slot) {
            return NbtSlotId(((5 + (8 - slot)) + 95) as i8);
        }
        if (36..=44).contains(&slot) {
            return NbtSlotId((slot - 36) as i8);
        }
        NbtSlotId(-1)
    }

    // Tries to "pickup" an item. Returns if it succeeded
    // Not sure why vanilla does it in such a convoluted way
    // but it is the way it is
    pub fn pickup_item(&mut self, stack: &mut ItemStack) -> bool {
        // Can we combine with anything in the inventory?
        if self.can_merge_item_stack_in_inventory(stack, false, 9, 35) {
            self.merge_item_stack_in_inventory(stack, false, 9, 35);
            true
        } else {
            // We couldn't combine this stack with anything in the inventory
            // so try the hotbar
            if self.merge_item_stack_in_inventory(stack, false, 36, 44) {
                return true;
            }
            // Try to find an empty slot in the inventory as a last resort
            self.merge_item_stack_in_inventory(stack, false, 9, 35)
        }
    }

    // Returns whether we could merge an item stack without changing the inventory.
    pub fn can_merge_item_stack_in_inventory(
        &mut self,
        stack: &ItemStack,
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

                    if to_move > 0 {
                        return true;
                    }
                }
            }
        }
        false // Give up
    }
}

impl Default for InventoryPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryBehavior for InventoryPlayer {
    fn base(&self) -> &Inventory {
        &self.base
    }
    fn base_mut(&mut self) -> &mut Inventory {
        &mut self.base
    }

    fn get_network_slot_id(&self, slot: NbtSlotId) -> NetworkSlotId {
        let slot = slot.value() as i32;
        if (100..=103).contains(&slot) {
            return NetworkSlotId((5 + (8 - (slot - 95))) as i16);
        }
        if (9..=35).contains(&slot) {
            return NetworkSlotId(slot as i16);
        }
        if (0..=8).contains(&slot) {
            return NetworkSlotId((slot + 36) as i16);
        }
        NetworkSlotId(-1)
    }
}

pub struct InventoryChest {
    pub base: Inventory,
}

impl InventoryChest {
    pub fn new() -> Self {
        let mut base = Inventory::new(27);
        base.name = "Chest".to_string();
        InventoryChest { base }
    }
}

impl Default for InventoryChest {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryBehavior for InventoryChest {
    fn base(&self) -> &Inventory {
        &self.base
    }
    fn base_mut(&mut self) -> &mut Inventory {
        &mut self.base
    }
}

// Just a wrapper for two chest inventories
pub struct InventoryLargeChest<'a> {
    pub base: Inventory,
    pub upper: &'a mut InventoryChest,
    pub lower: &'a mut InventoryChest,
}

impl<'a> InventoryLargeChest<'a> {
    pub fn new(upper: &'a mut InventoryChest, lower: &'a mut InventoryChest) -> Self {
        let mut base = Inventory::new(0);
        base.name = "Large Chest".to_string();
        InventoryLargeChest { base, upper, lower }
    }
}

impl<'a> InventoryBehavior for InventoryLargeChest<'a> {
    fn base(&self) -> &Inventory {
        &self.base
    }
    fn base_mut(&mut self) -> &mut Inventory {
        &mut self.base
    }

    fn get_size_inventory(&self) -> i32 {
        self.upper.get_size_inventory() + self.lower.get_size_inventory()
    }

    fn get_stack_in_slot(&mut self, slot: i32) -> Option<&mut ItemStack> {
        let upper_size = self.upper.get_size_inventory();
        if slot < upper_size {
            self.upper.get_stack_in_slot(slot)
        } else {
            self.lower.get_stack_in_slot(slot - upper_size)
        }
    }

    fn decrease_stack_size(&mut self, slot: i32, count: i32) -> ItemStack {
        let upper_size = self.upper.get_size_inventory();
        if slot < upper_size {
            self.upper.decrease_stack_size(slot, count)
        } else {
            self.lower.decrease_stack_size(slot - upper_size, count)
        }
    }

    fn set_inventory_slot_contents(&mut self, slot: i32, stack: Option<&ItemStack>) {
        let upper_size = self.upper.get_size_inventory();
        if slot < upper_size {
            self.upper.set_inventory_slot_contents(slot, stack);
        } else {
            self.lower
                .set_inventory_slot_contents(slot - upper_size, stack);
        }
    }

    fn on_inventory_changed(&mut self) {
        self.upper.on_inventory_changed();
        self.lower.on_inventory_changed();
    }

    fn merge_item_stack_in_inventory(
        &mut self,
        stack: &mut ItemStack,
        reverse: bool,
        start_slot: i32,
        end_slot: i32,
    ) -> bool {
        let upper_size = self.upper.get_size_inventory();
        let total_size = upper_size + self.lower.get_size_inventory();
        let end = if end_slot == -1 {
            total_size - 1
        } else {
            end_slot
        };

        let mut success = self.upper.merge_item_stack_in_inventory(
            stack,
            reverse,
            0.max(start_slot),
            (upper_size - 1).min(end),
        );

        if !success || stack.count > 0 {
            success = self.lower.merge_item_stack_in_inventory(
                stack,
                reverse,
                0.max(start_slot - upper_size),
                (self.lower.get_size_inventory() - 1).min(end - upper_size),
            );
        }
        success || stack.count == 0
    }
}

pub struct InventoryCraftingTable {
    pub base: Inventory,
}

impl InventoryCraftingTable {
    pub fn new() -> Self {
        let mut base = Inventory::new(10);
        base.name = "Crafting".to_string();
        InventoryCraftingTable { base }
    }
}

impl Default for InventoryCraftingTable {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryBehavior for InventoryCraftingTable {
    fn base(&self) -> &Inventory {
        &self.base
    }
    fn base_mut(&mut self) -> &mut Inventory {
        &mut self.base
    }
}

pub struct InventoryDispenser {
    pub base: Inventory,
    // TODO: Maybe use JavaRandom? (does that matter???)
    pub rng: Random,
}

impl InventoryDispenser {
    pub fn new() -> Self {
        let mut base = Inventory::new(9);
        base.name = "Trap".to_string();
        InventoryDispenser {
            rng: Random::new(),
            base,
        }
    }

    pub fn get_random_stack(&mut self) -> Option<ItemStack> {
        let mut chosen = -1;
        let mut weight = 1;
        for i in 0..9 {
            if self.base.slots[i as usize].id == items::INVALID {
                continue;
            }
            if self.rng.next_int_bound(weight) == 0 {
                chosen = i;
            }
            weight += 1;
        }
        if chosen < 0 {
            return None;
        }
        Some(self.decrease_stack_size(chosen, 1))
    }
}

impl Default for InventoryDispenser {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryBehavior for InventoryDispenser {
    fn base(&self) -> &Inventory {
        &self.base
    }
    fn base_mut(&mut self) -> &mut Inventory {
        &mut self.base
    }
}

// TODO: Maybe make an enum for this?
// Slots: 0 = input, 1 = fuel, 2 = output.
pub struct InventoryFurnace {
    pub base: Inventory,
    pub burn_time: i32,
    pub max_burn_time: i32,
    pub cook_time: i32,
}

impl InventoryFurnace {
    pub fn new() -> Self {
        let mut base = Inventory::new(3);
        base.name = "Furnace".to_string();
        InventoryFurnace {
            base,
            burn_time: 0,
            max_burn_time: 0,
            cook_time: 0,
        }
    }

    pub fn is_burning(&self) -> bool {
        self.burn_time > 0
    }
}

impl Default for InventoryFurnace {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryBehavior for InventoryFurnace {
    fn base(&self) -> &Inventory {
        &self.base
    }
    fn base_mut(&mut self) -> &mut Inventory {
        &mut self.base
    }
}
