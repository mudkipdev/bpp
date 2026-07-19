/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/

use bpp_shared::constants::PLAYER_EYE_HEIGHT;
use bpp_shared::numeric_structs::{Int2, Vec3};
use bpp_shared::world::world::WorldManager;

use crate::commands::command::{Command, CommandBehavior, CommandSpawn, send_teleport};
use crate::player_conn::player_session::PlayerSession;
use crate::server::Server;

// Teleport to spawn
// Usage:
//   /spawn
impl CommandBehavior for CommandSpawn {
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
        let mut ipos = world.get_spawn_point(false);
        world.force_gen_chunk_sync(Int2::new(ipos.x >> 4, ipos.z >> 4));
        ipos.y = world.get_height_value(
            ipos.x,
            ipos.z,
        ); // So we don't clip in the ground since get spawn point gives the raw data which defaults to y=64

        send_teleport(
            session,
            Vec3::new(f64::from(ipos.x) + 0.5, f64::from(ipos.y) + PLAYER_EYE_HEIGHT + 0.0625, f64::from(ipos.z) + 0.5),
            0.0,
            0.0,
        );
        String::new()
    }
}
