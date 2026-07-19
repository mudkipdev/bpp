/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Mojang Studios (2011)
*/

// Library for emulating Java/Java Edition math functions

use std::sync::OnceLock;

/// @brief Linear interpolation function
///
/// @param t Interpolation factor
/// @param a Start value (t = 0.0)
/// @param b End value (t = 1.0)
/// @return Interpolated value between a and b
pub fn lerp(t: f64, a: f64, b: f64) -> f64 {
    a + t * (b - a)
}

/// @brief 3D Perlin noise gradient function
///
/// @param hash Hashed lattice value
/// @param x X of Distance Vector
/// @param y Y of Distance Vector
/// @param z Z of Distance Vector
/// @return double
pub fn grad3d(hash: i32, x: f64, y: f64, z: f64) -> f64 {
    let hash = hash & 15;
    let u = if hash < 8 { x } else { y };
    let v = if hash < 4 {
        y
    } else if hash != 12 && hash != 14 {
        z
    } else {
        x
    };
    (if (hash & 1) == 0 { u } else { -u }) + (if (hash & 2) == 0 { v } else { -v })
}

/// @brief 2D Perlin noise gradient function
///
/// @param hash Hashed lattice value
/// @param x X of Distance Vector
/// @param y Y of Distance Vector
/// @return double
pub fn grad2d(hash: i32, x: f64, y: f64) -> f64 {
    let hash = hash & 15;
    let u = (1 - ((hash & 8) >> 3)) as f64 * x;
    let v = if hash < 4 {
        0.0
    } else if hash != 12 && hash != 14 {
        y
    } else {
        x
    };
    (if (hash & 1) == 0 { u } else { -u }) + (if (hash & 2) == 0 { v } else { -v })
}

/// @brief Perlin-noise easing function
///
/// @param value Input value
/// @return Eased output value
pub fn fade(value: f64) -> f64 {
    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}

/// @brief Java-equivalent functions
///
// The following should be somewhat faithful implementation of
// Java's casting functions, as defined in
// "Chapter 5. Conversions and Contexts"
/// @brief Casts a double to a 64-bit integer
pub fn double_to_int64(value: f64) -> i64 {
    if value.is_nan() {
        return 0;
    }
    if value > i64::MAX as f64 {
        return i64::MAX;
    }
    if value < i64::MIN as f64 {
        return i64::MIN;
    }
    if value > 0.0 {
        return value.floor() as i64;
    }
    if value < 0.0 {
        return value.ceil() as i64;
    }
    0
}
/// @brief Casts a double to a 32-bit integer
pub fn double_to_int32(value: f64) -> i32 {
    if value.is_nan() {
        return 0;
    }
    if value > i32::MAX as f64 {
        return i32::MAX;
    }
    if value < i32::MIN as f64 {
        return i32::MIN;
    }
    if value > 0.0 {
        return value.floor() as i32;
    }
    if value < 0.0 {
        return value.ceil() as i32;
    }
    0
}
/// @brief Casts a float to a 64-bit integer
pub fn float_to_int64(value: f32) -> i64 {
    if value.is_nan() {
        return 0;
    }
    if value > i64::MAX as f32 {
        return i64::MAX;
    }
    if value < i64::MIN as f32 {
        return i64::MIN;
    }
    if value > 0.0 {
        return value.floor() as i64;
    }
    if value < 0.0 {
        return value.ceil() as i64;
    }
    0
}
/// @brief Casts a float to a 32-bit integer
pub fn float_to_int32(value: f32) -> i32 {
    if value.is_nan() {
        return 0;
    }
    if value > i32::MAX as f32 {
        return i32::MAX;
    }
    if value < i32::MIN as f32 {
        return i32::MIN;
    }
    if value > 0.0 {
        return value.floor() as i32;
    }
    if value < 0.0 {
        return value.ceil() as i32;
    }
    0
}

/// @brief Java-equivalent hashing function
///
/// @param value The input string
/// @return Hashed string expressed as an integer
pub fn hash_code(value: &str) -> i32 {
    let mut h: i32 = 0;
    if h == 0 && !value.is_empty() {
        for &byte in value.as_bytes() {
            h = 31i32.wrapping_mul(h).wrapping_add(byte as i8 as i32);
        }
    }
    h
}

/// @brief A struct that's used like Javas Math.java library
///
pub struct JavaMath;

impl JavaMath {
    pub const PI: f64 = std::f64::consts::PI;
    pub const PI_FLOAT: f32 = Self::PI as f32;

    pub fn abs(a: i32) -> i32 {
        if a < 0 { -a } else { a }
    }
}

/// @brief A small helper that's used to simplify or speed up some code
///
pub struct MathHelper;

impl MathHelper {
    pub const TABLE_SIZE: usize = 65536;

    fn sin_table() -> &'static [f32; Self::TABLE_SIZE] {
        static SIN_TABLE: OnceLock<[f32; MathHelper::TABLE_SIZE]> = OnceLock::new();
        SIN_TABLE.get_or_init(|| {
            let mut table = [0.0f32; MathHelper::TABLE_SIZE];
            for (i, slot) in table.iter_mut().enumerate() {
                *slot = ((i as f64) * JavaMath::PI * 2.0 / (MathHelper::TABLE_SIZE as f64)).sin() as f32;
            }
            table
        })
    }

    pub fn sin(x: f32) -> f32 {
        Self::sin_table()[(float_to_int32(x * 10430.378) & 0xFFFF) as usize]
    }

    pub fn cos(x: f32) -> f32 {
        Self::sin_table()[(float_to_int32(x * 10430.378 + 16384.0) & 0xFFFF) as usize]
    }

    pub fn sqrt_float(x: f32) -> f32 {
        x.sqrt()
    }

    pub fn sqrt_double(x: f64) -> f32 {
        x.sqrt() as f32
    }

    pub fn floor_float(x: f32) -> i32 {
        let i = float_to_int32(x);
        if x < i as f32 { i - 1 } else { i }
    }

    pub fn floor_double(x: f64) -> i32 {
        let i = double_to_int32(x);
        if x < i as f64 { i - 1 } else { i }
    }

    pub fn abs(x: f32) -> f32 {
        if x >= 0.0 { x } else { -x }
    }

    pub fn abs_max(a: f64, b: f64) -> f64 {
        let mut a = a;
        let mut b = b;
        if a < 0.0 {
            a = -a;
        }
        if b < 0.0 {
            b = -b;
        }
        if a > b { a } else { b }
    }

    pub fn init_sin_table() {
        Self::sin_table();
    }
}
