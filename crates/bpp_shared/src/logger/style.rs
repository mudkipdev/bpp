/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
pub const STYLE_BOLD: &str = "\x1b[1m";
pub const STYLE_ITALIC: &str = "\x1b[3m";
pub const STYLE_UNDERLINE: &str = "\x1b[4m";
pub const STYLE_STRIKETHROUGH: &str = "\x1b[9m";

pub const STYLE_FOREGROUND_BLACK: &str = "\x1b[30m";
pub const STYLE_FOREGROUND_RED: &str = "\x1b[31m";
pub const STYLE_FOREGROUND_GREEN: &str = "\x1b[32m";
pub const STYLE_FOREGROUND_YELLOW: &str = "\x1b[33m";
pub const STYLE_FOREGROUND_BLUE: &str = "\x1b[34m";
pub const STYLE_FOREGROUND_PURPLE: &str = "\x1b[35m";
pub const STYLE_FOREGROUND_CYAN: &str = "\x1b[36m";
pub const STYLE_FOREGROUND_WHITE: &str = "\x1b[37m";

pub const STYLE_BACKGROUND_BLACK: &str = "\x1b[40m";
pub const STYLE_BACKGROUND_RED: &str = "\x1b[41m";
pub const STYLE_BACKGROUND_GREEN: &str = "\x1b[42m";
pub const STYLE_BACKGROUND_YELLOW: &str = "\x1b[43m";
pub const STYLE_BACKGROUND_BLUE: &str = "\x1b[44m";
pub const STYLE_BACKGROUND_PURPLE: &str = "\x1b[45m";
pub const STYLE_BACKGROUND_CYAN: &str = "\x1b[46m";
pub const STYLE_BACKGROUND_WHITE: &str = "\x1b[47m";

pub const STYLE_RESET: &str = "\x1b[0m";

// Translate Minecraft-style colors into ASCII Escape sequence colors
pub fn format_to_style(format: i8) -> String {
    match format as u8 as char {
        // Colors
        '0' => "\x1b[30m".to_string(),
        '1' => "\x1b[34m".to_string(),
        '2' => "\x1b[32m".to_string(),
        '3' => "\x1b[36m".to_string(),
        '4' => "\x1b[31m".to_string(),
        '5' => "\x1b[35m".to_string(),
        '6' => "\x1b[33m".to_string(),
        '7' => "\x1b[37m".to_string(),
        '8' => "\x1b[90m".to_string(),
        '9' => "\x1b[94m".to_string(),
        'a' => "\x1b[92m".to_string(),
        'b' => "\x1b[96m".to_string(),
        'c' => "\x1b[91m".to_string(),
        'd' => "\x1b[95m".to_string(),
        'e' => "\x1b[93m".to_string(),
        'f' => "\x1b[97m".to_string(),
        // Bold
        'l' => "\x1b[1m".to_string(),
        // Strikethrough
        'm' => "\x1b[9m".to_string(),
        // Underlined
        'n' => "\x1b[4m".to_string(),
        // Italic
        'o' => "\x1b[3m".to_string(),
        // Obfuscated
        'k' => "\x1b[37;105m".to_string(),
        // Reset
        _ => STYLE_RESET.to_string(),
    }
}

// Translate the passed string with Minecraft-style formatters into ASCII Escape sequence colors
pub fn handle_formatting_codes(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output: Vec<u8> = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        // Check if first character is §
        if bytes[i] == 0xC2 && bytes.get(i + 1).copied().unwrap_or(0) == 0xA7 && i + 2 < bytes.len() {
            output.extend_from_slice(format_to_style(bytes[i + 2] as i8).as_bytes()); // Replace § and the next character
            i += 1; // Skip the next character
            i += 1; // Skip the next character
        } else {
            output.push(bytes[i]);
        }
        i += 1;
    }
    output.extend_from_slice(STYLE_RESET.as_bytes());
    String::from_utf8(output).unwrap()
}
