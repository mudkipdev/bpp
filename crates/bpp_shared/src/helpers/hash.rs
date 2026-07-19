/*
 * Copyright (c) 2026, jwaxy <jwaxy.is-a.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::hash::{DefaultHasher, Hash, Hasher};

// Taken from http://www.boost.org/doc/libs/1_55_0/doc/html/hash/reference.html#boost.hash_combine
// TODO: There might be a better alternative worth testing:
// https://stackoverflow.com/questions/35985960/c-why-is-boosthash-combine-the-best-way-to-combine-hash-values
pub fn hash_combine<T: Hash>(h: &mut u64, v: T) {
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    *h ^= hasher
        .finish()
        .wrapping_add(0x9e3779b97f4a7c15)
        .wrapping_add(*h << 6)
        .wrapping_add(*h >> 2);
}
