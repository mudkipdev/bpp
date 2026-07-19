/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use crate::inventory::inventories::InventoryPlayer;
use crate::inventory::inventory::{Inventory, InventoryBehavior};
use crate::inventory::inventory_interaction::{
    DeltaSlot, InventoryInteraction, InventoryInteractionBehavior,
};
use crate::inventory::item_stack::ItemStack;
use crate::tile_entities::tile_entity::{TileEntityBehavior, TileEntityChest};

fn chest_mut<'a>(guard: &'a mut MutexGuard<'_, dyn TileEntityBehavior + Send + 'static>) -> &'a mut TileEntityChest {
    guard
        .as_any_mut()
        .downcast_mut::<TileEntityChest>()
        .expect("tile entity was not a TileEntityChest")
}

pub struct ChestInventoryInteraction<'a> {
    pub base: InventoryInteraction,
    pub player_inventory: &'a mut InventoryPlayer,
    pub chest_handle: Weak<Mutex<dyn TileEntityBehavior + Send>>,
    pub shared_inventory: Inventory,
}

impl<'a> ChestInventoryInteraction<'a> {
    pub fn new(pinv: &'a mut InventoryPlayer, chest: Arc<Mutex<dyn TileEntityBehavior + Send>>) -> Self {
        let chest_handle = Arc::downgrade(&chest);
        let mut interaction = ChestInventoryInteraction {
            base: InventoryInteraction::new(),
            player_inventory: pinv,
            chest_handle,
            shared_inventory: Inventory::new(63),
        };
        interaction.merge_inventories();
        interaction
    }

    pub fn merge_inventories(&mut self) {
        let chest = self
            .chest_handle
            .upgrade()
            .expect("chest tile entity dropped");
        let mut guard = chest.lock().unwrap();
        let mut slot_count = 0usize;
        for slot in chest_mut(&mut guard).inventory.base.slots.iter() {
            self.shared_inventory.slots[slot_count] = *slot;
            slot_count += 1;
        }
        drop(guard);
        for i in 9..45 {
            self.shared_inventory.slots[slot_count] = self.player_inventory.base.slots[i];
            slot_count += 1;
        }
    }

    pub fn write_back(&mut self) {
        let chest = self
            .chest_handle
            .upgrade()
            .expect("chest tile entity dropped");
        let mut guard = chest.lock().unwrap();
        let chest_entity = chest_mut(&mut guard);
        for i in 0..27 {
            chest_entity.inventory.base.slots[i] = self.shared_inventory.slots[i];
        }
        drop(guard);
        for i in 27..63 {
            self.player_inventory.base.slots[i - 27 + 9] = self.shared_inventory.slots[i];
        }
    }
}

impl<'a> Drop for ChestInventoryInteraction<'a> {
    fn drop(&mut self) {
        if self.can_exist() {
            self.write_back();
        }
    }
}

impl<'a> InventoryInteractionBehavior for ChestInventoryInteraction<'a> {
    fn base(&self) -> &InventoryInteraction {
        &self.base
    }
    fn base_mut(&mut self) -> &mut InventoryInteraction {
        &mut self.base
    }

    fn inventory_and_carried(&mut self) -> (&mut dyn InventoryBehavior, &mut ItemStack) {
        (&mut self.shared_inventory, &mut self.base.carried)
    }

    fn propagate_change(&mut self) {
        self.write_back();
    }

    fn can_exist(&mut self) -> bool {
        self.chest_handle.upgrade().is_some()
    }

    fn init_snapshot(&mut self) {
        let chest = self
            .chest_handle
            .upgrade()
            .expect("chest tile entity dropped");
        let mut guard = chest.lock().unwrap();
        self.base.snapshot = chest_mut(&mut guard).inventory.base.slots.clone();
    }

    // Analyze the snapshot vs the current chest inventory
    fn tick_diff(&mut self) -> Vec<DeltaSlot> {
        let mut differences = Vec::new();
        let chest = self
            .chest_handle
            .upgrade()
            .expect("chest tile entity dropped");
        {
            let mut guard = chest.lock().unwrap();
            let chest_entity = chest_mut(&mut guard);
            for i in 0..self.base.snapshot.len() {
                let current = chest_entity.inventory.base.slots[i];
                if self.base.snapshot[i] == current {
                    continue;
                }
                self.base.snapshot[i] = current;
                differences.push(DeltaSlot {
                    stack: current,
                    slot: i as i32,
                });
            }
        }
        self.merge_inventories();
        differences
    }

    fn on_shift_click(&mut self, slot: i32) {
        let stack = match self.shared_inventory.get_stack_in_slot(slot) {
            Some(stack) => *stack,
            None => return,
        };

        let mut copy = stack;

        if slot <= 26 {
            // Chest -> inventory
            let _success = self
                .player_inventory
                .merge_item_stack_in_inventory(&mut copy, true, 9, 44);
        } else {
            // Inventory -> Chest
            let chest = self
                .chest_handle
                .upgrade()
                .expect("chest tile entity dropped");
            let mut guard = chest.lock().unwrap();
            let _success = chest_mut(&mut guard)
                .inventory
                .merge_item_stack_in_inventory(&mut copy, false, 0, -1);
        }

        // Update the source in the real inventory before re-merging
        if slot <= 26 {
            let chest = self
                .chest_handle
                .upgrade()
                .expect("chest tile entity dropped");
            let mut guard = chest.lock().unwrap();
            chest_mut(&mut guard).inventory.base.slots[slot as usize] = if copy.count == 0 {
                ItemStack::default()
            } else {
                copy
            };
        } else {
            let player_slot = (slot - 27 + 9) as usize;
            self.player_inventory.base.slots[player_slot] = if copy.count == 0 {
                ItemStack::default()
            } else {
                copy
            };
        }

        // Re-sync sharedInventory from the real inventories
        self.merge_inventories();
    }
}
