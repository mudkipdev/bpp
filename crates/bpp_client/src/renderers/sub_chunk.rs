/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use bpp_shared::numeric_structs::Int2;

pub struct SubChunk {
    pub chunk_pos: Int2,
    pub offset: Int2, // world offset of this sub chunk (chunkPos * 16) + offset
    pub chunk_slice: i32, // 0-8, y position of this slice

    // Opaque geometry
    pub vao: u32,
    pub vbo: u32,
    pub vertex_count: i32,

    // Translucent geometry (water, glass, etc.)
    pub trans_vao: u32,
    pub trans_vbo: u32,
    pub trans_vertex_count: i32,

    // Overlay geometry (grass overlay, etc)
    pub overlay_vao: u32,
    pub overlay_vbo: u32,
    pub overlay_vertex_count: i32,

    pub parent_chunk: i64, // so we can easily find the parent chunk when we need to update this sub chunk, hash of ChunkPos
    pub dirty: bool,
}

impl Default for SubChunk {
    fn default() -> Self {
        Self {
            chunk_pos: Int2::new(0, 0),
            offset: Int2::new(0, 0),
            chunk_slice: 0,
            vao: 0,
            vbo: 0,
            vertex_count: 0,
            trans_vao: 0,
            trans_vbo: 0,
            trans_vertex_count: 0,
            overlay_vao: 0,
            overlay_vbo: 0,
            overlay_vertex_count: 0,
            parent_chunk: 0,
            dirty: true,
        }
    }
}

impl SubChunk {
    // since we handle gpu buffer objects we have to be very careful about copying!
    fn cleanup(&mut self) {
        if self.vao != 0 {
            unsafe {
                gl::DeleteVertexArrays(1, &self.vao);
            }
            unsafe {
                gl::DeleteBuffers(1, &self.vbo);
            }
            self.vao = 0;
            self.vbo = 0;
        }
        if self.trans_vao != 0 {
            unsafe {
                gl::DeleteVertexArrays(1, &self.trans_vao);
            }
            unsafe {
                gl::DeleteBuffers(1, &self.trans_vbo);
            }
            self.trans_vao = 0;
            self.trans_vbo = 0;
        }
        if self.overlay_vao != 0 {
            unsafe {
                gl::DeleteVertexArrays(1, &self.overlay_vao);
            }
            unsafe {
                gl::DeleteBuffers(1, &self.overlay_vbo);
            }
            self.overlay_vao = 0;
            self.overlay_vbo = 0;
        }
    }
}

impl Drop for SubChunk {
    fn drop(&mut self) {
        self.cleanup();
    }
}
