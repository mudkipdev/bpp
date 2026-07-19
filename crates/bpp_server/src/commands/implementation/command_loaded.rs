/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/

use bpp_shared::networking::packets::{ChatMessage, PacketBehavior};
use bpp_shared::world::world::WorldManager;

use crate::commands::command::{Command, CommandBehavior, CommandLoaded};
use crate::player_conn::player_session::PlayerSession;
use crate::server::Server;

// Shows the number of loaded chunks
// Usage:
//   /loaded
impl CommandBehavior for CommandLoaded {
    fn base(&self) -> &Command {
        &self.base
    }
    fn base_mut(&mut self) -> &mut Command {
        &mut self.base
    }

    fn execute(
        &mut self,
        _parameters: &mut Vec<String>,
        session: &mut PlayerSession,
        world: &mut WorldManager,
        _transfer_dimension: &mut dyn FnMut(&mut PlayerSession),
        _server: &mut Server,
    ) -> String {
        let mut reply = ChatMessage::new();
        reply.message = format!("§e{}", world.chunks.len());
        reply.serialize(&mut session.stream);
        String::new()
    }
}
