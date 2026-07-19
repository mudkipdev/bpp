/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
use bpp_shared::base_types::{NetworkSlotId, WindowId};
use bpp_shared::inventory::inventory::Inventory;
use bpp_shared::inventory::item_stack::ItemStack;
use bpp_shared::networking::packets::{self, PacketBehavior};

use crate::player_conn::player_session::{ActiveContainer, PlayerSession};

pub fn send_inventory(session: &mut PlayerSession, window_id: WindowId, inventory: Inventory) {
    let mut items: Vec<ItemStack> = Vec::new();
    for item in &inventory.slots {
        items.push(ItemStack { id: item.id, count: item.count, data: item.data });
    }
    let mut fc = packets::FillContainer::new();
    fc.window_id = window_id;
    fc.items = items;
    fc.serialize(&mut session.stream);
}

// Sends a single slot update. windowId=-1 / slotId=-1 updates the cursor.
pub fn send_slot(session: &mut PlayerSession, window_id: WindowId, slot_id: NetworkSlotId, stack: Option<&ItemStack>) {
    let mut pkt = packets::SetSlot::new();
    pkt.window_id = window_id;
    pkt.slot_id = slot_id;
    pkt.item = match stack {
        Some(stack) => ItemStack { id: stack.id, count: stack.count, data: stack.data },
        None => ItemStack::default(),
    };
    pkt.serialize(&mut session.stream);
}

pub fn close_container(session: &mut PlayerSession) {
    // Get rid of our active interaction and reset the window id
    let mut cc = packets::CloseContainer::new();
    cc.window_id = session.open_window_id;
    cc.serialize(&mut session.stream);
    session.active_container = ActiveContainer::None;
    session.open_window_id = WindowId(0);
}
