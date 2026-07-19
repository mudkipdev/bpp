/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
use std::sync::{Arc, Mutex, OnceLock};

use bpp_shared::enums::blocks::{BLOCK_CHEST, BLOCK_CRAFTING_TABLE};
use bpp_shared::enums::network::packet_data;
use bpp_shared::networking::packets::{OpenContainer, PacketBehavior};
use bpp_shared::numeric_structs::Int3;
use bpp_shared::runtime::Runtime;
use bpp_shared::tile_entities::tile_entity::TileEntityChest;
use bpp_shared::world::world::WorldManager;

use crate::packet::packet_utils;
use crate::player_conn::player_session::{ActiveContainer, PlayerSession};

// Behavioral overrides for the base block behaviors.
// If its nullptr here, use the base block overrides. Else, use what is here.
#[derive(Clone, Copy, Default)]
pub struct BlockBehavior {
    pub on_block_activated:
        Option<fn(&mut WorldManager, Int3, &mut PlayerSession, &mut Runtime) -> bool>,
}

static BLOCK_BEHAVIORS: OnceLock<[BlockBehavior; 256]> = OnceLock::new();

pub fn block_behaviors() -> &'static [BlockBehavior; 256] {
    BLOCK_BEHAVIORS
        .get()
        .expect("initialize() must run before block_behaviors() is read")
}

pub fn initialize() {
    let mut behaviors = [BlockBehavior::default(); 256];

    // Register unique behaviors here
    behaviors[BLOCK_CRAFTING_TABLE.0 as u8 as usize].on_block_activated = Some(
        |world: &mut WorldManager,
         position: Int3,
         session: &mut PlayerSession,
         game_runtime: &mut Runtime|
         -> bool {
            let mut ow = OpenContainer::new();
            ow.window_id = session.get_next_window_id();
            ow.slot_count = 9;
            ow.title = "Crafting".to_string();
            ow.window_type = packet_data::CRAFTING_TABLE;
            ow.serialize(&mut session.stream);

            session.active_container = ActiveContainer::CraftingTable(position);
            session.with_active_interaction(game_runtime, world, |i| i.init_snapshot());
            false
        },
    );
    behaviors[BLOCK_CHEST.0 as u8 as usize].on_block_activated = Some(
        |world: &mut WorldManager,
         position: Int3,
         session: &mut PlayerSession,
         game_runtime: &mut Runtime|
         -> bool {
            let mut chest = world.get_tile_entity_shared::<TileEntityChest>(position);
            if chest.is_none() {
                let new_chest = Arc::new(Mutex::new(TileEntityChest::new(position)));
                world.create_tile_entity(new_chest.clone());
                chest = Some(new_chest);
            }
            let chest = chest.unwrap();

            // Are we a double chest?
            let l = world.get_block_id(Int3::new(position.x - 1, position.y, position.z));
            let r = world.get_block_id(Int3::new(position.x + 1, position.y, position.z));
            let f = world.get_block_id(Int3::new(position.x, position.y, position.z - 1));
            let b = world.get_block_id(Int3::new(position.x, position.y, position.z + 1));
            let double_chest =
                l == BLOCK_CHEST || r == BLOCK_CHEST || f == BLOCK_CHEST || b == BLOCK_CHEST;

            if double_chest {
                let partner_chest = if l == BLOCK_CHEST {
                    world.get_tile_entity_shared::<TileEntityChest>(Int3::new(
                        position.x - 1,
                        position.y,
                        position.z,
                    ))
                } else if r == BLOCK_CHEST {
                    world.get_tile_entity_shared::<TileEntityChest>(Int3::new(
                        position.x + 1,
                        position.y,
                        position.z,
                    ))
                } else if f == BLOCK_CHEST {
                    world.get_tile_entity_shared::<TileEntityChest>(Int3::new(
                        position.x,
                        position.y,
                        position.z - 1,
                    ))
                } else {
                    world.get_tile_entity_shared::<TileEntityChest>(Int3::new(
                        position.x,
                        position.y,
                        position.z + 1,
                    ))
                };
                let partner_chest = match partner_chest {
                    Some(partner_chest) => partner_chest,
                    None => return false,
                };

                let is_left_side = r == BLOCK_CHEST || b == BLOCK_CHEST;
                let (chest, partner_chest) = if is_left_side {
                    (chest, partner_chest)
                } else {
                    (partner_chest, chest)
                };

                let mut ow = OpenContainer::new();
                ow.window_id = session.get_next_window_id();
                ow.slot_count = 54;
                ow.title = "Large Chest".to_string();
                ow.window_type = packet_data::CHEST;
                ow.serialize(&mut session.stream);

                session.active_container =
                    ActiveContainer::LargeChest(Arc::downgrade(&chest), Arc::downgrade(&partner_chest));
                session.with_active_interaction(game_runtime, world, |i| i.init_snapshot());

                let inventory = session
                    .with_active_interaction(game_runtime, world, |i| i.inventory().base().clone())
                    .expect("just opened large chest interaction");
                let window_id = session.open_window_id;
                packet_utils::send_inventory(session, window_id, inventory);
                return false;
            }

            // Setup interaction
            session.active_container = ActiveContainer::Chest(Arc::downgrade(&chest));
            session.with_active_interaction(game_runtime, world, |i| i.init_snapshot());

            // Single chest
            // Open the chest window
            let mut ow = OpenContainer::new();
            ow.window_id = session.get_next_window_id();
            ow.slot_count = 27;
            ow.title = "Chest".to_string();
            ow.window_type = packet_data::CHEST;
            ow.serialize(&mut session.stream);

            // Send inventory
            let inventory = session
                .with_active_interaction(game_runtime, world, |i| i.inventory().base().clone())
                .expect("just opened chest interaction");
            let window_id = session.open_window_id;
            packet_utils::send_inventory(session, window_id, inventory);
            false
        },
    );

    let _ = BLOCK_BEHAVIORS.set(behaviors);
}
