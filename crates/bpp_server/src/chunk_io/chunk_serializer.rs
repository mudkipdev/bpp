/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::io::Write;

use bpp_shared::constants::CHUNK_HEIGHT;
use bpp_shared::numeric_structs::Int3;
use bpp_shared::world::chunk::Chunk;
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub fn serialize(chunk: &Chunk) -> Vec<u8> {
    serialize_region(chunk, 0, 16, 0, CHUNK_HEIGHT, 0, 16)
}

pub fn serialize_region(
    chunk: &Chunk,
    xmin: i32,
    xmax: i32,
    ymin: i32,
    ymax: i32,
    zmin: i32,
    zmax: i32,
) -> Vec<u8> {
    let size_x = xmax - xmin;
    let size_y = ymax - ymin;
    let size_z = zmax - zmin;

    let blocks = size_x * size_y * size_z;
    let nibbles = (blocks + 1) / 2;
    let total = blocks + nibbles * 3;

    let mut raw: Vec<u8> = vec![0u8; total as usize];
    let (block_data, rest) = raw.split_at_mut(blocks as usize);
    let (meta_data, rest) = rest.split_at_mut(nibbles as usize);
    let (block_light, sky_light) = rest.split_at_mut(nibbles as usize);

    fn pack_nibble(byte: &mut u8, val: u8, high: bool) {
        if high {
            *byte = (*byte & 0x0F) | ((val & 0x0F) << 4);
        } else {
            *byte = (*byte & 0xF0) | (val & 0x0F);
        }
    }

    let mut i: i32 = 0;
    for x in xmin..xmax {
        for z in zmin..zmax {
            for y in ymin..ymax {
                let pos = Int3::new(x, y, z);
                block_data[i as usize] = chunk.get_block(pos).0 as u8;
                pack_nibble(
                    &mut meta_data[(i >> 1) as usize],
                    chunk.get_meta(pos),
                    (i & 1) != 0,
                );
                pack_nibble(
                    &mut block_light[(i >> 1) as usize],
                    chunk.get_block_light(pos),
                    (i & 1) != 0,
                );
                pack_nibble(
                    &mut sky_light[(i >> 1) as usize],
                    chunk.get_sky_light(pos),
                    (i & 1) != 0,
                );
                i += 1;
            }
        }
    }

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    if encoder.write_all(&raw).is_err() {
        return Vec::new();
    }
    match encoder.finish() {
        Ok(compressed) => compressed,
        Err(_) => Vec::new(),
    }
}
