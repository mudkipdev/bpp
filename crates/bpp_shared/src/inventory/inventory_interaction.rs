/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 * Copyright (c) 2026, jwaxy <jwaxy.is-a.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/
use crate::enums::items;
use crate::inventory::inventory::InventoryBehavior;
use crate::inventory::item_stack::ItemStack;
use crate::items::item_properties;

pub struct DeltaSlot {
    pub stack: ItemStack,
    pub slot: i32,
}

// Used for actually interacting with inventories, will typically wrap 1 or more inventory objects for things like chests, etc
pub struct InventoryInteraction {
    pub snapshot: Vec<ItemStack>,
    pub carried: ItemStack,
}

impl InventoryInteraction {
    pub fn new() -> Self {
        InventoryInteraction {
            snapshot: Vec::new(),
            carried: ItemStack::default(),
        }
    }
}

impl Default for InventoryInteraction {
    fn default() -> Self {
        Self::new()
    }
}

pub trait InventoryInteractionBehavior {
    fn base(&self) -> &InventoryInteraction;
    fn base_mut(&mut self) -> &mut InventoryInteraction;

    // Returns the inventory this interaction wraps together with the carried (cursor) stack
    fn inventory_and_carried(&mut self) -> (&mut dyn InventoryBehavior, &mut ItemStack);

    fn inventory(&mut self) -> &mut dyn InventoryBehavior {
        self.inventory_and_carried().0
    }

    // Hook that propagates a change on the wrapped inventory out to whatever real inventories it
    // was merged from (used by the chest/crafting table shared inventories)
    fn propagate_change(&mut self) {}

    fn can_exist(&mut self) -> bool {
        true
    }

    // Take a snapshot of this inventory
    fn init_snapshot(&mut self) {
        let slots = self.inventory().base().slots.clone();
        self.base_mut().snapshot = slots;
    }

    // Analyze the snapshot vs the current inventory
    // Returns a list of slots that are different
    fn tick_diff(&mut self) -> Vec<DeltaSlot> {
        let mut differences = Vec::new();
        let len = self.base().snapshot.len();
        for i in 0..len {
            let _current = self.inventory().get_stack_in_slot(i as i32);
            let current = self.inventory().base().slots[i];
            let snap = self.base().snapshot[i];

            if snap == current {
                continue;
            }

            self.base_mut().snapshot[i] = current;
            differences.push(DeltaSlot {
                stack: current,
                slot: i as i32,
            });
        }
        differences
    }

    fn on_left_click(&mut self, slot: i32) {
        let (inventory, carried) = self.inventory_and_carried();
        let changed = default_on_left_click(inventory, carried, slot);
        if changed {
            self.propagate_change();
        }
    }

    fn on_right_click(&mut self, slot: i32) {
        let (inventory, carried) = self.inventory_and_carried();
        let changed = default_on_right_click(inventory, carried, slot);
        if changed {
            self.propagate_change();
        }
    }

    fn on_shift_click(&mut self, slot: i32) {
        let inventory = self.inventory();
        default_on_shift_click(inventory, slot);
        self.propagate_change();
    }
}

pub fn default_on_left_click(
    inventory: &mut dyn InventoryBehavior,
    carried: &mut ItemStack,
    slot: i32,
) -> bool {
    // Empty slot
    let target = match inventory.get_stack_in_slot(slot) {
        Some(stack) => *stack,
        None => {
            if carried.id != items::INVALID {
                let c = *carried;
                inventory.set_inventory_slot_contents(slot, Some(&c));
                *carried = ItemStack::default();
            }
            inventory.on_inventory_changed();
            return true;
        }
    };

    // Not carrying anything
    if carried.id == items::INVALID {
        *carried = target;
        inventory.clear_slot(slot);
        inventory.on_inventory_changed();
        return true;
    }

    // Same item; merge
    if target.id == carried.id && target.data == carried.data {
        let max_stack = item_properties::get_max_stack(target.id);
        let space = max_stack - target.count as i32;
        let to_move = space.min(carried.count as i32);
        let new_target = ItemStack {
            id: target.id,
            count: target.count + to_move as i8,
            data: target.data,
        };
        inventory.set_inventory_slot_contents(slot, Some(&new_target));
        carried.count -= to_move as i8;
        if carried.count == 0 {
            *carried = ItemStack::default();
        }
        inventory.on_inventory_changed();
        return true;
    }

    // Different item; swap
    inventory.set_inventory_slot_contents(slot, Some(&*carried));
    *carried = target;
    inventory.on_inventory_changed();
    true
}

pub fn default_on_right_click(
    inventory: &mut dyn InventoryBehavior,
    carried: &mut ItemStack,
    slot: i32,
) -> bool {
    let target = inventory.get_stack_in_slot(slot).map(|stack| *stack);

    if carried.id != items::INVALID {
        let target = match target {
            Some(target) => target,
            None => {
                let single = ItemStack {
                    id: carried.id,
                    count: 1,
                    data: carried.data,
                };
                inventory.set_inventory_slot_contents(slot, Some(&single));
                carried.count -= 1;
                if carried.count == 0 {
                    *carried = ItemStack::default();
                }
                inventory.on_inventory_changed();
                return true;
            }
        };

        // If we right click on the same item we are carrying just add one
        if target.id == carried.id && target.data == carried.data {
            let max_stack = item_properties::get_max_stack(target.id);
            let space = max_stack - target.count as i32;
            if space >= 1 {
                let new_target = ItemStack {
                    id: target.id,
                    count: target.count + 1,
                    data: target.data,
                };
                inventory.set_inventory_slot_contents(slot, Some(&new_target));
                carried.count -= 1;
                if carried.count == 0 {
                    *carried = ItemStack::default();
                }
                inventory.on_inventory_changed();
                return true;
            }
            return false;
        }

        // If we right click on a different item, swap the cursor and that item
        inventory.set_inventory_slot_contents(slot, Some(&*carried));
        *carried = target;
        inventory.on_inventory_changed();
        return true;
    }

    let target = match target {
        Some(target) => target,
        None => return false,
    };

    // Only split items if there stack count is greater than 1 and we aren't carrying anything
    if target.count > 1 {
        // Beta always take the higher of the two if uneven
        let taken = (target.count as i32 + 1) / 2;
        let left = target.count as i32 - taken;
        let new_target = ItemStack {
            id: target.id,
            count: left as i8,
            data: target.data,
        };
        inventory.set_inventory_slot_contents(slot, Some(&new_target));
        *carried = ItemStack {
            id: target.id,
            count: taken as i8,
            data: target.data,
        };
        inventory.on_inventory_changed();
        return true;
    }

    // If its only one item we just pick it up
    *carried = target;
    inventory.clear_slot(slot);
    inventory.on_inventory_changed();
    true
}

pub fn default_on_shift_click(inventory: &mut dyn InventoryBehavior, slot: i32) {
    let mut copy = match inventory.get_stack_in_slot(slot) {
        Some(stack) => *stack,
        None => return,
    };
    inventory.merge_item_stack_in_inventory(&mut copy, false, 0, -1);
    inventory.set_inventory_slot_contents(slot, Some(&copy));
}
