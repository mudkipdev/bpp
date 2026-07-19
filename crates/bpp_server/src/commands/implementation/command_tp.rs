/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/

use bpp_shared::networking::packets::{ChatMessage, PacketBehavior};
use bpp_shared::world::world::WorldManager;

use crate::commands::command::{
    Command, CommandBehavior, CommandTeleport, ERROR_REASON_PARAMETERS, ERROR_REASON_SYNTAX, parse_double3,
    send_teleport,
};
use crate::player_conn::player_session::PlayerSession;
use crate::server::Server;

// Teleports a player to coordinates or to another player.
// Usage:
//   /tp <x> <y> <z>
//   /tp <player> <x> <y> <z>
//   /tp <source_player> <target_player>
impl CommandBehavior for CommandTeleport {
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
        if parameters.len() < 2 {
            return ERROR_REASON_SYNTAX.to_string();
        }

        let mut source_exists = false;
        let mut offset: usize = 1;

        // Check if player is even passed
        // Inspired by https://stackoverflow.com/a/16575564
        {
            let num = parameters[offset].parse::<f64>();
            if num.is_ok() {
                source_exists = true;
            } else {
                // source = FindSession(session, parameters[offset++]);
                return String::new();
            }
        }

        // TODO Should prolly report if a non-existent player runs this
        if !source_exists {
            return format!("{} does not exist!", parameters[offset - 1]);
        }

        // /tp <player> <x> <y> <z>
        if parameters.len() - offset >= 3 {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let pos = parse_double3(&mut offset, parameters);
                send_teleport(session, pos, 0.0, 0.0);

                let mut reply = ChatMessage::new();
                reply.message = format!("§eTeleported {} to {}", session.username, pos);
                reply.serialize(&mut session.stream);
            }));
            return match result {
                Ok(()) => String::new(),
                Err(_) => ERROR_REASON_PARAMETERS.to_string(),
            };
        }

        // /tp <player> <target_player>
        /*
        if (parameters.size() - offset == 1) { // offset=1→params[1], offset=2→params[2]
        	PlayerSession* dest = FindSession(session, parameters[offset]);
        	if (!dest)
        		return parameters[offset] + " does not exist!";
        	SendTeleport(*source, dest->position.pos, dest->rotation.x, dest->rotation.y);
        	Packet::ChatMessage reply;
        	reply.message = "§eTeleported " + source->username + " to " + session.username;
        	reply.Serialize(session.stream);
        	return "";
        }
        */

        ERROR_REASON_SYNTAX.to_string()
    }
}
