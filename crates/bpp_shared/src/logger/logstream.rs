/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
use std::sync::Mutex;

use crate::logger::logger::Logger;
use crate::logger::loglevel::LogLevel;

pub struct LogStream {
    level: LogLevel,
    buffer: Mutex<String>,
}

impl LogStream {
    pub fn new(level: LogLevel) -> Self {
        LogStream {
            level,
            buffer: Mutex::new(String::new()),
        }
    }

    pub fn push(&self, logger: &Logger, value: &str) {
        if let Some(stripped) = value.strip_suffix('\n') {
            self.buffer.lock().unwrap().push_str(stripped);
            self.flush(logger);
        } else {
            self.buffer.lock().unwrap().push_str(value);
        }
    }

    pub fn flush(&self, logger: &Logger) {
        let mut buffer = self.buffer.lock().unwrap();
        logger.log(buffer.clone(), self.level);
        buffer.clear();
    }
}
