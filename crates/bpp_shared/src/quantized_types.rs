/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::fmt;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use crate::numeric_structs::TriNumber;

macro_rules! fixed {
    ($name:ident, $storage:ty, $wide:ty) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name<const SCALE: $storage> {
            pub value: $storage,
        }

        impl<const SCALE: $storage> $name<SCALE> {
            pub const SCALE: $storage = SCALE;

            pub fn new(value: f64) -> Self {
                Self {
                    value: (value * SCALE as f64) as $storage,
                }
            }

            pub const fn from_raw(raw: $storage) -> Self {
                Self { value: raw }
            }

            pub const fn raw(self) -> $storage {
                self.value
            }

            pub fn value(self) -> f64 {
                self.value as f64 / SCALE as f64
            }
        }

        impl<const SCALE: $storage> Add for $name<SCALE> {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                Self::from_raw(self.value.wrapping_add(other.value))
            }
        }

        impl<const SCALE: $storage> Sub for $name<SCALE> {
            type Output = Self;
            fn sub(self, other: Self) -> Self {
                Self::from_raw(self.value.wrapping_sub(other.value))
            }
        }

        impl<const SCALE: $storage> Neg for $name<SCALE> {
            type Output = Self;
            fn neg(self) -> Self {
                Self::from_raw(self.value.wrapping_neg())
            }
        }

        impl<const SCALE: $storage> AddAssign for $name<SCALE> {
            fn add_assign(&mut self, other: Self) {
                *self = *self + other;
            }
        }

        impl<const SCALE: $storage> SubAssign for $name<SCALE> {
            fn sub_assign(&mut self, other: Self) {
                *self = *self - other;
            }
        }

        impl<const SCALE: $storage> std::ops::Mul for $name<SCALE> {
            type Output = Self;
            fn mul(self, other: Self) -> Self {
                Self::from_raw(
                    ((self.value as $wide * other.value as $wide) / SCALE as $wide) as $storage,
                )
            }
        }

        impl<const SCALE: $storage> std::ops::Div for $name<SCALE> {
            type Output = Self;
            fn div(self, other: Self) -> Self {
                Self::from_raw(
                    ((self.value as $wide * SCALE as $wide) / other.value as $wide) as $storage,
                )
            }
        }

        impl<const SCALE: $storage> fmt::Display for $name<SCALE> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.value())
            }
        }
    };
}

fixed!(FixedI8, i8, i64);
fixed!(FixedI16, i16, i64);
fixed!(FixedI32, i32, i64);

pub type NetworkEntityOffset = TriNumber<FixedI8<32>>;
pub type NetworkEntityPosition = TriNumber<FixedI32<32>>;
pub type NetworkEntityVelocity = TriNumber<FixedI16<8000>>;
