/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Mojang Studios (2011)
*/

use crate::helpers::java::java_random::Random;
use crate::numeric_structs::Int32_2;
use crate::world::chunk::Chunk;
use crate::world::generator::shared::feature_gen::WorldWrapper;

/// @brief The base generator class
pub struct Generator {
    pub rand: Random,
    pub seed: i64,
}

impl Generator {
    pub fn new(seed: i64) -> Self {
        Self { rand: Random::new(), seed }
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new(0)
    }
}

pub trait GeneratorBehavior {
    fn base(&self) -> &Generator;
    fn base_mut(&mut self) -> &mut Generator;

    fn generate_chunk(&mut self, _chunk: &mut Chunk) {}

    fn populate_chunk(&mut self, _cpos: Int32_2, _world: &mut WorldWrapper) -> bool {
        true
    }
}
