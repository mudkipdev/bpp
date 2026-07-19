/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 * Copyright (c) 2026, jwaxy <jwaxy.is-a.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::sync::{Arc, Mutex};

use crate::blocks::block_registration;
use crate::crafting::recipe_manager::RecipeManager;
use crate::helpers::java::java_random::Random;
use crate::items::item_properties;
use crate::logger::logger::global_logger;
use crate::world::storage::region_manager::RegionManager;
use crate::world::storage::save_manager::{LevelData, SaveManager};
use crate::world::world::WorldManager;

// General game runtime that the client and server can use so that way we don't reuse a bunch of code and have to maintain it in two places.
pub struct Runtime {
    // Storage
    pub save_manager: SaveManager,
    pub world: Arc<Mutex<WorldManager>>,
    pub world_hell: Arc<Mutex<WorldManager>>,
    pub overworld_region_manager: Arc<Mutex<RegionManager>>,
    pub hell_region_manager: Arc<Mutex<RegionManager>>, // hehe i call it hell instead of nether cause im quirky

    // Gameplay
    pub recipe_manager: RecipeManager,
}

impl Runtime {
    pub fn new() -> Self {
        let mut recipe_manager = RecipeManager::default();
        block_registration::register_all();
        item_properties::register_all();
        recipe_manager.add_vanilla_recipes();
        global_logger().info("New game runtime created!\n");
        Self {
            save_manager: SaveManager::default(),
            world: Arc::new(Mutex::new(WorldManager::default())),
            world_hell: Arc::new(Mutex::new(WorldManager::new(true))),
            overworld_region_manager: Arc::new(Mutex::new(RegionManager::default())),
            hell_region_manager: Arc::new(Mutex::new(RegionManager::default())),
            recipe_manager,
        }
    }

    pub fn init(&mut self, level_path: &str, seed_override: &str) {
        // Setup our save
        let mut new_save = false;
        if !self.save_manager.initialize(level_path) {
            global_logger().warn("**** FAILED TO LOAD WORLD DATA! Attempting to create new world... \n");
            new_save = true;
            let random_seed = if !seed_override.is_empty() {
                self.save_manager.seed_from_string(seed_override)
            } else {
                Random::new().next_long()
            };
            if !self.save_manager.create_new_world(LevelData {
                random_seed,
                ..LevelData::default()
            }) {
                global_logger().error("**** FAILED TO CREATE NEW WORLD! \n");
                std::process::exit(1);
            }
            global_logger().info("New world created successfully. \n");
        }

        // Initialize our region managers
        self.overworld_region_manager
            .lock()
            .unwrap()
            .initialize(format!("{level_path}/region"));
        self.hell_region_manager
            .lock()
            .unwrap()
            .initialize(format!("{level_path}/DIM-1/region"));

        // Bind our pointers
        self.overworld_region_manager.lock().unwrap().world = Arc::downgrade(&self.world);
        self.hell_region_manager.lock().unwrap().world = Arc::downgrade(&self.world_hell);

        // Initialize save data with our world objects
        self.save_manager.load_level_data();
        let random_seed = self.save_manager.get_level_data().random_seed;
        self.world.lock().unwrap().init_world_seed(random_seed);
        self.world_hell.lock().unwrap().init_world_seed(random_seed);

        // World time
        let time = self.save_manager.get_level_data().time;
        self.world.lock().unwrap().elapsed_ticks = time;
        self.world_hell.lock().unwrap().elapsed_ticks = time;

        // Bind the region managers with the world objects
        self.world.lock().unwrap().region_manager =
            Some(Arc::clone(&self.overworld_region_manager));
        self.world_hell.lock().unwrap().region_manager =
            Some(Arc::clone(&self.hell_region_manager));

        // If we created a new save then make a new spawn point
        if new_save {
            self.world.lock().unwrap().init_spawn();
        } else {
            self.world.lock().unwrap().spawn_point = self.save_manager.get_level_data().spawn_point;
        }
        let spawn_point = self.world.lock().unwrap().spawn_point;
        self.world_hell.lock().unwrap().spawn_point = spawn_point; // Interestingly the world spawn doesn't have the /= or *= 8 stuff

        global_logger().info("Game runtime initialized!\n");
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
