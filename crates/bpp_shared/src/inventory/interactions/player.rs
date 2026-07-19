/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/
use crate::enums::items;
use crate::inventory::interactions::crafting::{
    CraftingInventoryInteraction, CraftingInventoryInteractionBehavior, crafting_on_left_click,
    crafting_on_right_click, crafting_on_shift_click,
};
use crate::inventory::inventories::{InvMap, InventoryPlayer};
use crate::inventory::inventory::InventoryBehavior;
use crate::inventory::inventory_interaction::{InventoryInteraction, InventoryInteractionBehavior};
use crate::inventory::item_stack::ItemStack;
use crate::numeric_structs::UInt8_2;
use crate::runtime::Runtime;

pub struct PlayerInventoryInteraction<'a> {
    pub base: CraftingInventoryInteraction<'a>,
    pub player_inventory: &'a mut InventoryPlayer,
}

impl<'a> PlayerInventoryInteraction<'a> {
    pub fn new(inv: &'a mut InventoryPlayer, game_runtime: &'a Runtime) -> Self {
        PlayerInventoryInteraction {
            base: CraftingInventoryInteraction::new(game_runtime, UInt8_2::new(2, 2)),
            player_inventory: inv,
        }
    }

    pub fn on_close(&mut self) {}
}

impl<'a> InventoryInteractionBehavior for PlayerInventoryInteraction<'a> {
    fn base(&self) -> &InventoryInteraction {
        &self.base.base
    }
    fn base_mut(&mut self) -> &mut InventoryInteraction {
        &mut self.base.base
    }

    fn inventory_and_carried(&mut self) -> (&mut dyn InventoryBehavior, &mut ItemStack) {
        (&mut *self.player_inventory, &mut self.base.base.carried)
    }

    fn can_exist(&mut self) -> bool {
        true
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

impl<'a> CraftingInventoryInteractionBehavior for PlayerInventoryInteraction<'a> {
    fn runtime(&self) -> &Runtime {
        self.base.runtime
    }

    fn grid_size(&self) -> UInt8_2 {
        self.base.grid_size
    }

    fn craft_inventory(&mut self) -> &mut dyn InventoryBehavior {
        &mut *self.player_inventory
    }

    fn shift_click_result(&mut self) {
        let result = self.player_inventory.base.slots[0];
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
            self.player_inventory.base.slots[0] = if copy.count == 0 {
                ItemStack::default()
            } else {
                copy
            };
        }
    }

    fn shift_click_other(&mut self, slot: i32) {
        let from = self.player_inventory.get_inventory_area_from_slot(slot);
        let stack = match self.player_inventory.get_stack_in_slot(slot) {
            Some(stack) => *stack,
            None => return,
        };

        let mut copy = stack; // work on a copy so we can detect what moved

        if matches!(
            from,
            InvMap::Armor | InvMap::CraftingArea | InvMap::CraftingResult | InvMap::Hotbar
        ) {
            let success = self
                .player_inventory
                .merge_item_stack_in_inventory(&mut copy, false, 9, 35);
            if !success {
                self.player_inventory
                    .merge_item_stack_in_inventory(&mut copy, false, 36, 44);
            }
        } else {
            self.player_inventory
                .merge_item_stack_in_inventory(&mut copy, false, 36, 44);
        }

        // Update the source slot to whatever count is left
        if copy.count == 0 {
            self.player_inventory.clear_slot(slot);
        } else if let Some(stack) = self.player_inventory.get_stack_in_slot(slot) {
            stack.count = copy.count;
        }
    }
}
