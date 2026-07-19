/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::fs::{File, OpenOptions};

#[derive(Default)]
pub struct FileHandle {
    stream: Option<File>,
}

impl FileHandle {
    pub fn open(path: &str) -> Self {
        match OpenOptions::new().read(true).write(true).open(path) {
            Ok(file) => Self { stream: Some(file) },
            Err(_) => panic!("Failed to open file: {path}"),
        }
    }

    pub fn get(&mut self) -> &mut File {
        self.stream.as_mut().expect("FileHandle is not open")
    }

    pub fn is_open(&self) -> bool {
        self.stream.is_some()
    }
}
