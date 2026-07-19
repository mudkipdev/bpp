/*
 * Copyright (c) 2025, MINA <github.com/9mina>
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::sync::RwLock;

use crate::logger::logger::global_logger;

type ConfType = HashMap<String, String>;

pub struct Config {
    properties_mutex: RwLock<ConfType>,

    path: RwLock<String>,
}

impl Config {
    pub fn new(path: &str) -> Config {
        Config {
            properties_mutex: RwLock::new(ConfType::new()),
            path: RwLock::new(path.to_string()),
        }
    }

    // get the value at key or a the default mapped_type if key doesn't exist
    pub fn get(&self, key: &str) -> String {
        let read_lock = self.properties_mutex.read().unwrap();
        read_lock.get(key).cloned().unwrap_or_default()
    }

    pub fn get_as_string(&self, key: &str) -> String {
        self.get(key)
    }

    // get the value at key as number
    pub fn get_as_number<T>(&self, key: &str) -> T
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Debug,
    {
        self.get(key).parse::<T>().unwrap()
    }

    // get the value at key as boolean
    pub fn get_as_boolean(&self, key: &str) -> bool {
        let val = self.get(key);
        if val == "true" || val == "1" {
            return true;
        }
        // All other cases default to false
        /*
        if val == "false" || val == "0" {
            return false;
        }
        */
        false
    }

    // set value at key.
    // will create key if it doesn't exist.
    pub fn set(&self, key: &str, value: &str) {
        let mut write_lock = self.properties_mutex.write().unwrap();
        write_lock.insert(key.to_string(), value.to_string());
    }

    // overwrite the properties in memory
    pub fn overwrite(&self, config: ConfType) {
        let mut write_lock = self.properties_mutex.write().unwrap();
        *write_lock = config;
    }

    // read a properties file from disk into memory.
    // returns false on error.
    pub fn load_from_disk(&self) -> bool {
        let path = self.path.read().unwrap().clone();
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(_) => {
                global_logger().warn("**** Error opening properties file (load). Attempting to create new file...\n");
                return false;
            }
        };

        let mut write_lock = self.properties_mutex.write().unwrap();

        write_lock.clear();

        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = match line {
                Ok(line) => line,
                Err(_) => break,
            };

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let delimiter_pos = match line.find('=') {
                Some(pos) => pos,
                None => {
                    global_logger().error(format!("**** Invalid line in properties file: {line}\n"));
                    continue;
                }
            };

            let mut key = line[..delimiter_pos].to_string();
            let value = line[delimiter_pos + 1..].to_string();

            // Trim whitespace (optional)
            key.truncate(key.trim_end_matches([' ', '\t', '\n', '\r', '\x0b', '\x0c']).len());
            let value = value.trim_start_matches([' ', '\t', '\n', '\r', '\x0b', '\x0c']).to_string();

            write_lock.insert(key, value);
        }
        true
    }

    // save the properties in memory to disk.
    // returns false on error.
    pub fn save_to_disk(&self) -> bool {
        let path = self.path.read().unwrap().clone();
        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(_) => {
                global_logger().error("**** Error opening properties file (save). \n");
                return false;
            }
        };

        let read_lock = self.properties_mutex.read().unwrap();
        for (key, value) in read_lock.iter() {
            if let Err(e) = writeln!(file, "{key}={value}") {
                global_logger().error(format!("**** Error while writing properties file: {e}\n"));
                return false;
            }
        }

        global_logger().info("Properties file saved successfully.\n");
        true
    }

    // set a new path to the properties file
    pub fn set_path(&self, path: &str) {
        *self.path.write().unwrap() = path.to_string();
    }

    // get the current properties path
    pub fn get_path(&self) -> String {
        self.path.read().unwrap().clone()
    }
}
