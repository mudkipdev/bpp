/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/

use bpp_shared::base_types::ItemId;
use bpp_shared::enums::blocks::{BLOCK_AIR, BLOCK_MAX};
use bpp_shared::inventory::inventory::InventoryBehavior;
use bpp_shared::inventory::item_stack::ItemStack;
use bpp_shared::items::item_properties;
use bpp_shared::networking::packets::{ChatMessage, PacketBehavior};
use bpp_shared::strings::labels::w_id_to_label;
use bpp_shared::world::world::WorldManager;

use crate::commands::command::{Command, CommandBehavior, CommandGive};
use crate::packet::packet_utils;
use crate::player_conn::player_session::PlayerSession;
use crate::server::Server;

// Give yourself a block or item
// Usage:
//   /give <id>[:meta] [amount]
impl CommandBehavior for CommandGive {
    fn base(&self) -> &Command {
        &self.base
    }
    fn base_mut(&mut self) -> &mut Command {
        &mut self.base
    }

    fn execute(
        &mut self,
        parameters: &mut Vec<String>,
        session: &mut PlayerSession,
        _world: &mut WorldManager,
        _transfer_dimension: &mut dyn FnMut(&mut PlayerSession),
        _server: &mut Server,
    ) -> String {
        // TODO: Let player specify another player to give to
        if parameters.len() <= 1 {
            return "Missing item id!".to_string();
        }

        let mut item = ItemStack::default();
        let item_arg = &parameters[1];
        let colon_pos = item_arg.find(':');
        let id_string = match colon_pos {
            Some(pos) => &item_arg[..pos],
            None => &item_arg[..],
        };
        let mut meta_string = "";
        if let Some(pos) = colon_pos {
            meta_string = &item_arg[pos + 1..];
        }
        item.id = ItemId(id_string.parse::<i32>().unwrap() as i16);
        if !meta_string.is_empty() {
            item.data = meta_string.parse::<i32>().unwrap() as i16;
        }
        item.count = item_properties::get_max_stack(item.id) as i8; // I don't want 64 pickaxes anymore!!
        if parameters.len() > 2 {
            item.count = parameters[2].parse::<i32>().unwrap() as i8;
        }

        // Check if its even a valid item
        if (item.id.value() as i32 > BLOCK_AIR.0 as i32 && (item.id.value() as i32) < BLOCK_MAX.0 as i32)
            || item_properties::is_valid(item.id)
        {
            let mut reply = ChatMessage::new();
            reply.message = format!(
                "§eGave {} ({}:{}) x{} to {}",
                w_id_to_label(item.id.value()),
                item.id.value(),
                item.data,
                item.count,
                session.username
            );

            reply.serialize(&mut session.stream);

            // Try the hotbar
            if session.inventory.merge_item_stack_in_inventory(&mut item, false, 36, 44) {
                let window_id = session.open_window_id;
                let inventory = session.inventory.base.clone();
                packet_utils::send_inventory(session, window_id, inventory);
                return String::new();
            }

            // Try the main inventory
            if session.inventory.merge_item_stack_in_inventory(&mut item, false, 9, 35) {
                let window_id = session.open_window_id;
                let inventory = session.inventory.base.clone();
                packet_utils::send_inventory(session, window_id, inventory);
                return String::new();
            }

            // TODO: Drop on the ground
            return String::new();
        }
        format!("{} is not a valid item id!", item.id.value())
    }
}
