/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/

use bpp_shared::constants::PLAYER_EYE_HEIGHT;
use bpp_shared::networking::packets::{PacketBehavior, PlayerPositionAndRotation};
use bpp_shared::numeric_structs::{Double2, Double3, Float2, Float3, Int2, Int3, Vec2, Vec3};
use bpp_shared::world::world::WorldManager;

use crate::player_conn::player_session::PlayerSession;
use crate::server::Server;

pub const ERROR_OPERATOR: &str = "Only operators can use this command!";
pub const ERROR_CREATIVE: &str = "Only creative players can use this command!";
pub const ERROR_WHITELIST: &str = "Only whitelisted players can use this command!";
pub const ERROR_REASON_SYNTAX: &str = "Invalid Syntax";
pub const ERROR_REASON_PARAMETERS: &str = "Invalid Parameters";
pub const ERROR_REASON_ERROR: &str = "Error";
pub const ERROR_REASON_NO_CMD: &str = "No command passed";

pub const MAX_CHAT_LINE_SIZE: i32 = 60;

// Small define for a bit less copy-paste
// #define DEFINE_COMMAND(name, label, description, syntax, requiresOp, requiresCreative) is expressed as
// the define_command! macro below.
macro_rules! define_command {
    ($name:ident, $label:expr, $description:expr, $syntax:expr, $requires_op:expr, $requires_creative:expr) => {
        pub struct $name {
            pub base: Command,
        }

        impl $name {
            pub fn new() -> Self {
                $name {
                    base: Command::new($label, $description, $syntax, $requires_op, $requires_creative),
                }
            }
        }
    };
}

/*
#define DEFINE_PERMSCHECK(session)                                                                                      \
	std::string perms = CheckPermissions(client);                                                                      \
	if (!perms.empty())                                                                                                \
		return perms;
*/

// Base class for how a command is defined
pub struct Command {
    label: String,
    description: String,
    syntax: String,
    requires_op: bool,
    requires_creative: bool,
}

impl Command {
    pub fn get_label(&self) -> &str {
        &self.label
    }
    pub fn get_description(&self) -> &str {
        &self.description
    }
    pub fn get_syntax(&self) -> &str {
        &self.syntax
    }
    pub fn get_requires_operator(&self) -> bool {
        self.requires_op
    }
    pub fn get_requires_creative(&self) -> bool {
        self.requires_creative
    }

    pub fn new(label: &str, description: &str, syntax: &str, requires_op: bool, requires_creative: bool) -> Self {
        Command {
            label: label.to_string(),
            description: description.to_string(),
            syntax: syntax.to_string(),
            requires_op,
            requires_creative,
        }
    }
}

pub trait CommandBehavior {
    fn base(&self) -> &Command;
    fn base_mut(&mut self) -> &mut Command;

    fn execute(
        &mut self,
        parameters: &mut Vec<String>,
        session: &mut PlayerSession,
        world: &mut WorldManager,
        transfer_dimension: &mut dyn FnMut(&mut PlayerSession),
        server: &mut Server,
    ) -> String;
}

// Commands
// Anyone can run these
define_command!(CommandHelp, "help", "Lists commands or helps with command", "[command]", false, false);
define_command!(
    CommandTeleport,
    "tp",
    "Teleports player to coordinates or another player",
    "<player> <x> <y> <z> / <player> <player>",
    false,
    false
);
define_command!(CommandTime, "time", "Gets or sets the current world time", "<new_time>", false, false);
define_command!(CommandSpawn, "spawn", "Teleport to spawn", "", false, false);
define_command!(CommandSeed, "seed", "Get the world seed", "", false, false);
define_command!(CommandGive, "give", "Give yourself a block or item", "<id>:[meta] [amount]", false, false);
define_command!(CommandList, "list", "List all currently online players", "", false, false);
define_command!(CommandLoaded, "loaded", "Shows the number of loaded chunks", "", false, false);
define_command!(CommandDimension, "dim", "Swap to the other dimension", "", false, false);
define_command!(CommandVersion, "version", "Shows the current Server version", "", false, false);
/*
DEFINE_COMMAND(CommandPose, "pose", "Set the current players' pose", "<crouch/fire/sit>", false, false);
DEFINE_COMMAND(CommandInterface, "interface", "Open the desired interface", "<id>", false, false);
// Needs at least creative mode to run
DEFINE_COMMAND(CommandGive, "give", "Give yourself a block or item", "<id> [meta] [amount]", false, true);
DEFINE_COMMAND(CommandHealth, "health", "Get or Set Player Health", "[health]", false, true);
// Must be operator
DEFINE_COMMAND(CommandUptime, "uptime", "Shows how long the server has been alive for in ticks", "", true, false);
DEFINE_COMMAND(CommandOp, "op", "Grant a player operator privlidges", "[player]", true, false);
DEFINE_COMMAND(CommandDeop, "deop", "Revoke a players' operator privlidges", "[player]", true, false);
DEFINE_COMMAND(CommandWhitelist, "whitelist", "Modify the whitelist", "<reload/list> / <add/remove> <player>", true,
			   false);
DEFINE_COMMAND(CommandKick, "kick", "Kick a player from the server", "[player]", true, false);
DEFINE_COMMAND(CommandCreative, "creative", "Toggle creative mode", "", true, false);
DEFINE_COMMAND(CommandSound, "sound", "Play a specified sound", "<id> [meta]", true, false);
DEFINE_COMMAND(CommandKill, "kil", "Kill the specified player", "[player]", true, false);
DEFINE_COMMAND(CommandGamerule, "gamerule", "Configure gamerules", "<rule> <state>", true, false);
DEFINE_COMMAND(CommandSave, "save", "Forces the server to save all loaded chunks", "", true, false);
DEFINE_COMMAND(CommandStop, "stop", "Forces the server to stop", "", true, false);
DEFINE_COMMAND(CommandFree, "free", "Forces the server to unload chunks nobody can see", "", true, false);
DEFINE_COMMAND(CommandLoaded, "loaded", "Shows the number of loaded chunks", "", true, false);
DEFINE_COMMAND(CommandUsage, "usage", "Shows the current memory usage in megabytes", "", true, false);
DEFINE_COMMAND(CommandSummon, "summon", "Summon a player entity", "<player>", true, false);
DEFINE_COMMAND(CommandPopulated, "populated", "Check the population status of the current chunk", "", true, false);
DEFINE_COMMAND(CommandRegion, "region", "Test the region infrastructure", "<action>", true, false);
DEFINE_COMMAND(CommandEntity, "entity", "Get the latest entity id", "", true, false);
DEFINE_COMMAND(CommandModified, "modified", "Get the number of modified chunks", "", true, false);
DEFINE_COMMAND(CommandPacket, "packet", "Send a custom packet", "[broadcast] <data>", true, false);
*/

// Helper: send a PlayerPositionAndRotation packet to move a session to new coords.
pub fn send_teleport(target: &mut PlayerSession, position: Vec3, yaw: f32, pitch: f32) {
    // This is hacky but force gen the chunk we are at
    let chunk_pos = Int2::new((position.x as i32) >> 4, (position.z as i32) >> 4);
    let entity = target.entity.as_ref().unwrap();
    let world = entity.lock().unwrap().base.base.world.upgrade().expect("world dropped");
    world.lock().unwrap().force_gen_chunk_sync(chunk_pos);

    // Update our server-side entity position to match the teleport, so that movement broadcasts are correct.
    entity.lock().unwrap().base.base.teleport(position, Vec2::new(f64::from(yaw), f64::from(pitch)));

    // Keep server-side position in sync so movement broadcasts are correct.
    target.position.pos = position;

    let mut pkt = PlayerPositionAndRotation::new();
    pkt.position.x = position.x;
    pkt.position.y = position.y;
    pkt.camera_y = position.y + PLAYER_EYE_HEIGHT;
    pkt.position.z = position.z;
    pkt.rotation.x = yaw;
    pkt.rotation.y = pitch;
    pkt.on_ground = false;
    pkt.serialize(&mut target.stream);
}

pub fn parse_int3(offset: &mut usize, parameters: &[String]) -> Int3 {
    let out = Int3::new(
        parameters[*offset].parse::<i32>().unwrap(),
        parameters[*offset + 1].parse::<i32>().unwrap(),
        parameters[*offset + 2].parse::<i32>().unwrap(),
    );
    *offset += 3;
    out
}

pub fn parse_float2(offset: &mut usize, parameters: &[String]) -> Float2 {
    let out = Float2::new(
        parameters[*offset].parse::<f32>().unwrap(),
        parameters[*offset + 1].parse::<f32>().unwrap(),
    );
    *offset += 2;
    out
}

pub fn parse_float3(offset: &mut usize, parameters: &[String]) -> Float3 {
    let out = Float3::new(
        parameters[*offset].parse::<f32>().unwrap(),
        parameters[*offset + 1].parse::<f32>().unwrap(),
        parameters[*offset + 2].parse::<f32>().unwrap(),
    );
    *offset += 3;
    out
}

pub fn parse_double2(offset: &mut usize, parameters: &[String]) -> Double2 {
    let out = Double2::new(
        parameters[*offset].parse::<f64>().unwrap(),
        parameters[*offset + 1].parse::<f64>().unwrap(),
    );
    *offset += 2;
    out
}

pub fn parse_double3(offset: &mut usize, parameters: &[String]) -> Double3 {
    let out = Double3::new(
        parameters[*offset].parse::<f64>().unwrap(),
        parameters[*offset + 1].parse::<f64>().unwrap(),
        parameters[*offset + 2].parse::<f64>().unwrap(),
    );
    *offset += 3;
    out
}
