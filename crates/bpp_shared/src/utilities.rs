/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::fs;
use std::path::Path;

use crate::logger::logger::global_logger;
#[cfg(any())]
use crate::numeric_structs::{Int2, Int32_2};
#[cfg(any())]
use crate::world::storage::region_manager::RegionManager;
#[cfg(any())]
use crate::world::storage::save_manager::SaveManager;

// Creates a temp directory and deletes it if it already exists. Returns false on failure.
pub fn recreate_temp_dir(dir: &Path) -> bool {
    match fs::remove_dir_all(dir) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            global_logger().error(format!("Failed to remove directory: {e}\n"));
            return false;
        }
    }
    if let Err(e) = fs::create_dir_all(dir) {
        global_logger().error(format!("Failed to create directory: {e}\n"));
        return false;
    }
    true
}

// Cleans up level data since mcr files can become bloated
#[cfg(any())]
pub fn clean_level(rel_path: String) -> bool {
    let mut save_manager = SaveManager::default();
    let mut region_manager = RegionManager::default();
    let mut out_region_manager = RegionManager::default();

    if !save_manager.initialize(&rel_path) {
        global_logger().error("Failed to initialize save manager for cleaning!\n");
        return false;
    }

    // Overworld first
    {
        if !region_manager.initialize(format!("{rel_path}/region")) {
            global_logger().error("Failed to initialize overworld region manager for cleaning! (Skipping)\n");
        }

        if !recreate_temp_dir(Path::new(&format!("{rel_path}/tempRegion"))) {
            global_logger().error("Failed to recreate temp directory!\n");
            return false;
        }

        if !out_region_manager.initialize(format!("{rel_path}/tempRegion")) {
            global_logger().error("Failed to initialize output region manager for cleaning!\n");
            return false;
        }

        // Check what regions exist
        global_logger().info("Scanning for regions to clean...\n");
        let mut region_coords: Vec<Int32_2> = Vec::new();
        for entry in fs::read_dir(format!("{rel_path}/region")).unwrap() {
            let entry = entry.unwrap();
            let region_path = entry.path();
            // Is this a valid region file?
            if region_path.extension().is_some_and(|ext| ext == "mcr") {
                // Get the coordinate pair for this region
                let filename = entry.file_name().to_string_lossy().to_string();

                // Does this have a valid name?
                if let Some(rest) = filename.strip_prefix("r.").and_then(|s| s.strip_suffix(".mcr")) {
                    let mut parts = rest.splitn(2, '.');
                    if let (Some(rx), Some(rz)) = (parts.next(), parts.next())
                        && let (Ok(rx), Ok(rz)) = (rx.parse::<i32>(), rz.parse::<i32>())
                    {
                        region_coords.push(Int32_2::new(rx, rz));
                    }
                }
            }
        }

        global_logger().info(format!("Found {} regions to clean.\n", region_coords.len()));

        // Go through each region
        let mut chunks_cleaned = 0;
        let mut region_cnt = 0;
        for region_coord in &region_coords {
            let region = region_manager.load_region(*region_coord);
            out_region_manager.create_region(*region_coord);
            let new_region = out_region_manager.load_region(*region_coord);
            for cx in 0..32 {
                for cz in 0..32 {
                    // Load the chunk if it exists
                    if region.lock().unwrap().chunk_exists(Int2::new(cx, cz)) {
                        chunks_cleaned += 1;
                        let chunk = region
                            .lock()
                            .unwrap()
                            .get_chunk(Int32_2::new(region_coord.x * 32 + cx, *region_coord.z() * 32 + cz));
                        // Save to a new region file
                        new_region.lock().unwrap().add_chunk(chunk, 0, None);
                    }
                }
            }
            region_cnt += 1;
            global_logger().info(format!(
                "Cleaned region (Overworld):{region_coord}: {region_cnt} / {}\n",
                region_coords.len()
            ));
        }

        global_logger().info(format!(
            "Cleaned {chunks_cleaned} chunks across {} regions.\n",
            region_coords.len()
        ));

        // Delete original, copy from temp, delete temp
        region_manager.release();
        out_region_manager.release();
        let _ = fs::remove_dir_all(format!("{rel_path}/region"));
        fs::copy(format!("{rel_path}/tempRegion"), format!("{rel_path}/region")).unwrap();
        let _ = fs::remove_dir_all(format!("{rel_path}/tempRegion"));
    }

    // Nether
    {
        if !region_manager.initialize(format!("{rel_path}/DIM-1/region")) {
            global_logger().error("Failed to initialize nether region manager for cleaning! (Skipping)\n");
        }

        if !recreate_temp_dir(Path::new(&format!("{rel_path}/tempRegionNether"))) {
            global_logger().error("Failed to recreate temp directory!\n");
            return false;
        }

        if !out_region_manager.initialize(format!("{rel_path}/tempRegionNether")) {
            global_logger().error("Failed to initialize output region manager for cleaning!\n");
            return false;
        }

        // Check what regions exist
        global_logger().info("Scanning for regions to clean...\n");
        let mut region_coords: Vec<Int32_2> = Vec::new();
        for entry in fs::read_dir(format!("{rel_path}/DIM-1/region")).unwrap() {
            let entry = entry.unwrap();
            let region_path = entry.path();
            // Is this a valid region file?
            if region_path.extension().is_some_and(|ext| ext == "mcr") {
                // Get the coordinate pair for this region
                let filename = entry.file_name().to_string_lossy().to_string();

                // Does this have a valid name?
                if let Some(rest) = filename.strip_prefix("r.").and_then(|s| s.strip_suffix(".mcr")) {
                    let mut parts = rest.splitn(2, '.');
                    if let (Some(rx), Some(rz)) = (parts.next(), parts.next())
                        && let (Ok(rx), Ok(rz)) = (rx.parse::<i32>(), rz.parse::<i32>())
                    {
                        region_coords.push(Int32_2::new(rx, rz));
                    }
                }
            }
        }

        global_logger().info(format!("Found {} regions to clean.\n", region_coords.len()));

        // Go through each region
        let mut chunks_cleaned = 0;
        let mut region_cnt = 0;
        for region_coord in &region_coords {
            let region = region_manager.load_region(*region_coord);
            out_region_manager.create_region(*region_coord);
            let new_region = out_region_manager.load_region(*region_coord);
            for cx in 0..32 {
                for cz in 0..32 {
                    // Load the chunk if it exists
                    if region.lock().unwrap().chunk_exists(Int2::new(cx, cz)) {
                        chunks_cleaned += 1;
                        let chunk = region
                            .lock()
                            .unwrap()
                            .get_chunk(Int32_2::new(region_coord.x * 32 + cx, *region_coord.z() * 32 + cz));
                        // Save to a new region file
                        new_region.lock().unwrap().add_chunk(chunk, 0, None);
                    }
                }
            }
            region_cnt += 1;
            global_logger().info(format!(
                "Cleaned region (Nether):{region_coord}: {region_cnt} / {}\n",
                region_coords.len()
            ));
        }

        global_logger().info(format!(
            "Cleaned {chunks_cleaned} chunks across {} regions.\n",
            region_coords.len()
        ));

        // Delete original, copy from temp, delete temp
        region_manager.release();
        out_region_manager.release();
        let _ = fs::remove_dir_all(format!("{rel_path}/DIM-1/region"));
        fs::copy(format!("{rel_path}/tempRegionNether"), format!("{rel_path}/DIM-1/region")).unwrap();
        let _ = fs::remove_dir_all(format!("{rel_path}/tempRegionNether"));
    }
    true // yay we did it
}
