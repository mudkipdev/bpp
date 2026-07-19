/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Oracle/OpenJDK (1995-2024)
*/

// A reimplementation of the random function that Java provides
// https://docs.oracle.com/javase/8/docs/api/java/util/Random.html

// For more info, cross-reference with JDK source
// https://github.com/openjdk/jdk8u-dev/blob/master/jdk/src/share/classes/java/util/Random.java

use std::time::{SystemTime, UNIX_EPOCH};

/// @brief A faithful reimplementation of the Java pseudorandom number generator
///
pub struct Random {
    seed: u64,
}

impl Random {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const ADDEND: u64 = 0xB;
    const MASK: u64 = (1u64 << 48) - 1;

    /// @brief Performs a new iteration of the PRNG
    ///
    /// @return Pseudorandom 32-bit integer value
    fn next(&mut self, bits: i32) -> i32 {
        self.seed = self.seed.wrapping_mul(Self::MULTIPLIER).wrapping_add(Self::ADDEND) & Self::MASK;
        (self.seed >> (48 - bits)) as i32
    }

    /// @brief Construct a new Java Random object
    ///
    /// @param initialSeed The initial seed value (defaults to current time)
    pub fn with_seed(initial_seed: i64) -> Self {
        let mut random = Self { seed: 0 };
        random.set_seed(initial_seed);
        random
    }

    /// @brief Construct a new Java Random object
    ///
    pub fn new() -> Self {
        // Default seed: current time
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;
        Self::with_seed(now)
    }

    /// @brief Set the Seed value that'll be used for all subsequently generated values
    ///
    /// @param s Seed value
    pub fn set_seed(&mut self, s: i64) {
        self.seed = (s as u64 ^ Self::MULTIPLIER) & Self::MASK;
    }

    /// @brief Returns the next int32_t (32-bit integer)
    ///
    /// @return Pseudorandom 32-bit integer value
    pub fn next_int(&mut self) -> i32 {
        self.next(32)
    }

    /// @brief Returns the next bound int32_t (32-bit integer)
    ///
    /// @return Pseudorandom 32-bit integer value
    pub fn next_int_bound(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            panic!("bound must be positive");
        }

        if (bound & -bound) == bound {
            // power of two
            return ((bound as i64).wrapping_mul(self.next(31) as i64) >> 31) as i32;
        }

        let mut bits;
        let mut val;
        loop {
            bits = self.next(31);
            val = bits % bound;
            if (bits as u32)
                .wrapping_sub(val as u32)
                .wrapping_add((bound - 1) as u32) as i32
                >= 0
            {
                break;
            }
        }
        val
    }

    /// @brief Returns the next long (64-bit integer)
    ///
    /// @return Pseudorandom 64-bit long value
    pub fn next_long(&mut self) -> i64 {
        ((self.next(32) as i64) << 32).wrapping_add(self.next(32) as i64)
    }

    /// @brief Returns the next pseudorandom double
    ///
    /// @return Pseudorandom double value between 0.0 (inclusive) and 1.0 (exclusive)
    pub fn next_double(&mut self) -> f64 {
        (((self.next(26) as i64) << 27) + self.next(27) as i64) as f64 / (1i64 << 53) as f64
    }

    /// @brief Returns the next pseudorandom boolean
    ///
    /// @return Pseudorandom boolean
    pub fn next_boolean(&mut self) -> bool {
        self.next(1) != 0
    }

    /// @brief Returns the next pseudorandom float
    ///
    /// @return Pseudorandom float value between 0.0 (inclusive) and 1.0 (exclusive)
    pub fn next_float(&mut self) -> f32 {
        self.next(24) as f32 / (1i32 << 24) as f32
    }
}

impl Default for Random {
    fn default() -> Self {
        Self::new()
    }
}
