/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

// zero-cost wrapper that gives integers a distinct type
macro_rules! branded {
    ($name:ident, $underlying:ty) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub $underlying);

        impl $name {
            pub const fn value(self) -> $underlying {
                self.0
            }
        }

        impl From<$underlying> for $name {
            fn from(value: $underlying) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $underlying {
            fn from(branded: $name) -> Self {
                branded.0
            }
        }
    };
}

// stuff that needs arithmetic like ticks and item amount/damage needs to stay unbranded
pub type TickTime = i64;
pub type ItemAmount = i8;
pub type ItemDamage = i16;

branded!(EntityId, i32);
branded!(ItemId, i16);
branded!(MapId, i16);
branded!(WindowId, i8);
branded!(TransactionId, i16);

// protocol/nbt slots are numbered differently
branded!(NetworkSlotId, i16);
branded!(NbtSlotId, i8);

const _: () = assert!(size_of::<EntityId>() == size_of::<i32>());
const _: () = assert!(size_of::<ItemId>() == size_of::<i16>());
