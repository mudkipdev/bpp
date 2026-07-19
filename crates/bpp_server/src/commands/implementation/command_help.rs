/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/

use bpp_shared::networking::packets::{ChatMessage, PacketBehavior};
use bpp_shared::world::world::WorldManager;

use crate::commands::command::{Command, CommandBehavior, CommandHelp, ERROR_REASON_SYNTAX, MAX_CHAT_LINE_SIZE};
use crate::commands::command_manager::CommandManager;
use crate::player_conn::player_session::PlayerSession;
use crate::server::Server;

// Lists commands or helps with command
// Usage:
//   /help
//   /help [command]
impl CommandBehavior for CommandHelp {
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
        //DEFINE_PERMSCHECK(pClient)
        let registered_commands = CommandManager::get_registered_commands();
        let mut pkt = ChatMessage::new();
        // Get help with specific command
        if parameters.len() > 1 {
            for i in 0..registered_commands.len() {
                if registered_commands[i].base().get_label() == parameters[1] {
                    pkt.message =
                        format!("§7{}: {}", registered_commands[i].base().get_label(), registered_commands[i].base().get_description());
                    pkt.serialize(&mut session.stream);
                    // Only print syntax if it has a value
                    if !registered_commands[i].base().get_syntax().is_empty() {
                        pkt.message =
                            format!("§7/{} {}", registered_commands[i].base().get_label(), registered_commands[i].base().get_syntax());
                        pkt.serialize(&mut session.stream);
                    }
                    if registered_commands[i].base().get_requires_operator() {
                        pkt.message = "§7(Requires operator)".to_string();
                        pkt.serialize(&mut session.stream);
                    }
                    return String::new();
                }
            }
            return "Command not found!".to_string();
        } else {
            // List all commands
            pkt.message = "§7-- All commands --".to_string();
            pkt.serialize(&mut session.stream);
            pkt.message = "§7".to_string();
            for i in 0..registered_commands.len() {
                pkt.message += registered_commands[i].base().get_label();
                if i < registered_commands.len() - 1 {
                    pkt.message += ", ";
                }
                if pkt.message.len() as i32 > MAX_CHAT_LINE_SIZE || i == registered_commands.len() - 1 {
                    pkt.serialize(&mut session.stream);
                    pkt.message = "§7".to_string();
                }
            }
            return String::new();
        }
        ERROR_REASON_SYNTAX.to_string()
    }
}
