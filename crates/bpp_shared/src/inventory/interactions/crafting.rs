/*
 * Copyright (c) 2026, jwaxy <jwaxy.is-a.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/
use crate::enums::items;
use crate::inventory::inventory::InventoryBehavior;
use crate::inventory::inventory_interaction::{
    InventoryInteraction, InventoryInteractionBehavior, default_on_left_click,
    default_on_right_click,
};
use crate::inventory::item_stack::ItemStack;
use crate::items::item_properties;
use crate::numeric_structs::UInt8_2;
use crate::runtime::Runtime;

pub struct CraftingInventoryInteraction<'a> {
    pub base: InventoryInteraction,
    pub runtime: &'a Runtime,
    pub grid_size: UInt8_2,
}

impl<'a> CraftingInventoryInteraction<'a> {
    // We don't do any inventory merging in this class because
    // sharedInventory, craftingInventory, playerInventory all may or may not be the same thing.
    pub fn new(runtime: &'a Runtime, grid_size: UInt8_2) -> Self {
        CraftingInventoryInteraction {
            base: InventoryInteraction::new(),
            runtime,
            grid_size,
        }
    }
}

pub trait CraftingInventoryInteractionBehavior: InventoryInteractionBehavior {
    fn runtime(&self) -> &Runtime;
    fn grid_size(&self) -> UInt8_2;

    fn craft_inventory(&mut self) -> &mut dyn InventoryBehavior;

    fn update_result(&mut self) {
        crafting_update_result(self);
    }

    fn finish_craft(&mut self) {
        self.craft_inventory().base_mut().slots[0] = ItemStack::default();

        // Consume one of each ingredient that went into this craft
        let total = self.grid_size().total() as usize;
        for i in 1..=total {
            let mut stack = self.craft_inventory().base().slots[i];
            stack.decrement_count(1);
            self.craft_inventory().base_mut().slots[i] = stack;
        }

        // The grid changed, there might be another possible craft
        self.update_result();
    }

    fn take_result(&mut self) {
        let result = self.craft_inventory().base().slots[0];
        if result.id == items::INVALID {
            return;
        }

        let carried = self.base().carried;
        if carried.id == items::INVALID {
            self.base_mut().carried = result;
        } else if carried.id == result.id && carried.data == result.data {
            // Same type, try merge with cursor
            let max_stack = item_properties::get_max_stack(carried.id);
            if carried.count as i32 + result.count as i32 > max_stack {
                return;
            }
            self.base_mut().carried.count += result.count;
        } else {
            // Cursor holds something else
            return;
        }

        self.finish_craft();
    }

    fn handle_crafting(&mut self, slot: i32) {
        let total = self.grid_size().total() as i32;
        if slot == 0 || slot > total {
            return;
        }

        self.update_result();
    }

    fn shift_click_result(&mut self);
    fn shift_click_other(&mut self, slot: i32);
}

pub fn crafting_update_result<T: CraftingInventoryInteractionBehavior + ?Sized>(
    interaction: &mut T,
) {
    let grid_size = interaction.grid_size();
    let total = grid_size.total() as usize;
    let grid: Vec<ItemStack> = interaction.craft_inventory().base().slots[1..1 + total].to_vec();
    let result = interaction
        .runtime()
        .recipe_manager
        .match_grid(&grid, grid_size);
    interaction.craft_inventory().base_mut().slots[0] = result;
}

pub fn crafting_on_left_click<T: CraftingInventoryInteractionBehavior + ?Sized>(
    interaction: &mut T,
    slot: i32,
) {
    if slot == 0 {
        interaction.take_result();
    } else {
        let (inventory, carried) = interaction.inventory_and_carried();
        let changed = default_on_left_click(inventory, carried, slot);
        if changed {
            interaction.propagate_change();
        }
    }
    interaction.handle_crafting(slot);
}

pub fn crafting_on_right_click<T: CraftingInventoryInteractionBehavior + ?Sized>(
    interaction: &mut T,
    slot: i32,
) {
    if slot == 0 {
        interaction.take_result();
    } else {
        let (inventory, carried) = interaction.inventory_and_carried();
        let changed = default_on_right_click(inventory, carried, slot);
        if changed {
            interaction.propagate_change();
        }
    }
    interaction.handle_crafting(slot);
}

pub fn crafting_on_shift_click<T: CraftingInventoryInteractionBehavior + ?Sized>(
    interaction: &mut T,
    slot: i32,
) {
    if slot == 0 {
        interaction.shift_click_result();
    } else {
        interaction.shift_click_other(slot);
    }
    interaction.handle_crafting(slot);
}
