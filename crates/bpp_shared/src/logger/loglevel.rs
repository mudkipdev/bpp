/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum LogLevel {
    None = 0,
    Chat = 1,
    Message = 2,
    Info = 4,
    Warning = 8,
    Error = 16,
    Debug = 32,
}

pub const LOG_ALL: i8 = LogLevel::Debug as i8
    | LogLevel::Error as i8
    | LogLevel::Warning as i8
    | LogLevel::Info as i8
    | LogLevel::Message as i8
    | LogLevel::Chat as i8;
