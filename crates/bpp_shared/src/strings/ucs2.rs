/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/

use crate::logger::logger::global_logger;

// Some helper functions to provide reliable
// conversion from and to UTF-8 and UCS-2,
// as it's needed by Minecraft for various things,
// such as NBT and Packet data

// Turn a UCS-2 String into a UTF-8 String
pub fn to_utf8(str: Vec<u16>) -> String {
    // UCS-2 limits all values to only be 0x0000 to 0xFFFF
    // in UTF-16 land, 0xDC00–0xDFFF have a special purpose,
    // which we will ignore
    let mut out: Vec<u8> = Vec::new();
    for i in 0..str.len() {
        let c = str[i];
        if c <= 0x7F {
            out.push(c as u8);
        } else if c <= 0x7FF {
            out.push((0xC0 | (c >> 6)) as u8);
            out.push((0x80 | (c & 0x3F)) as u8);
        } else {
            // c <= 0xFFFF
            out.push((0xE0 | (c >> 12)) as u8);
            out.push((0x80 | ((c >> 6) & 0x3F)) as u8);
            out.push((0x80 | (c & 0x3F)) as u8);
        }
    }
    String::from_utf8(out).unwrap()
}

// Decode a UTF-8 Character into a singular UCS-2 character
pub fn decode_utf8_char(s: &str, i: &mut usize) -> u32 {
    let bytes = s.as_bytes();
    let c = bytes[*i];

    // Try to parse as a one-byte character (e.g. ASCII)
    if c < 0x80 {
        let r = bytes[*i] as u32;
        *i += 1;
        return r;
    }
    // Try to parse as a two-byte character
    if (c & 0xE0) == 0xC0 {
        let cp = (((bytes[*i] & 0x1F) as u32) << 6) | ((bytes[*i + 1] & 0x3F) as u32);
        *i += 2;
        return cp;
    }
    // Try to parse as a three-byte character
    if (c & 0xF0) == 0xE0 {
        let cp = (((bytes[*i] & 0x0F) as u32) << 12)
            | (((bytes[*i + 1] & 0x3F) as u32) << 6)
            | ((bytes[*i + 2] & 0x3F) as u32);
        *i += 3;
        return cp;
    }
    // Try to parse as a four-byte character
    if (c & 0xF8) == 0xF0 {
        let cp = (((bytes[*i] & 0x07) as u32) << 18)
            | (((bytes[*i + 1] & 0x3F) as u32) << 12)
            | (((bytes[*i + 2] & 0x3F) as u32) << 6)
            | ((bytes[*i + 3] & 0x3F) as u32);
        *i += 4;
        return cp;
    }
    // All other parsing failed,
    // return Unknown/Unrecognized Character value
    *i += 1;
    0xFFFD
}

// Turn a UTF-8 String into a UCS-2 String
pub fn to_ucs2(str: String) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::new();

    let mut i = 0;
    while i < str.len() {
        let cp = decode_utf8_char(&str, &mut i);

        if cp > 0xFFFF {
            global_logger().warn("Code point not representable in UCS-2\n");
            // Spit out whatever we managed to get
            return out;
        }

        // optionally reject surrogate range too
        if cp >= 0xD800 && cp <= 0xDFFF {
            global_logger().warn("Invalid Unicode scalar\n");
            // Spit out whatever we managed to get
            return out;
        }

        out.push(cp as u16);
    }

    out
}
