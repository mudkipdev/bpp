/*
 * Copyright (c) 2026, jwaxy <jwaxy.is-a.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/
use crate::enums::blocks::BLOCK_CRAFTING_TABLE;
use crate::enums::items;
use crate::inventory::interactions::crafting::{
    CraftingInventoryInteraction, CraftingInventoryInteractionBehavior, crafting_on_left_click,
    crafting_on_right_click, crafting_on_shift_click, crafting_update_result,
};
use crate::inventory::inventories::{InventoryCraftingTable, InventoryPlayer};
use crate::inventory::inventory::{Inventory, InventoryBehavior};
use crate::inventory::inventory_interaction::{
    DeltaSlot, InventoryInteraction, InventoryInteractionBehavior,
};
use crate::inventory::item_stack::ItemStack;
use crate::numeric_structs::{Int3, UInt8_2};
use crate::runtime::Runtime;
use crate::world::world::WorldManager;

pub struct CraftingTableInventoryInteraction<'a> {
    pub base: CraftingInventoryInteraction<'a>,
    pub craft_inventory: InventoryCraftingTable,
    pub world: &'a WorldManager,
    pub block_position: Int3,
    pub player_inventory: &'a mut InventoryPlayer,
    pub shared_inventory: Inventory,
}

impl<'a> CraftingTableInventoryInteraction<'a> {
    pub fn new(
        pinv: &'a mut InventoryPlayer,
        world: &'a WorldManager,
        game_runtime: &'a Runtime,
        crafting_table_pos: Int3,
    ) -> Self {
        let mut interaction = CraftingTableInventoryInteraction {
            base: CraftingInventoryInteraction::new(game_runtime, UInt8_2::new(3, 3)),
            craft_inventory: InventoryCraftingTable::new(),
            world,
            block_position: crafting_table_pos,
            player_inventory: pinv,
            shared_inventory: Inventory::new(46),
        };
        interaction.merge_inventories();
        interaction
    }

    pub fn write_back(&mut self) {
        let mut slot_count = 0usize;
        for i in 0..10 {
            self.craft_inventory.base.slots[i] = self.shared_inventory.slots[slot_count];
            slot_count += 1;
        }
        for i in 9..45 {
            self.player_inventory.base.slots[i] = self.shared_inventory.slots[slot_count];
            slot_count += 1;
        }
    }

    fn merge_inventories(&mut self) {
        let mut slot_count = 0usize;
        for i in 0..10 {
            self.shared_inventory.slots[slot_count] = self.craft_inventory.base.slots[i];
            slot_count += 1;
        }
        for i in 9..45 {
            self.shared_inventory.slots[slot_count] = self.player_inventory.base.slots[i];
            slot_count += 1;
        }
    }
}

impl<'a> Drop for CraftingTableInventoryInteraction<'a> {
    fn drop(&mut self) {
        self.write_back();

        let total = self.base.grid_size.total() as usize;
        for i in 1..=total {
            let mut stack = self.craft_inventory.base.slots[i];
            if stack.id == items::INVALID {
                continue;
            }
            self.player_inventory
                .merge_item_stack_in_inventory(&mut stack, true, 9, 44);
        }
    }
}

impl<'a> InventoryInteractionBehavior for CraftingTableInventoryInteraction<'a> {
    fn base(&self) -> &InventoryInteraction {
        &self.base.base
    }
    fn base_mut(&mut self) -> &mut InventoryInteraction {
        &mut self.base.base
    }

    fn inventory_and_carried(&mut self) -> (&mut dyn InventoryBehavior, &mut ItemStack) {
        (&mut self.shared_inventory, &mut self.base.base.carried)
    }

    fn propagate_change(&mut self) {
        self.write_back();
    }

    fn can_exist(&mut self) -> bool {
        self.world.get_block_id(self.block_position) == BLOCK_CRAFTING_TABLE
    }

    fn init_snapshot(&mut self) {
        self.base.base.snapshot = self.shared_inventory.slots.clone();
    }

    fn tick_diff(&mut self) -> Vec<DeltaSlot> {
        let mut differences = Vec::new();
        self.merge_inventories(); // make sure sharedInventory is current before diffing
        for i in 0..self.base.base.snapshot.len() {
            if self.base.base.snapshot[i] == self.shared_inventory.slots[i] {
                continue;
            }
            self.base.base.snapshot[i] = self.shared_inventory.slots[i];
            differences.push(DeltaSlot {
                stack: self.base.base.snapshot[i],
                slot: i as i32,
            });
        }
        differences
    }

    fn on_left_click(&mut self, slot: i32) {
        crafting_on_left_click(self, slot);
    }

    fn on_right_click(&mut self, slot: i32) {
        crafting_on_right_click(self, slot);
    }

    fn on_shift_click(&mut self, slot: i32) {
        crafting_on_shift_click(self, slot);
    }
}

impl<'a> CraftingInventoryInteractionBehavior for CraftingTableInventoryInteraction<'a> {
    fn runtime(&self) -> &Runtime {
        self.base.runtime
    }

    fn grid_size(&self) -> UInt8_2 {
        self.base.grid_size
    }

    fn craft_inventory(&mut self) -> &mut dyn InventoryBehavior {
        &mut self.craft_inventory
    }

    fn update_result(&mut self) {
        crafting_update_result(self);
        self.merge_inventories();
    }

    fn shift_click_result(&mut self) {
        let result = self.craft_inventory.base.slots[0];
        if result.id == items::INVALID {
            return;
        }

        let mut copy = result;
        if self
            .player_inventory
            .merge_item_stack_in_inventory(&mut copy, true, 9, 44)
        {
            self.finish_craft();
        } else {
            self.craft_inventory.base.slots[0] = if copy.count == 0 {
                ItemStack::default()
            } else {
                copy
            };
        }
    }

    fn shift_click_other(&mut self, slot: i32) {
        let stack = match self.shared_inventory.get_stack_in_slot(slot) {
            Some(stack) => *stack,
            None => return,
        };

        let mut copy = stack;

        if slot < 10 {
            // Grid -> inventory
            // Try the main inventory then the hotbar
            let success = self
                .player_inventory
                .merge_item_stack_in_inventory(&mut copy, false, 9, 35);
            if !success {
                self.player_inventory
                    .merge_item_stack_in_inventory(&mut copy, false, 36, 44);
            }
        } else {
            // We can't shift click into the crafting grid itself, so just try the other area of the inventory
            // We shift clicked in the inventory
            if slot > 9 && slot < 37 {
                self.player_inventory
                    .merge_item_stack_in_inventory(&mut copy, false, 36, 44);
            } else {
                self.player_inventory
                    .merge_item_stack_in_inventory(&mut copy, false, 9, 35);
            }
        }

        // Update the source in the real inventory before re-merging
        if slot < 10 {
            self.craft_inventory.base.slots[slot as usize] = if copy.count == 0 {
                ItemStack::default()
            } else {
                copy
            };
        } else {
            self.player_inventory.base.slots[(slot - 1) as usize] = if copy.count == 0 {
                ItemStack::default()
            } else {
                copy
            };
        }

        // Re-sync sharedInventory from the real inventories
        self.merge_inventories();
    }
}
