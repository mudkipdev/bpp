/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use crate::enums::items;
use crate::inventory::inventories::{InventoryLargeChest, InventoryPlayer};
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

pub struct LargeChestInventoryInteraction<'a> {
    pub base: InventoryInteraction,
    pub player_inventory: &'a mut InventoryPlayer,
    pub upper_chest: Weak<Mutex<dyn TileEntityBehavior + Send>>,
    pub lower_chest: Weak<Mutex<dyn TileEntityBehavior + Send>>,
    pub shared_inventory: Inventory,
}

impl<'a> LargeChestInventoryInteraction<'a> {
    pub fn new(
        pinv: &'a mut InventoryPlayer,
        upper: Arc<Mutex<dyn TileEntityBehavior + Send>>,
        lower: Arc<Mutex<dyn TileEntityBehavior + Send>>,
    ) -> Self {
        let upper_chest = Arc::downgrade(&upper);
        let lower_chest = Arc::downgrade(&lower);
        let mut interaction = LargeChestInventoryInteraction {
            base: InventoryInteraction::new(),
            player_inventory: pinv,
            upper_chest,
            lower_chest,
            shared_inventory: Inventory::new(90),
        };
        interaction.merge_inventories();
        interaction
    }

    pub fn merge_inventories(&mut self) {
        let upper = self
            .upper_chest
            .upgrade()
            .expect("upper chest tile entity dropped");
        let lower = self
            .lower_chest
            .upgrade()
            .expect("lower chest tile entity dropped");
        let mut upper_guard = upper.lock().unwrap();
        let mut lower_guard = lower.lock().unwrap();
        let mut chest_inventory =
            InventoryLargeChest::new(&mut chest_mut(&mut upper_guard).inventory, &mut chest_mut(&mut lower_guard).inventory);

        let mut slot_count = 0usize;
        let size = chest_inventory.get_size_inventory();
        for i in 0..size {
            let stack = chest_inventory
                .get_stack_in_slot(i)
                .copied()
                .unwrap_or_default();
            self.shared_inventory.slots[slot_count] = stack;
            slot_count += 1;
        }
        drop(chest_inventory);
        drop(upper_guard);
        drop(lower_guard);

        for i in 9..45 {
            self.shared_inventory.slots[slot_count] = self.player_inventory.base.slots[i];
            slot_count += 1;
        }
    }

    pub fn write_back(&mut self) {
        let upper = self
            .upper_chest
            .upgrade()
            .expect("upper chest tile entity dropped");
        let lower = self
            .lower_chest
            .upgrade()
            .expect("lower chest tile entity dropped");
        let mut upper_guard = upper.lock().unwrap();
        let mut lower_guard = lower.lock().unwrap();
        let mut chest_inventory =
            InventoryLargeChest::new(&mut chest_mut(&mut upper_guard).inventory, &mut chest_mut(&mut lower_guard).inventory);

        for i in 0..54i32 {
            let slot = self.shared_inventory.slots[i as usize];
            let stack = if slot.id != items::INVALID {
                Some(&slot)
            } else {
                None
            };
            chest_inventory.set_inventory_slot_contents(i, stack);
        }
        drop(chest_inventory);
        drop(upper_guard);
        drop(lower_guard);

        for i in 54..90 {
            self.player_inventory.base.slots[i - 54 + 9] = self.shared_inventory.slots[i];
        }
    }
}

impl<'a> Drop for LargeChestInventoryInteraction<'a> {
    fn drop(&mut self) {
        if self.can_exist() {
            self.write_back();
        }
    }
}

impl<'a> InventoryInteractionBehavior for LargeChestInventoryInteraction<'a> {
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
        self.upper_chest.upgrade().is_some() && self.lower_chest.upgrade().is_some()
    }

    fn init_snapshot(&mut self) {
        let upper = self
            .upper_chest
            .upgrade()
            .expect("upper chest tile entity dropped");
        let lower = self
            .lower_chest
            .upgrade()
            .expect("lower chest tile entity dropped");
        let mut upper_guard = upper.lock().unwrap();
        let mut lower_guard = lower.lock().unwrap();
        let mut chest_inventory =
            InventoryLargeChest::new(&mut chest_mut(&mut upper_guard).inventory, &mut chest_mut(&mut lower_guard).inventory);

        let size = chest_inventory.get_size_inventory();
        self.base.snapshot = Vec::with_capacity(size as usize);
        for i in 0..size {
            let stack = chest_inventory
                .get_stack_in_slot(i)
                .copied()
                .unwrap_or_default();
            self.base.snapshot.push(stack);
        }
    }

    // Analyze the snapshot vs the current chest inventory
    fn tick_diff(&mut self) -> Vec<DeltaSlot> {
        let mut differences = Vec::new();
        {
            let upper = self
                .upper_chest
                .upgrade()
                .expect("upper chest tile entity dropped");
            let lower = self
                .lower_chest
                .upgrade()
                .expect("lower chest tile entity dropped");
            let mut upper_guard = upper.lock().unwrap();
            let mut lower_guard = lower.lock().unwrap();
            let mut chest_inventory =
                InventoryLargeChest::new(&mut chest_mut(&mut upper_guard).inventory, &mut chest_mut(&mut lower_guard).inventory);

            for i in 0..self.base.snapshot.len() {
                let current = chest_inventory
                    .get_stack_in_slot(i as i32)
                    .copied()
                    .unwrap_or_default();
                if self.base.snapshot[i] == current {
                    continue;
                }
                self.base.snapshot[i] = current;
                differences.push(DeltaSlot {
                    stack: self.base.snapshot[i],
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

        if slot <= 53 {
            // Chest -> inventory
            let _success = self
                .player_inventory
                .merge_item_stack_in_inventory(&mut copy, true, 9, 44);
        } else {
            // Inventory -> Chest
            let upper = self
                .upper_chest
                .upgrade()
                .expect("upper chest tile entity dropped");
            let lower = self
                .lower_chest
                .upgrade()
                .expect("lower chest tile entity dropped");
            let mut upper_guard = upper.lock().unwrap();
            let mut lower_guard = lower.lock().unwrap();
            let mut chest_inventory =
                InventoryLargeChest::new(&mut chest_mut(&mut upper_guard).inventory, &mut chest_mut(&mut lower_guard).inventory);
            let _success = chest_inventory.merge_item_stack_in_inventory(&mut copy, false, 0, -1);
        }

        // Update the source in the real inventory before re-merging
        if slot <= 53 {
            let upper = self
                .upper_chest
                .upgrade()
                .expect("upper chest tile entity dropped");
            let lower = self
                .lower_chest
                .upgrade()
                .expect("lower chest tile entity dropped");
            let mut upper_guard = upper.lock().unwrap();
            let mut lower_guard = lower.lock().unwrap();
            let mut chest_inventory =
                InventoryLargeChest::new(&mut chest_mut(&mut upper_guard).inventory, &mut chest_mut(&mut lower_guard).inventory);
            let ptr = if copy.count == 0 { None } else { Some(&copy) };
            chest_inventory.set_inventory_slot_contents(slot, ptr);
        } else {
            let player_slot = (slot - 54 + 9) as usize;
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
