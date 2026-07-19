/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/

use bpp_shared::networking::packets::{ChatMessage, PacketBehavior};
use bpp_shared::version::{PROJECT_NAME, PROJECT_VERSION_FULL_STRING};
use bpp_shared::world::world::WorldManager;

use crate::commands::command::{Command, CommandBehavior, CommandVersion};
use crate::player_conn::player_session::PlayerSession;
use crate::server::Server;

// Shows the current Server version
// Usage:
//   /version
impl CommandBehavior for CommandVersion {
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
        _server: &mut Server,
    ) -> String {
        let mut pkt = ChatMessage::new();
        pkt.message = format!("§eCurrent {PROJECT_NAME} version is {PROJECT_VERSION_FULL_STRING}");
        pkt.serialize(&mut session.stream);
        String::new()
    }
}
