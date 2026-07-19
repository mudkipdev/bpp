/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum EntityType {
    // Misc
    None,
    Item,
    Player,
    Fish,
    Fireball,
    // Objects
    Boat,
    Minecart,
    StorageMinecart,
    FurnaceMinecart,
    LitTnt,
    Arrow,
    ThrownSnowball,
    ThrownEgg,
    FallingSand,
    FallingGravel,
    FishingBobber,
    // Mobs
    Creeper,
    Skeleton,
    Spider,
    GiantZombie,
    Zombie,
    Slime,
    Ghast,
    ZombiePigman,
    Pig,
    Sheep,
    Cow,
    Chicken,
    Squid,
    Wolf,
    Painting,
}
