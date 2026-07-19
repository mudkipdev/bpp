/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TagType(pub u8);

pub const TAG_END: TagType = TagType(0);
pub const TAG_BYTE: TagType = TagType(1);
pub const TAG_SHORT: TagType = TagType(2);
pub const TAG_INT: TagType = TagType(3);
pub const TAG_LONG: TagType = TagType(4);
pub const TAG_FLOAT: TagType = TagType(5);
pub const TAG_DOUBLE: TagType = TagType(6);
pub const TAG_BYTEARRAY: TagType = TagType(7);
pub const TAG_STRING: TagType = TagType(8);
pub const TAG_LIST: TagType = TagType(9);
pub const TAG_COMPOUND: TagType = TagType(10);
pub const TAG_INTARRAY: TagType = TagType(11);

#[derive(Clone, Debug)]
pub enum Tag {
    End,

    // Leaf values
    Byte { name: String, byte_value: i8 },
    Short { name: String, short_value: i16 },
    Int { name: String, int_value: i32 },
    Long { name: String, long_value: i64 },
    Float { name: String, float_value: f32 },
    Double { name: String, double_value: f64 },
    String { name: String, string_value: String },
    ByteArray { name: String, byte_array: Vec<i8> },
    IntArray { name: String, int_array: Vec<i32> },

    // Container values
    List { name: String, list_type: TagType, list: Vec<Tag> }, // list_type is the element type for TAG_LIST
    Compound { name: String, compound: HashMap<String, Tag> },
}

impl Tag {
    pub fn r#type(&self) -> TagType {
        match self {
            Tag::End => TAG_END,
            Tag::Byte { .. } => TAG_BYTE,
            Tag::Short { .. } => TAG_SHORT,
            Tag::Int { .. } => TAG_INT,
            Tag::Long { .. } => TAG_LONG,
            Tag::Float { .. } => TAG_FLOAT,
            Tag::Double { .. } => TAG_DOUBLE,
            Tag::String { .. } => TAG_STRING,
            Tag::ByteArray { .. } => TAG_BYTEARRAY,
            Tag::IntArray { .. } => TAG_INTARRAY,
            Tag::List { .. } => TAG_LIST,
            Tag::Compound { .. } => TAG_COMPOUND,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Tag::End => "",
            Tag::Byte { name, .. }
            | Tag::Short { name, .. }
            | Tag::Int { name, .. }
            | Tag::Long { name, .. }
            | Tag::Float { name, .. }
            | Tag::Double { name, .. }
            | Tag::String { name, .. }
            | Tag::ByteArray { name, .. }
            | Tag::IntArray { name, .. }
            | Tag::List { name, .. }
            | Tag::Compound { name, .. } => name,
        }
    }

    // Typed getters; panic if wrong type
    pub fn get_byte(&self) -> i8 {
        self.expect(TAG_BYTE);
        match self {
            Tag::Byte { byte_value, .. } => *byte_value,
            _ => unreachable!(),
        }
    }

    pub fn get_short(&self) -> i16 {
        self.expect(TAG_SHORT);
        match self {
            Tag::Short { short_value, .. } => *short_value,
            _ => unreachable!(),
        }
    }

    pub fn get_int(&self) -> i32 {
        self.expect(TAG_INT);
        match self {
            Tag::Int { int_value, .. } => *int_value,
            _ => unreachable!(),
        }
    }

    pub fn get_long(&self) -> i64 {
        self.expect(TAG_LONG);
        match self {
            Tag::Long { long_value, .. } => *long_value,
            _ => unreachable!(),
        }
    }

    pub fn get_float(&self) -> f32 {
        self.expect(TAG_FLOAT);
        match self {
            Tag::Float { float_value, .. } => *float_value,
            _ => unreachable!(),
        }
    }

    pub fn get_double(&self) -> f64 {
        self.expect(TAG_DOUBLE);
        match self {
            Tag::Double { double_value, .. } => *double_value,
            _ => unreachable!(),
        }
    }

    pub fn get_byte_array(&self) -> &[i8] {
        self.expect(TAG_BYTEARRAY);
        match self {
            Tag::ByteArray { byte_array, .. } => byte_array,
            _ => unreachable!(),
        }
    }

    pub fn get_int_array(&self) -> &[i32] {
        self.expect(TAG_INTARRAY);
        match self {
            Tag::IntArray { int_array, .. } => int_array,
            _ => unreachable!(),
        }
    }

    pub fn get_string(&self) -> &str {
        self.expect(TAG_STRING);
        match self {
            Tag::String { string_value, .. } => string_value,
            _ => unreachable!(),
        }
    }

    pub fn get_list(&self) -> &[Tag] {
        self.expect(TAG_LIST);
        match self {
            Tag::List { list, .. } => list,
            _ => unreachable!(),
        }
    }

    pub fn get_compound(&self) -> &HashMap<String, Tag> {
        self.expect(TAG_COMPOUND);
        match self {
            Tag::Compound { compound, .. } => compound,
            _ => unreachable!(),
        }
    }

    // Compound lookup helpers
    pub fn has(&self, key: &str) -> bool {
        match self {
            Tag::Compound { compound, .. } => compound.contains_key(key),
            _ => false,
        }
    }

    pub fn get(&self, key: &str) -> &Tag {
        match self {
            Tag::Compound { compound, .. } => match compound.get(key) {
                Some(tag) => tag,
                None => panic!("NBT key not found: {key}"),
            },
            _ => panic!("NBT key not found: {key}"),
        }
    }

    fn expect(&self, t: TagType) {
        if self.r#type() != t {
            panic!("NBT type mismatch");
        }
    }
}

pub struct NBTwriter;

impl NBTwriter {
    pub fn new(out: &mut Vec<u8>, root: &Tag) -> Self {
        // root should be a TAG_Compound with whatever name you want (usually "")
        // write_tag handles type byte + name + payload + TAG_END automatically
        let writer = NBTwriter;
        writer.write_tag(out, root, false);
        writer
    }

    pub fn write_tag(&self, out: &mut Vec<u8>, tag: &Tag, payload: bool) -> i64 {
        if !payload {
            out.push(tag.r#type().0);
        }
        if !payload && tag.r#type() != TAG_END {
            self.write_string(out, tag.name());
        }

        match tag {
            Tag::End => {}
            Tag::Byte { byte_value, .. } => out.push(*byte_value as u8),
            Tag::Short { short_value, .. } => self.write_i16(out, *short_value),
            Tag::Int { int_value, .. } => self.write_i32(out, *int_value),
            Tag::Long { long_value, .. } => self.write_i64(out, *long_value),
            Tag::Float { float_value, .. } => self.write_f32(out, *float_value),
            Tag::Double { double_value, .. } => self.write_f64(out, *double_value),
            Tag::String { string_value, .. } => self.write_string(out, string_value),

            Tag::ByteArray { byte_array, .. } => {
                self.write_i32(out, byte_array.len() as i32);
                for b in byte_array {
                    out.push(*b as u8);
                }
            }

            Tag::IntArray { int_array, .. } => {
                self.write_i32(out, int_array.len() as i32);
                for b in int_array {
                    self.write_i32(out, *b);
                }
            }

            Tag::List { list_type, list, .. } => {
                self.write_i8(out, list_type.0 as i8);
                self.write_i32(out, list.len() as i32);
                for element in list {
                    self.write_tag(out, element, true);
                }
            }

            Tag::Compound { compound, .. } => {
                for (_key, child) in compound {
                    self.write_tag(out, child, false);
                }
                // TAG_END terminates the compound
                out.push(TAG_END.0);
            }
        }

        0
    }

    // Write helpers
    pub fn write_i32(&self, out: &mut Vec<u8>, v: i32) {
        let u = v as u32;
        out.push(((u >> 24) & 0xFF) as u8);
        out.push(((u >> 16) & 0xFF) as u8);
        out.push(((u >> 8) & 0xFF) as u8);
        out.push((u & 0xFF) as u8);
    }

    pub fn write_i64(&self, out: &mut Vec<u8>, v: i64) {
        self.write_i32(out, (((v as u64) >> 32) & 0xFFFFFFFF) as i32);
        self.write_i32(out, ((v as u64) & 0xFFFFFFFF) as i32);
    }

    pub fn write_i16(&self, out: &mut Vec<u8>, v: i16) {
        let u = v as u16;
        out.push(((u >> 8) & 0xFF) as u8);
        out.push((u & 0xFF) as u8);
    }

    pub fn write_i8(&self, out: &mut Vec<u8>, v: i8) {
        out.push(v as u8);
    }

    pub fn write_f32(&self, out: &mut Vec<u8>, v: f32) {
        let raw = v.to_bits();
        self.write_i32(out, raw as i32);
    }

    pub fn write_f64(&self, out: &mut Vec<u8>, v: f64) {
        let raw = v.to_bits();
        self.write_i64(out, raw as i64);
    }

    pub fn write_string(&self, out: &mut Vec<u8>, s: &str) {
        self.write_i16(out, s.len() as i16);
        out.extend_from_slice(s.as_bytes());
    }
}

pub struct NBTParser<'a> {
    pub data: &'a [u8],
    pub length: i64,
    pub pos: i64,
    pub root: Tag,
}

impl<'a> NBTParser<'a> {
    pub fn new(pdata: &'a [u8], plength: i64) -> Self {
        let mut parser = NBTParser { data: pdata, length: plength, pos: 0, root: Tag::End };
        let root = parser.parse_tag();
        if root.r#type() != TAG_COMPOUND {
            panic!("NBT root tag is not a compound!");
        }
        parser.root = root;
        parser
    }

    // Parse a tag, either with type and name bytes (parse_tag) or just a payload (parse_payload)
    pub fn parse_payload(&mut self, ptype: TagType, pname: &str) -> Tag {
        match ptype {
            TAG_BYTE => Tag::Byte { name: pname.to_string(), byte_value: self.read_i8() },
            TAG_SHORT => Tag::Short { name: pname.to_string(), short_value: self.read_i16() },
            TAG_INT => Tag::Int { name: pname.to_string(), int_value: self.read_i32() },
            TAG_LONG => Tag::Long { name: pname.to_string(), long_value: self.read_i64() },
            TAG_FLOAT => Tag::Float { name: pname.to_string(), float_value: self.read_f32() },
            TAG_DOUBLE => Tag::Double { name: pname.to_string(), double_value: self.read_f64() },
            TAG_STRING => Tag::String { name: pname.to_string(), string_value: self.read_string() },

            TAG_BYTEARRAY => {
                let count = self.read_i32();
                let mut byte_array = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    byte_array.push(self.read_i8());
                }
                Tag::ByteArray { name: pname.to_string(), byte_array }
            }

            TAG_INTARRAY => {
                let count = self.read_i32();
                let mut int_array = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    int_array.push(self.read_i32());
                }
                Tag::IntArray { name: pname.to_string(), int_array }
            }

            TAG_LIST => {
                let inner_type = TagType(self.read_i8() as u8);
                let count = self.read_i32();

                if inner_type == TAG_END && count > 0 {
                    panic!("Invalid TAG_List");
                }

                let mut list = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    list.push(self.parse_payload(inner_type, ""));
                }

                Tag::List { name: pname.to_string(), list_type: inner_type, list }
            }

            TAG_COMPOUND => {
                let mut compound = HashMap::new();
                loop {
                    let child = self.parse_tag();
                    if child.r#type() == TAG_END {
                        break;
                    }
                    compound.insert(child.name().to_string(), child);
                }
                Tag::Compound { name: pname.to_string(), compound }
            }

            _ => panic!("Unsupported payload type in list"),
        }
    }

    // Parse a tag including its type byte and name
    pub fn parse_tag(&mut self) -> Tag {
        if self.pos >= self.length {
            panic!("Unexpected end of NBT data");
        }

        let r#type = TagType(self.data[self.pos as usize]);
        self.pos += 1;
        if r#type == TAG_END {
            return Tag::End; // no name for TAG_End
        }

        let name = self.read_string();
        self.parse_payload(r#type, &name)
    }

    // Read helpers
    pub fn read_i32(&mut self) -> i32 {
        if self.pos + 4 > self.length {
            panic!("NBT: unexpected end");
        }
        let p = self.pos as usize;
        let v = ((self.data[p] as u32) << 24)
            | ((self.data[p + 1] as u32) << 16)
            | ((self.data[p + 2] as u32) << 8)
            | (self.data[p + 3] as u32);
        self.pos += 4;
        v as i32
    }

    pub fn read_i64(&mut self) -> i64 {
        let hi = self.read_i32() as u32;
        let lo = self.read_i32() as u32;
        (((hi as u64) << 32) | (lo as u64)) as i64
    }

    pub fn read_i16(&mut self) -> i16 {
        if self.pos + 2 > self.length {
            panic!("NBT: i16 out of bounds");
        }
        let p = self.pos as usize;
        let v = ((self.data[p] as u16) << 8) | (self.data[p + 1] as u16);
        self.pos += 2;
        v as i16
    }

    pub fn read_i8(&mut self) -> i8 {
        if self.pos >= self.length {
            panic!("NBT: i8 out of bounds");
        }
        let v = self.data[self.pos as usize] as i8;
        self.pos += 1;
        v
    }

    pub fn read_f32(&mut self) -> f32 {
        let raw = self.read_i32() as u32;
        f32::from_bits(raw)
    }

    pub fn read_f64(&mut self) -> f64 {
        let raw = self.read_i64() as u64;
        f64::from_bits(raw)
    }

    pub fn read_string(&mut self) -> String {
        let len = self.read_i16() as u16;
        if self.pos + (len as i64) > self.length {
            panic!("NBT: string out of bounds");
        }
        let p = self.pos as usize;
        let bytes = self.data[p..p + len as usize].to_vec();
        let s = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => e.into_bytes().into_iter().map(|b| b as char).collect(),
        };
        self.pos += len as i64;
        s
    }
}
