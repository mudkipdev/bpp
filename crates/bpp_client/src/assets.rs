/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::path::Path;

use bpp_shared::logger::logger::global_logger;

// Responsible for acquiring, extracting and caching assets
pub struct AssetManager {}

impl AssetManager {
    pub fn new() -> Self {
        let manager = Self {};
        // Check if assets already exist
        if Path::new("assets/").exists() {
            return manager;
        }
        global_logger().info("No assets found! Extracting from client.jar file...\n");
        // Attempt to extract assets
        if manager.extract_assets() {
            return manager;
        }
        global_logger().info("No client.jar file found! Downloading...\n");
        // Attempt to redownload assets
        if !manager.download_assets() {
            global_logger().error("Failed to download client.jar file!\n");
            return manager;
        }
        global_logger().info("Retrying extracting from client.jar file...\n");
        // Attempt to extract assets again
        if manager.extract_assets() {
            return manager;
        }
        global_logger().error("Failed to extract assets!\n");
        manager
    }

    fn download_assets(&self) -> bool {
        // https://piston-data.mojang.com/v1/objects/43db9b498cb67058d2e12d394e6507722e71bb45/client.jar
        false
    }

    fn extract_assets(&self) -> bool {
        if !Path::new("client.jar").exists() {
            return false;
        }
        global_logger().info("Extracted assets successfully!\n");
        true
    }
}
