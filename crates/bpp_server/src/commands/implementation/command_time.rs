/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/

use bpp_shared::networking::packets::{ChatMessage, PacketBehavior};
use bpp_shared::world::world::WorldManager;

use crate::commands::command::{Command, CommandBehavior, CommandTime, ERROR_REASON_SYNTAX};
use crate::player_conn::player_session::PlayerSession;
use crate::server::Server;

// Gets or sets the current world time
// Usage:
//   /time
//   /time <new_time>
impl CommandBehavior for CommandTime {
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
        world: &mut WorldManager,
        _transfer_dimension: &mut dyn FnMut(&mut PlayerSession),
        _server: &mut Server,
    ) -> String {
        // Set the time
        if parameters.len() > 2 {
            if parameters[1] == "set" {
                world.elapsed_ticks = parameters[2].parse::<i64>().unwrap();
            } else if parameters[1] == "add" {
                world.elapsed_ticks += parameters[2].parse::<i64>().unwrap();
            } else {
                return format!("Invalid argument {}", parameters[1]);
            }
            let mut reply = ChatMessage::new();
            reply.message = format!("§eSet time to {}", world.elapsed_ticks);
            reply.serialize(&mut session.stream);
            return String::new();
        }

        // Get the time
        if parameters.len() == 1 {
            let mut reply = ChatMessage::new();
            reply.message = format!("§eCurrent Time is {}", world.elapsed_ticks);
            reply.serialize(&mut session.stream);
            return String::new();
        }
        ERROR_REASON_SYNTAX.to_string()
    }
}
