/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::logger::logstream::LogStream;
use crate::logger::loglevel::{LOG_ALL, LogLevel};
use crate::logger::style::handle_formatting_codes;

// Reference for Escape Codes
// https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797

fn civil_from_unix(timestamp: i64) -> (i64, u32, u32, u32, u32, u32) {
    let days = timestamp.div_euclid(86400);
    let secs_of_day = timestamp.rem_euclid(86400);
    let hour = (secs_of_day / 3600) as u32;
    let minute = ((secs_of_day % 3600) / 60) as u32;
    let second = (secs_of_day % 60) as u32;

    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let year = if m <= 2 { y + 1 } else { y };

    (year, m, d, hour, minute, second)
}

pub struct Logger {
    log_file: Option<File>,
    log_level_text: i8,
    log_level_terminal: i8,

    pub msg: LogStream,
    pub chat: LogStream,
    pub info: LogStream,
    pub warn: LogStream,
    pub error: LogStream,
    pub debug: LogStream,
}

impl Logger {
    pub fn get_current_time_string(&self, file_format: bool) -> String {
        let now = SystemTime::now();
        let secs = now
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let (year, month, day, hour, minute, second) = civil_from_unix(secs);

        if file_format {
            format!("{year:04}-{month:02}-{day:02}-{hour:02}-{minute:02}-{second:02}")
        } else {
            #[cfg(target_os = "horizon")]
            {
                format!("{hour:02}:{minute:02}:{second:02}")
            }
            #[cfg(not(target_os = "horizon"))]
            {
                format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
            }
        }
    }

    pub fn new() -> Self {
        let mut logger = Logger {
            log_file: None,
            log_level_text: LOG_ALL,
            log_level_terminal: LOG_ALL,
            msg: LogStream::new(LogLevel::Message),
            chat: LogStream::new(LogLevel::Chat),
            info: LogStream::new(LogLevel::Info),
            warn: LogStream::new(LogLevel::Warning),
            error: LogStream::new(LogLevel::Error),
            debug: LogStream::new(LogLevel::Debug),
        };

        if logger.log_level_text != LogLevel::None as i8 {
            let log_dir = Path::new("logs");

            if !log_dir.exists() {
                fs::create_dir_all(log_dir).expect("Failed to create log directory");
            }

            let log_file_path = log_dir.join(format!("{}.log", logger.get_current_time_string(true)));

            let log_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file_path);

            logger.log_file = Some(log_file.expect("Failed to open log file"));
        }

        logger
    }

    pub fn msg(&self, message: impl AsRef<str>) {
        self.msg.push(self, message.as_ref());
    }

    pub fn chat(&self, message: impl AsRef<str>) {
        self.chat.push(self, message.as_ref());
    }

    pub fn info(&self, message: impl AsRef<str>) {
        self.info.push(self, message.as_ref());
    }

    pub fn warn(&self, message: impl AsRef<str>) {
        self.warn.push(self, message.as_ref());
    }

    pub fn error(&self, message: impl AsRef<str>) {
        self.error.push(self, message.as_ref());
    }

    pub fn debug(&self, message: impl AsRef<str>) {
        self.debug.push(self, message.as_ref());
    }

    // Log a message with the passed Level
    pub fn log(&self, message: String, level: LogLevel) {
        match level {
            LogLevel::Chat => self.chat_message(message),
            LogLevel::Info => self.log_info(message),
            LogLevel::Warning => self.warning(message),
            LogLevel::Error => self.log_error(message),
            LogLevel::Debug => self.log_debug(message),
            _ => self.message(message),
        }
    }

    // Log a chat message
    fn chat_message(&self, message: String) {
        let time = self.get_current_time_string(false);
        if self.log_level_terminal & (LogLevel::Chat as i8) != 0 {
            println!("{} {}", time, handle_formatting_codes(&message));
        }
        if self.log_level_text & (LogLevel::Chat as i8) != 0 {
            self.write_to_file(&format!("{time} {message}\n"));
        }
    }

    // Log a message without a header
    fn message(&self, message: String) {
        let time = self.get_current_time_string(false);
        if self.log_level_terminal & (LogLevel::Message as i8) != 0 {
            println!("{} {}", time, message);
        }
        if self.log_level_text & (LogLevel::Message as i8) != 0 {
            self.write_to_file(&format!("{time} {message}\n"));
        }
    }

    // Log a message with an INFO header
    fn log_info(&self, message: String) {
        let time = self.get_current_time_string(false);
        let header = "[INFO]";
        if self.log_level_terminal & (LogLevel::Info as i8) != 0 {
            println!("{time} \x1b[1;30;107m{header}\x1b[0m {message}");
        }
        if self.log_level_text & (LogLevel::Info as i8) != 0 {
            self.write_to_file(&format!("{time} {header} {message}\n"));
        }
    }

    // Log a warning
    fn warning(&self, message: String) {
        let time = self.get_current_time_string(false);
        let header = "[WARNING]";
        if self.log_level_terminal & (LogLevel::Warning as i8) != 0 {
            eprintln!("{time} \x1b[1;30;43m{header}\x1b[0;33m {message}\x1b[0m");
        }
        if self.log_level_text & (LogLevel::Warning as i8) != 0 {
            self.write_to_file(&format!("{time} {header} {message}\n"));
        }
    }

    // Log an error
    fn log_error(&self, message: String) {
        let time = self.get_current_time_string(false);
        let header = "[ERROR]";
        if self.log_level_terminal & (LogLevel::Error as i8) != 0 {
            eprintln!("{time} \x1b[1;30;101m{header} {message}\x1b[0m");
        }
        if self.log_level_text & (LogLevel::Error as i8) != 0 {
            self.write_to_file(&format!("{time} {header} {message}\n"));
        }
    }

    // Log Debug Data
    fn log_debug(&self, message: String) {
        let time = self.get_current_time_string(false);
        let header = "[DEBUG]";
        if self.log_level_terminal & (LogLevel::Debug as i8) != 0 {
            eprintln!("{time} \x1b[1;30;46m{header}\x1b[0m {message}");
        }
        if self.log_level_text & (LogLevel::Debug as i8) != 0 {
            self.write_to_file(&format!("{time} {header} {message}\n"));
        }
    }

    fn write_to_file(&self, text: &str) {
        if let Some(mut file) = self.log_file.as_ref() {
            let _ = file.write_all(text.as_bytes());
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.log_file = None;
    }
}

static LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn global_logger() -> &'static Logger {
    LOGGER.get_or_init(Logger::new)
}
