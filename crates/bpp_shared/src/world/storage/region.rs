/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

use crate::base_types::ItemId;
use crate::constants::{CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::enums::blocks::BlockType;
use crate::helpers::file_handle::FileHandle;
use crate::inventory::item_stack::ItemStack;
use crate::logger::logger::global_logger;
use crate::nbt::nbt::{NBTParser, NBTwriter, TAG_COMPOUND, Tag};
use crate::numeric_structs::{Int2, Int3, Int32_2};
use crate::tile_entities::tile_entity::{
    TileEntityChest, TileEntityDispenser, TileEntityFurnace, TileEntityMobSpawner, TileEntitySign,
};
use crate::world::chunk::{Chunk, ChunkState};

pub const REGION_WIDTH: i32 = 32;
pub const REGION_AREA: i32 = REGION_WIDTH * REGION_WIDTH;
pub const SECTOR_SIZE: u32 = 4096;

pub fn region_position_to_file_name(rpos: Int2) -> String {
    format!("r.{}.{}.mcr", rpos.x, *rpos.z())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CompressorFormat(pub u8);

pub const REGION_INVALID: CompressorFormat = CompressorFormat(0);
pub const REGION_GZIP: CompressorFormat = CompressorFormat(1);
pub const REGION_ZLIB: CompressorFormat = CompressorFormat(2);

#[derive(Clone, Copy, Debug, Default)]
pub struct FileHeaderEntry {
    pub offset: u32,
    pub number_of_sectors: u8,
    // TODO: Maybe store last-updated here?
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ChunkHeaderEntry {
    pub length: u32,
    pub format: u8,
}

pub struct Region {
    pub rpos: Int32_2,
    chunks: [Option<Arc<Mutex<Chunk>>>; REGION_AREA as usize],
    region_header: [FileHeaderEntry; REGION_AREA as usize],
    region_file: FileHandle,
}

impl Region {
    pub fn new(rpos: Int32_2, folder_path: String) -> Self {
        let mut region = Self {
            rpos,
            chunks: std::array::from_fn(|_| None),
            region_header: [FileHeaderEntry::default(); REGION_AREA as usize],
            region_file: FileHandle::open(&format!("{folder_path}/{}", region_position_to_file_name(rpos))),
        };
        // Cache our header
        region.read_header_from_file();
        region
    }

    pub fn chunk_exists(&self, localcpos: Int2) -> bool {
        let index = (localcpos.x + *localcpos.z() * 32) as usize;
        let r_header = &self.region_header[index];
        r_header.number_of_sectors != 0 && r_header.offset != 0
    }

    pub fn add_chunk(&mut self, chunk: Arc<Mutex<Chunk>>, timestamp: i64, entities: Option<Arc<Vec<Tag>>>) {
        let compressed = self.encode_nbt_data(&chunk, timestamp, entities);
        if compressed.is_empty() {
            return;
        }

        // Chunk header: 4 bytes length + 1 byte compression type
        let payload_length = compressed.len() as u32 + 1; // +1 for the format byte
        let sectors_needed = (payload_length + 4 + SECTOR_SIZE - 1) / SECTOR_SIZE;

        // Find a free run of sectors in the file
        // Build an occupancy set from the header
        let mut occupied: Vec<bool> = Vec::new();
        for i in 0..1024 {
            if self.region_header[i].offset == 0 {
                continue;
            }
            let end = self.region_header[i].offset + u32::from(self.region_header[i].number_of_sectors);
            if end as usize > occupied.len() {
                occupied.resize(end as usize, false);
            }
            for s in self.region_header[i].offset..end {
                occupied[s as usize] = true;
            }
        }

        // Find first free run of `sectorsNeeded` sectors starting at 2 (after header)
        let mut chosen_offset: u32 = 0;
        let mut s: u32 = 2;
        loop {
            let mut fits = true;
            for j in 0..sectors_needed {
                let idx = (s + j) as usize;
                if idx < occupied.len() && occupied[idx] {
                    fits = false;
                    break;
                }
            }
            if fits {
                chosen_offset = s;
                break;
            }
            s += 1;
        }

        // Write chunk data at the chosen sector
        let file = self.region_file.get();
        file.seek(SeekFrom::Start(u64::from(chosen_offset) * u64::from(SECTOR_SIZE))).unwrap();

        // 4-byte big-endian length
        file.write_all(&payload_length.to_be_bytes()).unwrap();

        // 1-byte compression type
        let format = REGION_ZLIB.0;
        file.write_all(&[format]).unwrap();

        // Compressed data
        file.write_all(&compressed).unwrap();

        // Pad to sector boundary
        let written = 5 + compressed.len();
        let padded = (sectors_needed as usize) * (SECTOR_SIZE as usize);
        if written < padded {
            let pad = vec![0u8; padded - written];
            file.write_all(&pad).unwrap();
        }

        // Update header entry
        let cpos = chunk.lock().unwrap().cpos;
        let local = Int2::new(cpos.x & 31, *cpos.z() & 31);
        let index = (local.x + *local.z() * 32) as usize;
        self.region_header[index].offset = chosen_offset;
        self.region_header[index].number_of_sectors = sectors_needed as u8;

        // Write updated header entry to file
        let file = self.region_file.get();
        file.seek(SeekFrom::Start((index * 4) as u64)).unwrap();
        let entry = (chosen_offset << 8) | u32::from(sectors_needed as u8);
        file.write_all(&entry.to_be_bytes()).unwrap();
        file.flush().unwrap();
    }

    pub fn get_chunk(&mut self, cpos: Int32_2) -> Option<Arc<Mutex<Chunk>>> {
        let local = Int2::new(cpos.x & 31, *cpos.z() & 31);
        let index = (local.x + *local.z() * 32) as usize;

        if !self.chunk_exists(local) {
            return None;
        }

        let offset = self.region_header[index].offset;
        if offset == 0 {
            return None;
        }

        let file = self.region_file.get();
        if file.seek(SeekFrom::Start(u64::from(offset) * u64::from(SECTOR_SIZE))).is_err() {
            return None;
        }

        // Read 4-byte length
        let mut length_buf = [0u8; 4];
        if file.read_exact(&mut length_buf).is_err() {
            return None;
        }
        let length = u32::from_be_bytes(length_buf);

        // Guard against corrupt/zero length before subtracting
        if length < 1 {
            global_logger().warn(format!("Invalid chunk length: {length}\n"));
            return None;
        }

        // Read compression type
        let mut format_buf = [0u8; 1];
        if file.read_exact(&mut format_buf).is_err() {
            return None;
        }
        let format = format_buf[0];

        if format != REGION_ZLIB.0 {
            global_logger().warn(format!("Unsupported compression format: {format}\n"));
            return None;
        }

        // Read compressed data (length includes the format byte, so actual data is length-1)
        let mut compressed = vec![0u8; (length - 1) as usize];
        if file.read_exact(&mut compressed).is_err() {
            return None;
        }

        Some(self.decode_nbt_data(&compressed))
    }

    // Read our header data into the "regionHeader"
    pub fn read_header_from_file(&mut self) {
        let file = self.region_file.get();
        file.seek(SeekFrom::Start(0)).unwrap(); // Beginning of sector 0
        for i in 0..1024 {
            let mut buf = [0u8; 4];
            file.read_exact(&mut buf).unwrap();
            let entry = u32::from_be_bytes(buf);
            self.region_header[i].number_of_sectors = (entry & 0xFF) as u8; // bottom 1 byte
            self.region_header[i].offset = entry >> 8; // top 3 bytes
        }
    }

    fn encode_nbt_data(&self, chunk: &Arc<Mutex<Chunk>>, timestamp: i64, entities: Option<Arc<Vec<Tag>>>) -> Vec<u8> {
        let chunk_guard = chunk.lock().unwrap();

        // Byte array, blocks
        let mut blocks_array = vec![0i8; (CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT) as usize];

        // Nibble arrays (4 bits per block, so half the size of blocks array)
        let mut data_array = vec![0i8; ((CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT) / 2) as usize];

        let mut block_light_array = vec![0i8; ((CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT) / 2) as usize];

        let mut sky_light_array = vec![0i8; ((CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT) / 2) as usize];

        // HeightMap, one byte per (x,z) column
        let mut height_map_array = vec![0i8; (CHUNK_WIDTH * CHUNK_WIDTH) as usize];
        for (i, h) in chunk_guard.height_map.iter().enumerate() {
            height_map_array[i] = *h as i8;
        }

        // Put blocks in there
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                for y in 0..CHUNK_HEIGHT {
                    let idx = (y + (z * CHUNK_HEIGHT) + (x * CHUNK_HEIGHT * CHUNK_WIDTH)) as usize;
                    blocks_array[idx] = chunk_guard.get_block(Int3::new(x, y, z)).0;
                    if y % 2 == 0 {
                        data_array[idx / 2] |= chunk_guard.get_meta(Int3::new(x, y, z)) as i8;
                        sky_light_array[idx / 2] |= chunk_guard.get_sky_light(Int3::new(x, y, z)) as i8;
                        block_light_array[idx / 2] |= chunk_guard.get_block_light(Int3::new(x, y, z)) as i8;
                    } else {
                        data_array[idx / 2] |= (chunk_guard.get_meta(Int3::new(x, y, z)) << 4) as i8;
                        sky_light_array[idx / 2] |= (chunk_guard.get_sky_light(Int3::new(x, y, z)) << 4) as i8;
                        block_light_array[idx / 2] |= (chunk_guard.get_block_light(Int3::new(x, y, z)) << 4) as i8;
                    }
                }
            }
        }

        // List tag for entities
        let mut entities_list: Vec<Tag> = Vec::new();
        if let Some(entities) = &entities {
            entities_list.extend(entities.iter().cloned());
        }
        let entities_tag = Tag::List { name: "Entities".to_string(), list_type: TAG_COMPOUND, list: entities_list };

        // Nested compound inside a list for tile entities
        let mut tile_entities_list: Vec<Tag> = Vec::new();
        for te in &chunk_guard.tile_entities {
            tile_entities_list.push(te.lock().unwrap().serialize());
        }
        let tile_entities_tag =
            Tag::List { name: "TileEntities".to_string(), list_type: TAG_COMPOUND, list: tile_entities_list };

        // Assemble level compound
        let mut level_compound = HashMap::new();
        level_compound.insert("xPos".to_string(), Tag::Int { name: "xPos".to_string(), int_value: chunk_guard.cpos.x });
        level_compound
            .insert("zPos".to_string(), Tag::Int { name: "zPos".to_string(), int_value: *chunk_guard.cpos.z() });
        level_compound.insert(
            "TerrainPopulated".to_string(),
            Tag::Byte { name: "TerrainPopulated".to_string(), byte_value: chunk_guard.is_terrain_populated as i8 },
        );
        level_compound
            .insert("LastUpdate".to_string(), Tag::Long { name: "LastUpdate".to_string(), long_value: timestamp });
        level_compound
            .insert("Blocks".to_string(), Tag::ByteArray { name: "Blocks".to_string(), byte_array: blocks_array });
        level_compound.insert("Data".to_string(), Tag::ByteArray { name: "Data".to_string(), byte_array: data_array });
        level_compound.insert(
            "BlockLight".to_string(),
            Tag::ByteArray { name: "BlockLight".to_string(), byte_array: block_light_array },
        );
        level_compound.insert(
            "SkyLight".to_string(),
            Tag::ByteArray { name: "SkyLight".to_string(), byte_array: sky_light_array },
        );
        level_compound.insert(
            "HeightMap".to_string(),
            Tag::ByteArray { name: "HeightMap".to_string(), byte_array: height_map_array },
        );
        level_compound.insert("Entities".to_string(), entities_tag);
        level_compound.insert("TileEntities".to_string(), tile_entities_tag);

        let level = Tag::Compound { name: "Level".to_string(), compound: level_compound };

        let mut root_compound = HashMap::new();
        root_compound.insert("Level".to_string(), level);
        let root = Tag::Compound { name: String::new(), compound: root_compound };

        // Serialize to bytes
        let mut raw = Vec::new();
        NBTwriter::new(&mut raw, &root);
        // Compress
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        if encoder.write_all(&raw).is_err() {
            return Vec::new();
        }
        match encoder.finish() {
            Ok(compressed) => compressed,
            Err(_) => Vec::new(),
        }
    }

    fn decode_nbt_data(&self, raw_data: &[u8]) -> Arc<Mutex<Chunk>> {
        let mut decompressed = Vec::new();
        let mut decoder = ZlibDecoder::new(raw_data);
        if decoder.read_to_end(&mut decompressed).is_err() {
            global_logger().warn("Decompression failed!\n");
        }

        let parser = NBTParser::new(&decompressed, decompressed.len() as i64);
        let lvl = parser.root.get("Level");

        let cx = lvl.get("xPos").get_int();
        let cz = lvl.get("zPos").get_int();
        let tp = lvl.get("TerrainPopulated").get_byte() != 0;
        let _lu = lvl.get("LastUpdate").get_long();

        let blocks = lvl.get("Blocks").get_byte_array();
        let data = lvl.get("Data").get_byte_array();
        let block_light = lvl.get("BlockLight").get_byte_array();
        let sky_light = lvl.get("SkyLight").get_byte_array();
        let height_map = lvl.get("HeightMap").get_byte_array();
        let tile_entities = lvl.get("TileEntities").get_list();
        let entities = lvl.get("Entities").get_list();

        // Setup our chunk
        let mut chunk = Chunk::default();
        chunk.cpos = Int32_2::new(cx, cz);
        chunk.state_store(if tp { ChunkState::Populated } else { ChunkState::Generated });
        chunk.is_terrain_populated = tp;
        for (i, h) in height_map.iter().enumerate() {
            chunk.height_map[i] = *h as u8;
        }

        // Load all of our block data
        for y in 0..CHUNK_HEIGHT {
            for x in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_WIDTH {
                    let idx = (y + (z * CHUNK_HEIGHT) + (x * CHUNK_HEIGHT * CHUNK_WIDTH)) as usize;
                    chunk.set_block(Int3::new(x, y, z), BlockType(blocks[idx]));
                    if y % 2 == 0 {
                        chunk.set_meta(Int3::new(x, y, z), (data[idx / 2] & 0xF) as u8);
                        chunk.set_block_light(Int3::new(x, y, z), (block_light[idx / 2] & 0xF) as u8);
                        chunk.set_sky_light(Int3::new(x, y, z), (sky_light[idx / 2] & 0xF) as u8);
                    } else {
                        chunk.set_meta(Int3::new(x, y, z), ((data[idx / 2] >> 4) & 0xF) as u8);
                        chunk.set_block_light(Int3::new(x, y, z), ((block_light[idx / 2] >> 4) & 0xF) as u8);
                        chunk.set_sky_light(Int3::new(x, y, z), ((sky_light[idx / 2] >> 4) & 0xF) as u8);
                    }
                }
            }
        }

        // Load our entities
        chunk.entity_tags = entities.to_vec();

        // Load our tile entities
        for te in tile_entities {
            let id = te.get("id").get_string();
            let tx = te.get("x").get_int();
            let ty = te.get("y").get_int();
            let tz = te.get("z").get_int();
            let pos = Int3::new(tx, ty, tz);

            // load a standard slot-based inventory from an Items list tag
            let load_slots = |slots: &mut [ItemStack]| {
                if !te.has("Items") {
                    return;
                }
                for item in te.get("Items").get_list() {
                    let slot = item.get("Slot").get_byte();
                    let item_id = item.get("id").get_short();
                    let count = item.get("Count").get_byte();
                    let damage = item.get("Damage").get_short();
                    if slot >= 0 && (slot as usize) < slots.len() {
                        slots[slot as usize] = ItemStack { id: ItemId(item_id), count, data: damage };
                    }
                }
            };

            if id == "Chest" {
                let mut ent = TileEntityChest::new(pos);
                load_slots(&mut ent.inventory.base.slots);
                chunk.tile_entities.push(Arc::new(Mutex::new(ent)));
            } else if id == "Furnace" {
                let mut ent = TileEntityFurnace::new(pos);
                load_slots(&mut ent.inventory.base.slots);
                chunk.tile_entities.push(Arc::new(Mutex::new(ent)));
            } else if id == "Trap" {
                let mut ent = TileEntityDispenser::new(pos);
                load_slots(&mut ent.inventory.base.slots);
                chunk.tile_entities.push(Arc::new(Mutex::new(ent)));
            } else if id == "Sign" {
                let mut ent = TileEntitySign::new(pos);
                if te.has("Text1") {
                    ent.text1 = te.get("Text1").get_string().to_string();
                }
                if te.has("Text2") {
                    ent.text2 = te.get("Text2").get_string().to_string();
                }
                if te.has("Text3") {
                    ent.text3 = te.get("Text3").get_string().to_string();
                }
                if te.has("Text4") {
                    ent.text4 = te.get("Text4").get_string().to_string();
                }
                chunk.tile_entities.push(Arc::new(Mutex::new(ent)));
            } else if id == "MobSpawner" {
                let mut ent = TileEntityMobSpawner::new(pos);
                if te.has("EntityId") {
                    ent.entity_id = te.get("EntityId").get_string().to_string();
                }
                if te.has("Delay") {
                    ent.delay = te.get("Delay").get_short();
                }
                chunk.tile_entities.push(Arc::new(Mutex::new(ent)));
            }
        }

        Arc::new(Mutex::new(chunk))
    }
}
