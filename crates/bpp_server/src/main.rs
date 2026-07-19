/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
pub mod blocks;
pub mod chunk_io;
pub mod commands;
pub mod entities;
pub mod packet;
pub mod player_conn;
pub mod server;
pub mod server_socket;

use std::sync::atomic::Ordering;

use bpp_shared::helpers::java::java_math::MathHelper;
use bpp_shared::helpers::platforms::{ARCH_NAME, BUILD_MODE, PLATFORM_NAME};
use bpp_shared::logger::logger::global_logger;
use bpp_shared::version::PROJECT_FULL_VERSION_LABEL;

use crate::server::{SHUTDOWN_REQUESTED, Server};

struct Args {
    port: u16,
    max_players: i32,
    enable_whitelist: bool,
    seed: i64,
    disable_portals: bool,
    force_nether_spawn: bool,
    pregen_radius: u32,
    chunk_render_radius: u32,
    chunk_gen_radius: u32,
    chunk_tick_radius: u32,
    entity_render_radius: u32,
    entity_tick_radius: u32,
}

impl Args {
    fn version() -> String {
        PROJECT_FULL_VERSION_LABEL.to_string()
    }

    fn parse(args: &[String]) -> Args {
        let mut parsed = Args {
            port: 25565,
            max_players: -1,
            enable_whitelist: false,
            seed: 0,
            disable_portals: false,
            force_nether_spawn: false,
            pregen_radius: 5,
            chunk_render_radius: 5,
            chunk_gen_radius: 5,
            chunk_tick_radius: 5,
            entity_render_radius: 5,
            entity_tick_radius: 5,
        };

        let mut iter = args.iter().skip(1);
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--port" | "-p" => {
                    if let Some(value) = iter.next() {
                        parsed.port = value.parse().unwrap_or(parsed.port);
                    }
                }
                "--max_players" => {
                    if let Some(value) = iter.next() {
                        parsed.max_players = value.parse().unwrap_or(parsed.max_players);
                    }
                }
                "--whitelist" | "-w" => parsed.enable_whitelist = true,
                "--seed" | "-s" => {
                    if let Some(value) = iter.next() {
                        parsed.seed = value.parse().unwrap_or(parsed.seed);
                    }
                }
                "--no_portals" => parsed.disable_portals = true,
                "--force_nether_spawn" => parsed.force_nether_spawn = true,
                "--pregen_radius" => {
                    if let Some(value) = iter.next() {
                        parsed.pregen_radius = value.parse().unwrap_or(parsed.pregen_radius);
                    }
                }
                "--chunk_render_radius" => {
                    if let Some(value) = iter.next() {
                        parsed.chunk_render_radius = value.parse().unwrap_or(parsed.chunk_render_radius);
                    }
                }
                "--chunk_gen_radius" => {
                    if let Some(value) = iter.next() {
                        parsed.chunk_gen_radius = value.parse().unwrap_or(parsed.chunk_gen_radius);
                    }
                }
                "--chunk_tick_radius" => {
                    if let Some(value) = iter.next() {
                        parsed.chunk_tick_radius = value.parse().unwrap_or(parsed.chunk_tick_radius);
                    }
                }
                "--entity_render_radius" => {
                    if let Some(value) = iter.next() {
                        parsed.entity_render_radius = value.parse().unwrap_or(parsed.entity_render_radius);
                    }
                }
                "--entity_tick_radius" => {
                    if let Some(value) = iter.next() {
                        parsed.entity_tick_radius = value.parse().unwrap_or(parsed.entity_tick_radius);
                    }
                }
                "--version" => {
                    println!("{}", Args::version());
                    std::process::exit(0);
                }
                _ => {}
            }
        }

        parsed
    }
}

fn signal_handler(_sig: i32) {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

fn main() {
    // Hook up signals
    let _ = ctrlc::set_handler(|| signal_handler(0));
    // Parse CLI Args
    let raw_args: Vec<String> = std::env::args().collect();
    let _args = Args::parse(&raw_args);
    // Init the sine table
    MathHelper::init_sin_table();
    // We're ready to roll
    global_logger().info(format!("Running on {PLATFORM_NAME} ({BUILD_MODE}, {ARCH_NAME})\n"));

    let mut server = Server::new();
    server.run();
}
