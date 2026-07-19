/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

#![allow(non_camel_case_types)]

use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

/// @brief A struct that contains three numbers (x,y,z)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TriNumber<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> TriNumber<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: Copy + Add<Output = T>> Add for TriNumber<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl<T: Copy + Sub<Output = T>> Sub for TriNumber<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<T: Copy + Mul<Output = T>> Mul for TriNumber<T> {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

impl<T: Copy + Div<Output = T>> Div for TriNumber<T> {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        Self::new(self.x / other.x, self.y / other.y, self.z / other.z)
    }
}

impl<T: Copy + Add<Output = T>> AddAssign for TriNumber<T> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl<T: Copy + Sub<Output = T>> SubAssign for TriNumber<T> {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl<T: Copy + Mul<Output = T>> MulAssign for TriNumber<T> {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl<T: Copy + Div<Output = T>> DivAssign for TriNumber<T> {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

// Allows for tri + 1 = (x+1,y+1,z+1)
impl<T: Copy + Add<Output = T>> Add<T> for TriNumber<T> {
    type Output = Self;
    fn add(self, other: T) -> Self {
        Self::new(self.x + other, self.y + other, self.z + other)
    }
}

// Allows for tri - 1 = (x-1,y-1,z-1)
impl<T: Copy + Sub<Output = T>> Sub<T> for TriNumber<T> {
    type Output = Self;
    fn sub(self, other: T) -> Self {
        Self::new(self.x - other, self.y - other, self.z - other)
    }
}

// Allows for tri * 2 = (x*2,y*2,z*2)
impl<T: Copy + Mul<Output = T>> Mul<T> for TriNumber<T> {
    type Output = Self;
    fn mul(self, other: T) -> Self {
        Self::new(self.x * other, self.y * other, self.z * other)
    }
}

// Allows for tri / 2 = (x/2,y/2,z/2)
impl<T: Copy + Div<Output = T>> Div<T> for TriNumber<T> {
    type Output = Self;
    fn div(self, other: T) -> Self {
        Self::new(self.x / other, self.y / other, self.z / other)
    }
}

impl<T: Copy + Add<Output = T>> AddAssign<T> for TriNumber<T> {
    fn add_assign(&mut self, other: T) {
        *self = *self + other;
    }
}

impl<T: Copy + Sub<Output = T>> SubAssign<T> for TriNumber<T> {
    fn sub_assign(&mut self, other: T) {
        *self = *self - other;
    }
}

impl<T: Copy + Mul<Output = T>> MulAssign<T> for TriNumber<T> {
    fn mul_assign(&mut self, other: T) {
        *self = *self * other;
    }
}

impl<T: Copy + Div<Output = T>> DivAssign<T> for TriNumber<T> {
    fn div_assign(&mut self, other: T) {
        *self = *self / other;
    }
}

impl<T: fmt::Display> fmt::Display for TriNumber<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl<T> Index<usize> for TriNumber<T> {
    type Output = T;
    fn index(&self, axis: usize) -> &T {
        match axis {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("TriNumber index out of range: {axis}"),
        }
    }
}

impl<T> IndexMut<usize> for TriNumber<T> {
    fn index_mut(&mut self, axis: usize) -> &mut T {
        match axis {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("TriNumber index out of range: {axis}"),
        }
    }
}

impl<T: Copy + Mul<Output = T>> TriNumber<T> {
    pub fn total(self) -> T {
        self.x * self.y * self.z
    }
}

/// @brief A struct that contains two numbers (x,y)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BiNumber<T> {
    pub x: T,
    pub y: T,
}

impl<T> BiNumber<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn z(&self) -> &T {
        &self.y
    }

    pub fn z_mut(&mut self) -> &mut T {
        &mut self.y
    }
}

impl<T: Copy + Add<Output = T>> Add for BiNumber<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl<T: Copy + Sub<Output = T>> Sub for BiNumber<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl<T: Copy + Mul<Output = T>> Mul for BiNumber<T> {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y)
    }
}

impl<T: Copy + Div<Output = T>> Div for BiNumber<T> {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        Self::new(self.x / other.x, self.y / other.y)
    }
}

impl<T: Copy + Add<Output = T>> AddAssign for BiNumber<T> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl<T: Copy + Sub<Output = T>> SubAssign for BiNumber<T> {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl<T: Copy + Mul<Output = T>> MulAssign for BiNumber<T> {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl<T: Copy + Div<Output = T>> DivAssign for BiNumber<T> {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

// Allows for bi + 2 = (x+2,y+2)
impl<T: Copy + Add<Output = T>> Add<T> for BiNumber<T> {
    type Output = Self;
    fn add(self, other: T) -> Self {
        Self::new(self.x + other, self.y + other)
    }
}

// Allows for bi - 2 = (x-2,y-2)
impl<T: Copy + Sub<Output = T>> Sub<T> for BiNumber<T> {
    type Output = Self;
    fn sub(self, other: T) -> Self {
        Self::new(self.x - other, self.y - other)
    }
}

// Allows for bi * 2 = (x*2,y*2)
impl<T: Copy + Mul<Output = T>> Mul<T> for BiNumber<T> {
    type Output = Self;
    fn mul(self, other: T) -> Self {
        Self::new(self.x * other, self.y * other)
    }
}

// Allows for bi / 2 = (x/2,y/2)
impl<T: Copy + Div<Output = T>> Div<T> for BiNumber<T> {
    type Output = Self;
    fn div(self, other: T) -> Self {
        Self::new(self.x / other, self.y / other)
    }
}

impl<T: Copy + Add<Output = T>> AddAssign<T> for BiNumber<T> {
    fn add_assign(&mut self, other: T) {
        *self = *self + other;
    }
}

impl<T: Copy + Sub<Output = T>> SubAssign<T> for BiNumber<T> {
    fn sub_assign(&mut self, other: T) {
        *self = *self - other;
    }
}

impl<T: Copy + Mul<Output = T>> MulAssign<T> for BiNumber<T> {
    fn mul_assign(&mut self, other: T) {
        *self = *self * other;
    }
}

impl<T: Copy + Div<Output = T>> DivAssign<T> for BiNumber<T> {
    fn div_assign(&mut self, other: T) {
        *self = *self / other;
    }
}

impl<T: fmt::Display> fmt::Display for BiNumber<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<T> Index<usize> for BiNumber<T> {
    type Output = T;
    fn index(&self, axis: usize) -> &T {
        match axis {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("BiNumber index out of range: {axis}"),
        }
    }
}

impl<T> IndexMut<usize> for BiNumber<T> {
    fn index_mut(&mut self, axis: usize) -> &mut T {
        match axis {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("BiNumber index out of range: {axis}"),
        }
    }
}

impl<T: Copy + Mul<Output = T>> BiNumber<T> {
    pub fn total(self) -> T {
        self.x * self.y
    }
}

/// @brief A struct that contains three numbers, two 32-bit integers for x and z, and a variable-sized y (8, 16, 32, 64 bits)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SlimInt3<T> {
    pub x: i32,
    pub y: T,
    pub z: i32,
}

impl<T> SlimInt3<T> {
    pub const fn new(x: i32, y: T, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl<T: Copy + Add<Output = T>> Add for SlimInt3<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl<T: Copy + Sub<Output = T>> Sub for SlimInt3<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<T: Copy + Mul<Output = T>> Mul for SlimInt3<T> {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

impl<T: Copy + Div<Output = T>> Div for SlimInt3<T> {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        Self::new(self.x / other.x, self.y / other.y, self.z / other.z)
    }
}

impl<T: fmt::Display> fmt::Display for SlimInt3<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl<T: Copy + Mul<Output = T> + From<i32>> SlimInt3<T> {
    pub fn total(self) -> T {
        T::from(self.x) * self.y * T::from(self.z)
    }
}

/* --- Pre-defined Tri and Bi numbers --- */

// Vector/Double (64-Bit float)
pub type Vec3 = TriNumber<f64>;
pub type Double3 = Vec3;
pub type Vec2 = BiNumber<f64>;
pub type Double2 = Vec2;

pub const VEC3_ZERO: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const VEC3_ONE: Vec3 = Vec3::new(1.0, 1.0, 1.0);
pub const VEC2_ZERO: Vec2 = Vec2::new(0.0, 0.0);
pub const VEC2_ONE: Vec2 = Vec2::new(1.0, 1.0);

pub const DOUBLE3_ZERO: Double3 = Double3::new(0.0, 0.0, 0.0);
pub const DOUBLE3_ONE: Double3 = Double3::new(1.0, 1.0, 1.0);
pub const DOUBLE2_ZERO: Double2 = Double2::new(0.0, 0.0);
pub const DOUBLE2_ONE: Double2 = Double2::new(1.0, 1.0);

// Float (32-Bit float)
pub type Float3 = TriNumber<f32>;
pub type Float2 = BiNumber<f32>;

pub const FLOAT3_ZERO: Float3 = Float3::new(0.0, 0.0, 0.0);
pub const FLOAT3_ONE: Float3 = Float3::new(1.0, 1.0, 1.0);
pub const FLOAT2_ZERO: Float2 = Float2::new(0.0, 0.0);
pub const FLOAT2_ONE: Float2 = Float2::new(1.0, 1.0);

// 8-Bit Integer
pub type Int8_3 = TriNumber<i8>;
pub type Int8_2 = BiNumber<i8>;

pub const INT8_3_ZERO: Int8_3 = Int8_3::new(0, 0, 0);
pub const INT8_3_ONE: Int8_3 = Int8_3::new(1, 1, 1);
pub const INT8_2_ZERO: Int8_2 = Int8_2::new(0, 0);
pub const INT8_2_ONE: Int8_2 = Int8_2::new(1, 1);

pub type Byte3 = Int8_3;
pub type Byte2 = Int8_2;

pub const BYTE3_ZERO: Byte3 = INT8_3_ZERO;
pub const BYTE3_ONE: Byte3 = INT8_3_ONE;
pub const BYTE2_ZERO: Byte2 = INT8_2_ZERO;
pub const BYTE2_ONE: Byte2 = INT8_2_ONE;

// 16-Bit Integer
pub type Int16_3 = TriNumber<i16>;
pub type Int16_2 = BiNumber<i16>;

pub const INT16_3_ZERO: Int16_3 = Int16_3::new(0, 0, 0);
pub const INT16_3_ONE: Int16_3 = Int16_3::new(1, 1, 1);
pub const INT16_2_ZERO: Int16_2 = Int16_2::new(0, 0);
pub const INT16_2_ONE: Int16_2 = Int16_2::new(1, 1);

pub type Short3 = Int16_3;
pub type Short2 = Int16_2;

pub const SHORT3_ZERO: Short3 = INT16_3_ZERO;
pub const SHORT3_ONE: Short3 = INT16_3_ONE;
pub const SHORT2_ZERO: Short2 = INT16_2_ZERO;
pub const SHORT2_ONE: Short2 = INT16_2_ONE;

// 32-Bit Integer (default)
pub type Int32_3 = TriNumber<i32>;
pub type Int32_2 = BiNumber<i32>;

pub const INT32_3_ZERO: Int32_3 = Int32_3::new(0, 0, 0);
pub const INT32_3_ONE: Int32_3 = Int32_3::new(1, 1, 1);
pub const INT32_2_ZERO: Int32_2 = Int32_2::new(0, 0);
pub const INT32_2_ONE: Int32_2 = Int32_2::new(1, 1);

pub type Int3 = Int32_3;
pub type Int2 = Int32_2;

pub const INT3_ZERO: Int3 = INT32_3_ZERO;
pub const INT3_ONE: Int3 = INT32_3_ONE;
pub const INT2_ZERO: Int2 = INT32_2_ZERO;
pub const INT2_ONE: Int2 = INT32_2_ONE;

// 64-Bit Integer
pub type Int64_3 = TriNumber<i64>;
pub type Int64_2 = BiNumber<i64>;

pub const INT64_3_ZERO: Int64_3 = Int64_3::new(0, 0, 0);
pub const INT64_3_ONE: Int64_3 = Int64_3::new(1, 1, 1);
pub const INT64_2_ZERO: Int64_2 = Int64_2::new(0, 0);
pub const INT64_2_ONE: Int64_2 = Int64_2::new(1, 1);

pub type Long3 = Int64_3;
pub type Long2 = Int64_2;

pub const LONG3_ZERO: Long3 = INT64_3_ZERO;
pub const LONG3_ONE: Long3 = INT64_3_ONE;
pub const LONG2_ZERO: Long2 = INT64_2_ZERO;
pub const LONG2_ONE: Long2 = INT64_2_ONE;

// Unsigned 8-Bit Integer
pub type UInt8_3 = TriNumber<u8>;
pub type UInt8_2 = BiNumber<u8>;

pub const UINT8_3_ZERO: UInt8_3 = UInt8_3::new(0, 0, 0);
pub const UINT8_3_ONE: UInt8_3 = UInt8_3::new(1, 1, 1);
pub const UINT8_2_ZERO: UInt8_2 = UInt8_2::new(0, 0);
pub const UINT8_2_ONE: UInt8_2 = UInt8_2::new(1, 1);

pub type UByte3 = UInt8_3;
pub type UByte2 = UInt8_2;

pub const UBYTE3_ZERO: UByte3 = UINT8_3_ZERO;
pub const UBYTE3_ONE: UByte3 = UINT8_3_ONE;
pub const UBYTE2_ZERO: UByte2 = UINT8_2_ZERO;
pub const UBYTE2_ONE: UByte2 = UINT8_2_ONE;

// Unsigned 16-Bit Integer
pub type UInt16_3 = TriNumber<u16>;
pub type UInt16_2 = BiNumber<u16>;

pub const UINT16_3_ZERO: UInt16_3 = UInt16_3::new(0, 0, 0);
pub const UINT16_3_ONE: UInt16_3 = UInt16_3::new(1, 1, 1);
pub const UINT16_2_ZERO: UInt16_2 = UInt16_2::new(0, 0);
pub const UINT16_2_ONE: UInt16_2 = UInt16_2::new(1, 1);

pub type UShort3 = UInt16_3;
pub type UShort2 = UInt16_2;

pub const USHORT3_ZERO: UShort3 = UINT16_3_ZERO;
pub const USHORT3_ONE: UShort3 = UINT16_3_ONE;
pub const USHORT2_ZERO: UShort2 = UINT16_2_ZERO;
pub const USHORT2_ONE: UShort2 = UINT16_2_ONE;

// Unsigned 32-Bit Integer (default)
pub type UInt32_3 = TriNumber<u32>;
pub type UInt32_2 = BiNumber<u32>;

pub const UINT32_3_ZERO: UInt32_3 = UInt32_3::new(0, 0, 0);
pub const UINT32_3_ONE: UInt32_3 = UInt32_3::new(1, 1, 1);
pub const UINT32_2_ZERO: UInt32_2 = UInt32_2::new(0, 0);
pub const UINT32_2_ONE: UInt32_2 = UInt32_2::new(1, 1);

pub type UInt3 = UInt32_3;
pub type UInt2 = UInt32_2;

pub const UINT3_ZERO: UInt3 = UINT32_3_ZERO;
pub const UINT3_ONE: UInt3 = UINT32_3_ONE;
pub const UINT2_ZERO: UInt2 = UINT32_2_ZERO;
pub const UINT2_ONE: UInt2 = UINT32_2_ONE;

// Unsigned 64-Bit Integer
pub type UInt64_3 = TriNumber<u64>;
pub type UInt64_2 = BiNumber<u64>;

pub const UINT64_3_ZERO: UInt64_3 = UInt64_3::new(0, 0, 0);
pub const UINT64_3_ONE: UInt64_3 = UInt64_3::new(1, 1, 1);
pub const UINT64_2_ZERO: UInt64_2 = UInt64_2::new(0, 0);
pub const UINT64_2_ONE: UInt64_2 = UInt64_2::new(1, 1);

pub type ULong3 = Int64_3;
pub type ULong2 = Int64_2;

pub const ULONG3_ZERO: UInt64_3 = UINT64_3_ZERO;
pub const ULONG3_ONE: UInt64_3 = UINT64_3_ONE;
pub const ULONG2_ZERO: UInt64_2 = UINT64_2_ZERO;
pub const ULONG2_ONE: UInt64_2 = UINT64_2_ONE;

// Slim Int3 defines
pub const SLIM_INT3_ZERO: SlimInt3<i32> = SlimInt3::new(0, 0, 0);
pub const SLIM_INT3_ONE: SlimInt3<i32> = SlimInt3::new(1, 1, 1);
