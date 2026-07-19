/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

use crate::helpers::file_handle::FileHandle;
use crate::helpers::java::java_math::hash_code;
use crate::logger::logger::global_logger;
use crate::nbt::nbt::{NBTParser, NBTwriter, TAG_COMPOUND, TAG_DOUBLE, TAG_FLOAT, Tag};
use crate::numeric_structs::Int3;

#[derive(Clone, Debug, PartialEq)]
pub struct LevelData {
    pub random_seed: i64,
    pub spawn_point: Int3,
    pub rain_time: i32,
    pub thunder_time: i32,
    pub raining: i8,
    pub time: i64,
    pub thundering: i8,
    pub version: i32,
    pub last_played: i64,
    pub level_name: String,
    pub size_on_disk: i64,
}

impl Default for LevelData {
    fn default() -> Self {
        Self {
            random_seed: 0,
            spawn_point: Int3::new(0, 0, 0),
            rain_time: 0,
            thunder_time: 0,
            raining: 0,
            time: 0,
            thundering: 0,
            version: 19132,
            last_played: 0,
            level_name: "world".to_string(),
            size_on_disk: 0,
        }
    }
}

pub struct SessionLock {
    file: Option<File>,
}

impl SessionLock {
    pub fn acquire(&mut self, path: &str) -> bool {
        let file = match OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path) {
            Ok(file) => file,
            Err(_) => return false,
        };
        if file.try_lock().is_err() {
            return false;
        }
        self.file = Some(file);
        self.write_timestamp();
        true
    }

    pub fn release(&mut self) {
        if let Some(file) = self.file.take() {
            let _ = file.unlock();
        }
    }

    fn write_timestamp(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64;
        let bytes = now.to_be_bytes();
        if let Some(file) = self.file.as_mut() {
            let _ = file.write_all(&bytes);
        }
    }
}

impl Default for SessionLock {
    fn default() -> Self {
        Self { file: None }
    }
}

impl Drop for SessionLock {
    fn drop(&mut self) {
        self.release();
    }
}

pub struct SaveManager {
    save_directory: String,
    world_file: Option<FileHandle>,
    session_lock: SessionLock,
    current_level_data: LevelData,
}

impl SaveManager {
    pub fn initialize(&mut self, save_name: &str) -> bool {
        self.save_directory = save_name.to_string();

        // Make sure we have the necessary folders
        let mut necessary_folders = 0;
        for dir in [
            format!("{save_name}/players"),
            format!("{save_name}/region"),
            format!("{save_name}/DIM-1/region"),
            format!("{save_name}/data"),
        ] {
            let existed = std::path::Path::new(&dir).is_dir();
            let _ = std::fs::create_dir_all(&dir);
            if !existed {
                necessary_folders += 1;
            }
        }
        if necessary_folders != 0 {
            global_logger().warn(format!("Failed to load {necessary_folders} necessary folder(s) for level {save_name}.\n"));
        }

        if !self.session_lock.acquire(&format!("{save_name}/session.lock")) {
            return false;
        }
        if !std::path::Path::new(&format!("{save_name}/level.dat")).exists() {
            return false;
        }
        self.world_file = Some(FileHandle::open(&format!("{save_name}/level.dat")));
        if !self.world_file.as_ref().unwrap().is_open() {
            return false;
        }
        true
    }

    pub fn load_level_data(&mut self) -> bool {
        let world_file = match self.world_file.as_mut() {
            Some(world_file) if world_file.is_open() => world_file,
            _ => return false,
        };

        let stream = world_file.get();

        // Read entire file into buffer
        if stream.seek(SeekFrom::Start(0)).is_err() {
            return false;
        }
        let mut compressed = Vec::new();
        if stream.read_to_end(&mut compressed).is_err() {
            return false;
        }

        // Decompress gzip
        let mut decoder = GzDecoder::new(compressed.as_slice());
        let mut raw = Vec::new();
        if decoder.read_to_end(&mut raw).is_err() {
            return false;
        }

        // Parse NBT
        let parser = NBTParser::new(&raw, raw.len() as i64);
        let data = parser.root.get("Data");

        self.current_level_data.random_seed = data.get("RandomSeed").get_long();
        self.current_level_data.spawn_point.x = data.get("SpawnX").get_int();
        self.current_level_data.spawn_point.y = data.get("SpawnY").get_int();
        self.current_level_data.spawn_point.z = data.get("SpawnZ").get_int();
        self.current_level_data.rain_time = data.get("rainTime").get_int();
        self.current_level_data.thunder_time = data.get("thunderTime").get_int();
        self.current_level_data.raining = data.get("raining").get_byte();
        self.current_level_data.time = data.get("Time").get_long();
        self.current_level_data.thundering = data.get("thundering").get_byte();
        self.current_level_data.version = data.get("version").get_int();
        self.current_level_data.last_played = data.get("LastPlayed").get_long();
        self.current_level_data.level_name = data.get("LevelName").get_string().to_string();
        self.current_level_data.size_on_disk = data.get("SizeOnDisk").get_long();

        true
    }

    pub fn get_level_data(&self) -> &LevelData {
        &self.current_level_data
    }

    pub fn create_new_world(&mut self, data: LevelData) -> bool {
        let _ = std::fs::create_dir_all(format!("{}/players", self.save_directory));
        let _ = std::fs::create_dir_all(format!("{}/region", self.save_directory));
        let _ = std::fs::create_dir_all(format!("{}/DIM-1/region", self.save_directory));
        let _ = std::fs::create_dir_all(format!("{}/data", self.save_directory));
        self.save_level_file(&data)
    }

    pub fn save_level_file(&mut self, level_data: &LevelData) -> bool {
        // Back up existing level.dat if present
        let level_dat_path = format!("{}/level.dat", self.save_directory);
        if std::path::Path::new(&level_dat_path).exists() {
            let _ = std::fs::copy(&level_dat_path, format!("{}/level.dat_old", self.save_directory));
        }

        let mut data_compound = HashMap::new();

        data_compound
            .insert("RandomSeed".to_string(), Tag::Long { name: "RandomSeed".to_string(), long_value: level_data.random_seed });
        data_compound
            .insert("SpawnX".to_string(), Tag::Int { name: "SpawnX".to_string(), int_value: level_data.spawn_point.x });
        data_compound
            .insert("SpawnY".to_string(), Tag::Int { name: "SpawnY".to_string(), int_value: level_data.spawn_point.y });
        data_compound
            .insert("SpawnZ".to_string(), Tag::Int { name: "SpawnZ".to_string(), int_value: level_data.spawn_point.z });
        data_compound
            .insert("rainTime".to_string(), Tag::Int { name: "rainTime".to_string(), int_value: level_data.rain_time });
        data_compound.insert(
            "thunderTime".to_string(),
            Tag::Int { name: "thunderTime".to_string(), int_value: level_data.thunder_time },
        );
        data_compound
            .insert("raining".to_string(), Tag::Byte { name: "raining".to_string(), byte_value: level_data.raining });
        data_compound.insert("Time".to_string(), Tag::Long { name: "Time".to_string(), long_value: level_data.time });
        data_compound.insert(
            "thundering".to_string(),
            Tag::Byte { name: "thundering".to_string(), byte_value: level_data.thundering },
        );
        data_compound
            .insert("version".to_string(), Tag::Int { name: "version".to_string(), int_value: level_data.version });
        data_compound.insert(
            "LastPlayed".to_string(),
            Tag::Long { name: "LastPlayed".to_string(), long_value: level_data.last_played },
        );
        data_compound.insert(
            "LevelName".to_string(),
            Tag::String { name: "LevelName".to_string(), string_value: level_data.level_name.clone() },
        );
        data_compound.insert(
            "SizeOnDisk".to_string(),
            Tag::Long { name: "SizeOnDisk".to_string(), long_value: level_data.size_on_disk },
        );

        let data_tag = Tag::Compound { name: "Data".to_string(), compound: data_compound };
        let mut root_compound = HashMap::new();
        root_compound.insert("Data".to_string(), data_tag);
        let root = Tag::Compound { name: String::new(), compound: root_compound };

        let mut raw = Vec::new();
        NBTwriter::new(&mut raw, &root);

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        if encoder.write_all(&raw).is_err() {
            return false;
        }
        let compressed = match encoder.finish() {
            Ok(compressed) => compressed,
            Err(_) => return false,
        };
        if compressed.is_empty() {
            return false;
        }

        let mut file = match File::create(&level_dat_path) {
            Ok(file) => file,
            Err(_) => return false,
        };
        if file.write_all(&compressed).is_err() {
            return false;
        }

        // Reopen as FileHandle for future use
        self.world_file = Some(FileHandle::open(&level_dat_path));
        true
    }

    pub fn seed_from_string(&self, input: &str) -> i64 {
        // If it's a plain number, use it directly
        if let Ok(numeric) = input.parse::<i64>() {
            return numeric;
        }

        // Otherwise hash it
        i64::from(hash_code(input))
    }

    fn player_data_path(&self, player_name: &str) -> Option<String> {
        if player_name.is_empty()
            || player_name.len() > 16
            || !player_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return None;
        }
        Some(format!("{}/players/{player_name}.dat", self.save_directory))
    }

    pub fn get_player_nbt(&mut self, player_name: &str) -> Tag {
        // return by value
        let player_path = match self.player_data_path(player_name) {
            Some(path) => path,
            None => return self.get_new_player_nbt(),
        };

        if !std::path::Path::new(&player_path).exists() {
            let fresh = self.get_new_player_nbt();
            self.save_player_nbt(player_name, &fresh);
            return fresh;
        }

        // Load existing player file
        let mut file = match File::open(&player_path) {
            Ok(file) => file,
            Err(_) => return self.get_new_player_nbt(),
        };

        let mut compressed = Vec::new();
        if file.read_to_end(&mut compressed).is_err() {
            return self.get_new_player_nbt();
        }

        let mut decoder = GzDecoder::new(compressed.as_slice());
        let mut raw = Vec::new();
        if decoder.read_to_end(&mut raw).is_err() {
            return self.get_new_player_nbt();
        }

        let parser = NBTParser::new(&raw, raw.len() as i64);
        parser.root
    }

    pub fn get_new_player_nbt(&self) -> Tag {
        let mut motion = Tag::List { name: "Motion".to_string(), list_type: TAG_DOUBLE, list: Vec::new() };
        let sleep_timer = Tag::Short { name: "SleepTimer".to_string(), short_value: 0 };
        let health = Tag::Short { name: "Health".to_string(), short_value: 20 };
        let air = Tag::Short { name: "Air".to_string(), short_value: 300 };
        let on_ground = Tag::Byte { name: "OnGround".to_string(), byte_value: 0 };
        let dimension = Tag::Int { name: "Dimension".to_string(), int_value: 0 };
        let mut rotation = Tag::List { name: "Rotation".to_string(), list_type: TAG_FLOAT, list: Vec::new() };
        let fall_distance = Tag::Float { name: "FallDistance".to_string(), float_value: 0.0 };
        let sleeping = Tag::Byte { name: "Sleeping".to_string(), byte_value: 0 };
        let mut pos = Tag::List { name: "Pos".to_string(), list_type: TAG_DOUBLE, list: Vec::new() };
        let death_time = Tag::Short { name: "DeathTime".to_string(), short_value: 0 };
        let fire = Tag::Short { name: "Fire".to_string(), short_value: -20 };
        let hurt_time = Tag::Short { name: "HurtTime".to_string(), short_value: 0 };
        let attack_time = Tag::Short { name: "AttackTime".to_string(), short_value: 0 };
        let inventory = Tag::List { name: "Inventory".to_string(), list_type: TAG_COMPOUND, list: Vec::new() };

        // Initialize our position with a default
        let pos_x = Tag::Double { name: String::new(), double_value: -1.0 };
        let pos_y = Tag::Double { name: String::new(), double_value: -1000000.0 };
        let pos_z = Tag::Double { name: String::new(), double_value: -1.0 };
        if let Tag::List { list, .. } = &mut pos {
            list.push(pos_x);
            list.push(pos_y);
            list.push(pos_z);
        }

        let rot_x = Tag::Float { name: String::new(), float_value: 0.0 };
        let rot_y = Tag::Float { name: String::new(), float_value: 0.0 };
        if let Tag::List { list, .. } = &mut rotation {
            list.push(rot_x);
            list.push(rot_y);
        }

        // Initialize our position with a default
        let mov_x = Tag::Double { name: String::new(), double_value: 0.0 };
        let mov_y = Tag::Double { name: String::new(), double_value: 0.0 };
        let mov_z = Tag::Double { name: String::new(), double_value: 0.0 };
        if let Tag::List { list, .. } = &mut motion {
            list.push(mov_x);
            list.push(mov_y);
            list.push(mov_z);
        }

        let mut compound = HashMap::new();
        compound.insert("Motion".to_string(), motion);
        compound.insert("SleepTimer".to_string(), sleep_timer);
        compound.insert("Health".to_string(), health);
        compound.insert("Air".to_string(), air);
        compound.insert("OnGround".to_string(), on_ground);
        compound.insert("Dimension".to_string(), dimension);
        compound.insert("Rotation".to_string(), rotation);
        compound.insert("FallDistance".to_string(), fall_distance);
        compound.insert("Sleeping".to_string(), sleeping);
        compound.insert("Pos".to_string(), pos);
        compound.insert("DeathTime".to_string(), death_time);
        compound.insert("Fire".to_string(), fire);
        compound.insert("HurtTime".to_string(), hurt_time);
        compound.insert("AttackTime".to_string(), attack_time);
        compound.insert("Inventory".to_string(), inventory);
        Tag::Compound { name: String::new(), compound }
    }

    pub fn save_player_nbt(&self, player_name: &str, player_data: &Tag) -> bool {
        let mut raw = Vec::new();
        NBTwriter::new(&mut raw, player_data);

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        if encoder.write_all(&raw).is_err() {
            return false;
        }
        let compressed = match encoder.finish() {
            Ok(compressed) => compressed,
            Err(_) => return false,
        };
        if compressed.is_empty() {
            return false;
        }

        let final_path = match self.player_data_path(player_name) {
            Some(path) => path,
            None => return false,
        };
        let tmp_path = format!("{final_path}.tmp");

        let mut file = match File::create(&tmp_path) {
            Ok(file) => file,
            Err(_) => return false,
        };
        if file.write_all(&compressed).is_err() {
            return false;
        }
        if file.flush().is_err() {
            return false;
        }
        drop(file);

        std::fs::rename(tmp_path, final_path).is_ok()
    }

    pub fn release(&mut self) {
        self.session_lock.release();
    }
}

impl Default for SaveManager {
    fn default() -> Self {
        Self {
            save_directory: String::new(),
            world_file: None,
            session_lock: SessionLock::default(),
            current_level_data: LevelData::default(),
        }
    }
}

impl Drop for SaveManager {
    fn drop(&mut self) {
        self.release();
    }
}
