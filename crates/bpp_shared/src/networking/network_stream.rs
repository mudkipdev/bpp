/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream};

use crate::base_types::ItemId;
use crate::enums::items;
use crate::enums::network::packet_data::entity_metadata;
use crate::inventory::item_stack::ItemStack;
use crate::logger::logger::global_logger;
use crate::numeric_structs::Int32_3;
use crate::strings::ucs2::{to_ucs2, to_utf8};

pub struct NetworkStream {
    client_socket: Option<TcpStream>,
    connected: bool,
    short_read: bool,
    // Bytes that were fetched from the socket but belong to a packet that could
    // not be fully read this tick. Drained before touching the socket again.
    read_back_buffer: VecDeque<u8>,
    write_buffer: Vec<u8>,
}

impl NetworkStream {
    pub fn new(client_socket: TcpStream) -> Self {
        let _ = client_socket.set_nonblocking(true);
        NetworkStream {
            client_socket: Some(client_socket),
            connected: true,
            short_read: false,
            read_back_buffer: VecDeque::new(),
            write_buffer: Vec::new(),
        }
    }

    pub fn new_detached() -> Self {
        NetworkStream {
            client_socket: None,
            connected: false,
            short_read: false,
            read_back_buffer: VecDeque::new(),
            write_buffer: Vec::new(),
        }
    }

    pub fn read_i8(&mut self) -> i8 {
        i8::from_be_bytes(self.read_array())
    }

    pub fn read_u8(&mut self) -> u8 {
        u8::from_be_bytes(self.read_array())
    }

    pub fn read_i16(&mut self) -> i16 {
        i16::from_be_bytes(self.read_array())
    }

    pub fn read_u16(&mut self) -> u16 {
        u16::from_be_bytes(self.read_array())
    }

    pub fn read_i32(&mut self) -> i32 {
        i32::from_be_bytes(self.read_array())
    }

    pub fn read_u32(&mut self) -> u32 {
        u32::from_be_bytes(self.read_array())
    }

    pub fn read_i64(&mut self) -> i64 {
        i64::from_be_bytes(self.read_array())
    }

    pub fn read_u64(&mut self) -> u64 {
        u64::from_be_bytes(self.read_array())
    }

    pub fn read_f32(&mut self) -> f32 {
        f32::from_bits(u32::from_be_bytes(self.read_array()))
    }

    pub fn read_f64(&mut self) -> f64 {
        f64::from_bits(u64::from_be_bytes(self.read_array()))
    }

    pub fn read_bool(&mut self) -> bool {
        self.read_i8() != 0
    }

    pub fn write_i8(&mut self, data: i8) {
        self.write_bytes(&data.to_be_bytes());
    }

    pub fn write_u8(&mut self, data: u8) {
        self.write_bytes(&data.to_be_bytes());
    }

    pub fn write_i16(&mut self, data: i16) {
        self.write_bytes(&data.to_be_bytes());
    }

    pub fn write_u16(&mut self, data: u16) {
        self.write_bytes(&data.to_be_bytes());
    }

    pub fn write_i32(&mut self, data: i32) {
        self.write_bytes(&data.to_be_bytes());
    }

    pub fn write_u32(&mut self, data: u32) {
        self.write_bytes(&data.to_be_bytes());
    }

    pub fn write_i64(&mut self, data: i64) {
        self.write_bytes(&data.to_be_bytes());
    }

    pub fn write_u64(&mut self, data: u64) {
        self.write_bytes(&data.to_be_bytes());
    }

    pub fn write_f32(&mut self, data: f32) {
        self.write_bytes(&data.to_bits().to_be_bytes());
    }

    pub fn write_f64(&mut self, data: f64) {
        self.write_bytes(&data.to_bits().to_be_bytes());
    }

    pub fn write_bool(&mut self, data: bool) {
        self.write_i8(data as i8);
    }

    pub fn set_connected(&mut self, val: bool) {
        self.connected = val;
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    // String-8 Handling
    pub fn read_string8(&mut self) -> String {
        let len = self.read_u16(); // adjust to u32 if your protocol uses 4-byte lengths
        let mut result = vec![0u8; len as usize];
        self.read_bytes(&mut result);
        match String::from_utf8(result) {
            Ok(s) => s,
            Err(e) => e.into_bytes().into_iter().map(|b| b as char).collect(),
        }
    }

    pub fn write_string8(&mut self, str: &str) {
        let length = str.len() as u16;
        self.write_u16(length);
        let mut data: Vec<u8> = Vec::new();
        data.reserve(str.len() * 2);
        for c in str.bytes() {
            data.push(c);
        }
        self.write_bytes(&data);
    }

    // String-16 Handling
    pub fn read_string16(&mut self) -> String {
        let len = self.read_u16();

        // Read as UTF-16 (2 bytes per char) regardless of platform wchar_t size
        let mut buf = vec![0u8; len as usize * 2];
        self.read_bytes(&mut buf);

        let mut result: Vec<u16> = Vec::with_capacity(len as usize);
        for i in 0..len as usize {
            // byteswap each UTF-16 unit, then widen to wchar_t
            result.push(u16::from_be_bytes([buf[i * 2], buf[i * 2 + 1]]));
        }
        to_utf8(result)
    }

    pub fn write_string16(&mut self, str: &str) {
        let str16 = to_ucs2(str.to_string());
        let length = str16.len() as u16;
        self.write_u16(length);
        let mut data: Vec<u8> = Vec::new();
        data.reserve(str16.len());
        for c in str16 {
            data.push(((c >> 8) & 0xFF) as u8);
            data.push((c & 0xFF) as u8);
        }
        self.write_bytes(&data);
    }

    // Raw byte buffer Read-Write (no endian conversion).
    // On a short read (EAGAIN/EWOULDBLOCK mid-packet), all bytes fetched so far
    // are pushed back into readBackBuffer so they are re-read next tick.
    // shortRead is set; the caller does NOT need to unread anything manually.
    pub fn read_bytes(&mut self, buf: &mut [u8]) -> usize {
        let len = buf.len();
        let mut received: usize = 0;

        // 1. consume existing buffered data
        while !self.read_back_buffer.is_empty() && received < len {
            buf[received] = self.read_back_buffer.pop_front().unwrap();
            received += 1;
        }

        // 2. try recv until we either fill or would block
        if let Some(stream) = self.client_socket.as_mut() {
            while received < len {
                match stream.read(&mut buf[received..len]) {
                    Ok(0) => {
                        self.connected = false;
                        return received;
                    }
                    Ok(n) => {
                        received += n;
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        break;
                    }
                    Err(_) => {
                        self.connected = false;
                        return received;
                    }
                }
            }
        }

        received
    }

    // Append bytes to the per-session write buffer (no syscall).
    pub fn write_bytes(&mut self, buf: &[u8]) {
        // Append to the write buffer -- no syscall here.
        // The actual send() happens once per tick in flushWriteBuffer().
        self.write_buffer.extend_from_slice(buf);
    }

    // Handles Entity Metadata Interpreting
    // TODO: Due to how this system works, a concrete length is never supplied.
    // Data is read until 0x7F is hit. Ideally we should exit out if we're past
    // a certain number of bytes
    pub fn read_entity_metadata(&mut self, metadata: &mut Vec<entity_metadata::DataEntry>) {
        let mut val = self.read_u8();
        while val != entity_metadata::END {
            // What type the data has
            let r#type = entity_metadata::Type(val >> 5);
            // Where the data goes for the relevant entity
            let index = val & 0x1F;
            match r#type {
                entity_metadata::BYTE => {
                    let num = self.read_i8();
                    metadata.push(entity_metadata::DataEntry {
                        r#type,
                        index,
                        value: entity_metadata::Value::Byte(num),
                    });
                }
                entity_metadata::SHORT => {
                    let num = self.read_i16();
                    metadata.push(entity_metadata::DataEntry {
                        r#type,
                        index,
                        value: entity_metadata::Value::Short(num),
                    });
                }
                entity_metadata::INTEGER => {
                    let num = self.read_i32();
                    metadata.push(entity_metadata::DataEntry {
                        r#type,
                        index,
                        value: entity_metadata::Value::Integer(num),
                    });
                }
                entity_metadata::FLOAT => {
                    let num = self.read_f32();
                    metadata.push(entity_metadata::DataEntry {
                        r#type,
                        index,
                        value: entity_metadata::Value::Float(num),
                    });
                }
                entity_metadata::STRING => {
                    let str = self.read_string16();
                    metadata.push(entity_metadata::DataEntry {
                        r#type,
                        index,
                        value: entity_metadata::Value::String(str),
                    });
                }
                entity_metadata::ITEM => {
                    let mut item = ItemStack::default();
                    item.id = ItemId(self.read_i16());
                    // TODO: Check if B1.7.3 actually does
                    // this for Entity Metadata too
                    if item.id != items::INVALID {
                        item.count = self.read_i8();
                        item.data = self.read_i16();
                    }
                    metadata.push(entity_metadata::DataEntry {
                        r#type,
                        index,
                        value: entity_metadata::Value::Item(item),
                    });
                }
                entity_metadata::COORDINATES => {
                    let coordinate =
                        Int32_3::new(self.read_i32(), self.read_i32(), self.read_i32());
                    metadata.push(entity_metadata::DataEntry {
                        r#type,
                        index,
                        value: entity_metadata::Value::Coordinates(coordinate),
                    });
                }
                _ => {
                    global_logger().warn(format!(
                        "NetworkStream::ReadEntityMetadata: Unknown metadata type {}\n",
                        r#type.0 as i32
                    ));
                }
            }
            // Read in the next value
            val = self.read_u8();
        }
    }

    // Handles Entity Metadata Conversion
    // TODO: Implement this! Ideally we could just pass an entity into here
    // and it'd take care of things automatically
    pub fn write_entity_metadata(&mut self, metadata: &[entity_metadata::DataEntry]) {
        for entry in metadata {
            let val = (entry.r#type.0 << 5) | (entry.index & 0x1F);
            self.write_u8(val);
            match &entry.value {
                entity_metadata::Value::Byte(num) => self.write_i8(*num),
                entity_metadata::Value::Short(num) => self.write_i16(*num),
                entity_metadata::Value::Integer(num) => self.write_i32(*num),
                entity_metadata::Value::Float(num) => self.write_f32(*num),
                entity_metadata::Value::String(str) => self.write_string16(str),
                entity_metadata::Value::Item(item) => {
                    self.write_i16(item.id.value());
                    if item.id != items::INVALID {
                        self.write_i8(item.count);
                        self.write_i16(item.data);
                    }
                }
                entity_metadata::Value::Coordinates(coordinate) => {
                    self.write_i32(coordinate.x);
                    self.write_i32(coordinate.y);
                    self.write_i32(coordinate.z);
                }
            }
        }
        self.write_u8(entity_metadata::END);
    }

    // Flush the write buffer to the socket once per tick.
    // Returns false if the connection was lost.
    pub fn flush_write_buffer(&mut self) -> bool {
        if self.write_buffer.is_empty() {
            return self.connected;
        }
        let mut sent: usize = 0;
        if let Some(stream) = self.client_socket.as_mut() {
            while sent < self.write_buffer.len() {
                match stream.write(&self.write_buffer[sent..]) {
                    Ok(0) => {
                        self.connected = false;
                        break;
                    }
                    Ok(n) => {
                        sent += n;
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        break;
                    }
                    Err(_) => {
                        self.connected = false;
                        break;
                    }
                }
            }
        }
        if sent > 0 {
            self.write_buffer.drain(0..sent);
        }
        self.connected
    }

    // Blocking flush for use SHUTDOWN ONLY
    pub fn flush_write_buffer_blocking(&mut self) {
        if self.write_buffer.is_empty() || self.client_socket.is_none() {
            return;
        }

        let mut stream = self.client_socket.take().unwrap();

        // Switch to blocking mode
        let _ = stream.set_nonblocking(false);

        let mut sent: usize = 0;
        while sent < self.write_buffer.len() {
            match stream.write(&self.write_buffer[sent..]) {
                Ok(0) => break,
                Ok(n) => sent += n,
                Err(_) => break,
            }
        }
        self.write_buffer.clear();

        // We close here so the client can get the packet data we just sent out before we disconnect
        let _ = stream.shutdown(Shutdown::Write);
    }

    pub fn has_data(&mut self) -> bool {
        // Check rollback buffer first, then the socket.
        if !self.read_back_buffer.is_empty() {
            return true;
        }
        match self.client_socket.as_ref() {
            Some(stream) => {
                let mut buf = [0u8; 1];
                match stream.peek(&mut buf) {
                    Ok(n) => n > 0,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => false,
                    Err(_) => {
                        self.connected = false;
                        false
                    }
                }
            }
            None => {
                self.connected = false;
                false
            }
        }
    }

    // Append pre-serialised bytes directly to the write buffer.
    // Used for shared-packet broadcast: serialise once, copy to N sessions.
    pub fn write_raw(&mut self, data: &[u8]) {
        self.write_bytes(data);
    }

    // Read-only view of the pending write buffer.
    // Valid only until the next Write*/writeRaw/flushWriteBuffer call.
    pub fn get_raw_write_buffer(&self) -> &Vec<u8> {
        &self.write_buffer
    }

    // Returns true if the last ReadBytes call hit a receive timeout (packet split
    // across ticks). All bytes that had already been read are held in readBackBuffer
    // and will be replayed automatically on the next ReadBytes call.
    pub fn check_and_clear_short_read(&mut self) -> bool {
        let val = self.short_read;
        self.short_read = false;
        val
    }

    fn read_array<const N: usize>(&mut self) -> [u8; N] {
        let mut buf = [0u8; N];
        self.read_bytes(&mut buf);
        buf
    }
}

impl Drop for NetworkStream {
    fn drop(&mut self) {
        if let Some(stream) = self.client_socket.take() {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }
}
