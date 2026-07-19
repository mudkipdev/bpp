/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/

use bpp_shared::networking::packets::{ChatMessage, PacketBehavior};
use bpp_shared::world::world::WorldManager;

use crate::commands::command::{Command, CommandBehavior, CommandList};
use crate::player_conn::player_session::PlayerSession;
use crate::server::Server;

// List all currently online players
// Usage:
//   /list
impl CommandBehavior for CommandList {
    fn base(&self) -> &Command {
        &self.base
    }
    fn base_mut(&mut self) -> &mut Command {
        &mut self.base
    }

    fn execute(
        &mut self,
        _parameters: &mut Vec<String>,
        _session: &mut PlayerSession,
        _world: &mut WorldManager,
        _transfer_dimension: &mut dyn FnMut(&mut PlayerSession),
        server: &mut Server,
    ) -> String {
        let players = server.get_players();
        let mut pkt = ChatMessage::new();
        pkt.message = format!("§7-- {} Player(s) --", players.len());
        pkt.serialize(&mut _session.stream);
        pkt.message = "§7".to_string();
        for i in 0..players.len() {
            let username = match players[i].try_lock() {
                Ok(p) => p.username.clone(),
                Err(_) => _session.username.clone(),
            };
            if pkt.message.len() + username.len() > 64 {
                pkt.serialize(&mut _session.stream);
                pkt.message = "§7".to_string();
            }
            pkt.message += &username;
            pkt.message += if i < (players.len() - 1) { ", " } else { "" };
        }
        pkt.serialize(&mut _session.stream);
        String::new()
    }
}
