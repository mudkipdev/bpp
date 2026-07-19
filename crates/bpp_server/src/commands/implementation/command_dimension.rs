/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/

use bpp_shared::enums::dimensions::Dimension;
use bpp_shared::networking::packets::{ChatMessage, PacketBehavior};
use bpp_shared::world::world::WorldManager;

use crate::commands::command::{Command, CommandBehavior, CommandDimension};
use crate::player_conn::player_session::PlayerSession;
use crate::server::Server;

// Transfers the player to the opposite dimension.
// Usage:
//   /dim
impl CommandBehavior for CommandDimension {
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
        _world: &mut WorldManager,
        _transfer_dimension: &mut dyn FnMut(&mut PlayerSession),
        server: &mut Server,
    ) -> String {
        let mut reply = ChatMessage::new();
        reply.message = if session.dimension == 0 {
            "§7Transferring to the Nether...".to_string()
        } else {
            "§7Transferring to the Overworld...".to_string()
        };
        reply.serialize(&mut session.stream);

        let new_dim = if session.dimension == -1 { Dimension::Overworld } else { Dimension::Nether };

        server.send_player_to_dimension(new_dim, session);

        String::new()
    }
}
