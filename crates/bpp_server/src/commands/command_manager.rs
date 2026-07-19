/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/

use std::sync::{Mutex, MutexGuard, OnceLock};

use bpp_shared::logger::logger::global_logger;
use bpp_shared::networking::packets::{ChatMessage, PacketBehavior};
use bpp_shared::world::world::WorldManager;

use crate::commands::command::{
    CommandBehavior, CommandDimension, CommandGive, CommandHelp, CommandList, CommandLoaded, CommandSeed,
    CommandSpawn, CommandTeleport, CommandTime, CommandVersion, ERROR_REASON_NO_CMD,
};
use crate::player_conn::player_session::PlayerSession;
use crate::server::Server;

static REGISTERED_COMMANDS: OnceLock<Mutex<Vec<Box<dyn CommandBehavior + Send>>>> = OnceLock::new();

/// @brief Responsible for all command handling and execution
pub struct CommandManager;

impl CommandManager {
    // Register all commands
    pub fn init(_server: &mut Server) {
        let mut registered_commands: Vec<Box<dyn CommandBehavior + Send>> = Vec::new();
        // Anyone can run these
        registered_commands.push(Box::new(CommandHelp::new()));
        registered_commands.push(Box::new(CommandTeleport::new()));
        registered_commands.push(Box::new(CommandTime::new()));
        registered_commands.push(Box::new(CommandSeed::new()));
        registered_commands.push(Box::new(CommandSpawn::new()));
        registered_commands.push(Box::new(CommandGive::new()));
        registered_commands.push(Box::new(CommandList::new()));
        registered_commands.push(Box::new(CommandLoaded::new()));
        registered_commands.push(Box::new(CommandDimension::new()));
        registered_commands.push(Box::new(CommandVersion::new()));
        /*
        registeredCommands.push_back(CommandPose());
        // Needs at least creative mode to run
        registeredCommands.push_back(CommandHealth());
        // Must be operator
        registeredCommands.push_back(CommandUptime());
        registeredCommands.push_back(CommandOp());
        registeredCommands.push_back(CommandDeop());
        registeredCommands.push_back(CommandWhitelist());
        registeredCommands.push_back(CommandKick());
        registeredCommands.push_back(CommandCreative());
        registeredCommands.push_back(CommandSound());
        registeredCommands.push_back(CommandKill());
        registeredCommands.push_back(CommandGamerule());
        registeredCommands.push_back(CommandSave());
        registeredCommands.push_back(CommandStop());
        registeredCommands.push_back(CommandFree());
        registeredCommands.push_back(CommandUsage());
        registeredCommands.push_back(CommandSummon());
        registeredCommands.push_back(CommandPopulated());
        registeredCommands.push_back(CommandInterface());
        registeredCommands.push_back(CommandRegion());
        registeredCommands.push_back(CommandEntity());
        registeredCommands.push_back(CommandModified());
        registeredCommands.push_back(CommandPacket());
        */
        global_logger().info(format!("Registered {} command(s)!\n", registered_commands.len()));
        REGISTERED_COMMANDS.set(Mutex::new(registered_commands)).ok();
    }

    // Get all registered commands
    pub fn get_registered_commands() -> MutexGuard<'static, Vec<Box<dyn CommandBehavior + Send>>> {
        REGISTERED_COMMANDS
            .get()
            .expect("CommandManager::init was not called")
            .lock()
            .unwrap()
    }

    // Parses commands and executes them
    pub fn parse(
        cmd_string: &mut String,
        session: &mut PlayerSession,
        world: &mut WorldManager,
        transfer_dimension: &mut dyn FnMut(&mut PlayerSession),
        server: &mut Server,
    ) {
        // Remove initial /
        *cmd_string = cmd_string[1..].to_string();
        // Set these up for command parsing
        let mut failure_reason = "Syntax".to_string();
        let mut command: Vec<String> = Vec::new();

        for token in cmd_string.split(' ') {
            // store token string in the vector
            command.push(token.to_string());
        }
        // No arguments passed, exit early
        if command.is_empty() || cmd_string.is_empty() {
            failure_reason = ERROR_REASON_NO_CMD.to_string();
        } else {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // TODO: Make this efficient
                let registered_commands = Self::get_registered_commands();
                // This'll throw an out of bounds error
                let index = registered_commands.iter().position(|cmd| cmd.base().get_label() == command[0]);
                drop(registered_commands);

                index.map(|index| {
                    // Take the command out so we don't hold the registry lock while executing it
                    // (a command's execute() may itself call get_registered_commands()).
                    let mut cmd = Self::get_registered_commands().remove(index);
                    let reason = cmd.execute(&mut command, session, world, transfer_dimension, server);
                    Self::get_registered_commands().insert(index, cmd);
                    reason
                })
            }));
            match result {
                Ok(Some(reason)) => failure_reason = reason,
                Ok(None) => {}
                Err(e) => {
                    let message = if let Some(s) = e.downcast_ref::<&str>() {
                        (*s).to_string()
                    } else if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown error".to_string()
                    };
                    global_logger().info(format!("{message} on /{cmd_string}\n"));
                }
            }
        }

        let mut fail_pkt = ChatMessage::new();
        if !failure_reason.is_empty() {
            if failure_reason == "Syntax" {
                fail_pkt.message = format!("§cInvalid Syntax \"{cmd_string}\"");
            } else {
                fail_pkt.message = format!("§c{failure_reason}");
            }
            fail_pkt.serialize(&mut session.stream);
        }
    }
}
