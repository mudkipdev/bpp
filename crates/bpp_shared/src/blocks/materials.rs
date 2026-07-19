/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

// Map colors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapColor {
    pub index: u8,
    pub color_value: u32, // packed RGB
}

impl MapColor {
    pub const fn air() -> Self {
        Self { index: 0, color_value: 0x000000 }
    }
    pub const fn grass() -> Self {
        Self { index: 1, color_value: 0x7FB238 }
    }
    pub const fn sand() -> Self {
        Self { index: 2, color_value: 0xF7E9A3 }
    }
    pub const fn cloth() -> Self {
        Self { index: 3, color_value: 0xA7A7A7 }
    }
    pub const fn tnt() -> Self {
        Self { index: 4, color_value: 0xFF0000 }
    }
    pub const fn ice() -> Self {
        Self { index: 5, color_value: 0xA0A0FF }
    }
    pub const fn iron() -> Self {
        Self { index: 6, color_value: 0xA7A7A7 }
    }
    pub const fn foliage() -> Self {
        Self { index: 7, color_value: 0x007C00 }
    }
    pub const fn snow() -> Self {
        Self { index: 8, color_value: 0xFFFFFF }
    }
    pub const fn clay() -> Self {
        Self { index: 9, color_value: 0xA4A8B8 }
    }
    pub const fn dirt() -> Self {
        Self { index: 10, color_value: 0xB7906F }
    }
    pub const fn stone() -> Self {
        Self { index: 11, color_value: 0x707070 }
    }
    pub const fn water() -> Self {
        Self { index: 12, color_value: 0x4040FF }
    }
    pub const fn wood() -> Self {
        Self { index: 13, color_value: 0x685432 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaterialType {
    Air,
    Grass,
    Ground, // dirt, gravel, clay
    Wood,
    Rock,
    Iron,
    Water,
    Lava,
    Leaves,
    Plants, // non-solid, no grass cover (flowers, saplings, etc)
    Sponge,
    Cloth, // wool, web
    Fire,
    Sand,
    Circuits, // redstone wire, rails, etc
    Glass,
    TNT,
    Coral,
    Ice,
    SnowLayer, // snow layer
    SnowBlock, // snow block
    Cactus,
    Clay,
    Pumpkin,
    Portal,
    Cake,
    Web,
    Piston,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PushabilityFlag {
    Normal = 0,
    NoPush = 1,
    Immovable = 2,
}

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub r#type: MaterialType,
    pub map_color: MapColor,
    pub is_liquid: bool,
    pub is_solid: bool,
    pub is_opaque: bool,
    pub can_burn: bool,
    pub is_ground_cover: bool,
    pub can_block_grass: bool,
    pub is_harvestable: bool,
    pub mobility_flag: PushabilityFlag,
}

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.r#type == other.r#type
    }
}

impl Eq for Material {}

impl Default for Material {
    fn default() -> Self {
        Self::rock()
    }
}

impl Material {
    pub const fn air() -> Self {
        Self {
            r#type: MaterialType::Air,
            map_color: MapColor::air(),
            is_liquid: false,
            is_solid: false,
            is_opaque: false,
            can_burn: false,
            is_ground_cover: true,
            can_block_grass: false,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::Normal,
        }
    }

    pub const fn grass() -> Self {
        Self {
            r#type: MaterialType::Grass,
            map_color: MapColor::grass(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::Normal,
        }
    }

    pub const fn ground() -> Self {
        Self {
            r#type: MaterialType::Ground,
            map_color: MapColor::dirt(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::Normal,
        }
    }

    pub const fn wood() -> Self {
        Self {
            r#type: MaterialType::Wood,
            map_color: MapColor::wood(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: true,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::Normal,
        }
    }

    pub const fn rock() -> Self {
        Self {
            r#type: MaterialType::Rock,
            map_color: MapColor::stone(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: false,
            mobility_flag: PushabilityFlag::Normal,
        }
    }

    pub const fn iron() -> Self {
        Self {
            r#type: MaterialType::Iron,
            map_color: MapColor::iron(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: false,
            mobility_flag: PushabilityFlag::Normal,
        }
    }

    pub const fn water() -> Self {
        Self {
            r#type: MaterialType::Water,
            map_color: MapColor::water(),
            is_liquid: true,
            is_solid: false,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: true,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::NoPush,
        }
    }

    pub const fn lava() -> Self {
        Self {
            r#type: MaterialType::Lava,
            map_color: MapColor::tnt(),
            is_liquid: true,
            is_solid: false,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: true,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::NoPush,
        }
    }

    pub const fn leaves() -> Self {
        Self {
            r#type: MaterialType::Leaves,
            map_color: MapColor::foliage(),
            is_liquid: false,
            is_solid: true,
            is_opaque: false,
            can_burn: true,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::NoPush,
        }
    }

    pub const fn plants() -> Self {
        Self {
            r#type: MaterialType::Plants,
            map_color: MapColor::foliage(),
            is_liquid: false,
            is_solid: false,
            is_opaque: false,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: false,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::NoPush,
        }
    }

    pub const fn sponge() -> Self {
        Self {
            r#type: MaterialType::Sponge,
            map_color: MapColor::cloth(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::Normal,
        }
    }

    pub const fn cloth() -> Self {
        Self {
            r#type: MaterialType::Cloth,
            map_color: MapColor::cloth(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: true,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::Normal,
        }
    }

    pub const fn fire() -> Self {
        Self {
            r#type: MaterialType::Fire,
            map_color: MapColor::air(),
            is_liquid: false,
            is_solid: false,
            is_opaque: false,
            can_burn: false,
            is_ground_cover: true,
            can_block_grass: false,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::NoPush,
        }
    }

    pub const fn sand() -> Self {
        Self {
            r#type: MaterialType::Sand,
            map_color: MapColor::sand(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::Normal,
        }
    }

    // MaterialLogic + setNoPushMobility
    pub const fn circuits() -> Self {
        Self {
            r#type: MaterialType::Circuits,
            map_color: MapColor::air(),
            is_liquid: false,
            is_solid: false,
            is_opaque: false,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: false,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::NoPush,
        }
    }

    pub const fn glass() -> Self {
        Self {
            r#type: MaterialType::Glass,
            map_color: MapColor::air(),
            is_liquid: false,
            is_solid: true,
            is_opaque: false,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::Normal,
        }
    }

    pub const fn tnt() -> Self {
        Self {
            r#type: MaterialType::TNT,
            map_color: MapColor::tnt(),
            is_liquid: false,
            is_solid: true,
            is_opaque: false,
            can_burn: true,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::Normal,
        }
    }

    pub const fn coral() -> Self {
        Self {
            r#type: MaterialType::Coral,
            map_color: MapColor::foliage(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::NoPush,
        }
    }

    pub const fn ice() -> Self {
        Self {
            r#type: MaterialType::Ice,
            map_color: MapColor::ice(),
            is_liquid: false,
            is_solid: true,
            is_opaque: false,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::Normal,
        }
    }

    pub const fn snow_layer() -> Self {
        Self {
            r#type: MaterialType::SnowLayer,
            map_color: MapColor::snow(),
            is_liquid: false,
            is_solid: false,
            is_opaque: false,
            can_burn: false,
            is_ground_cover: true,
            can_block_grass: false,
            is_harvestable: false,
            mobility_flag: PushabilityFlag::NoPush,
        }
    }

    pub const fn snow_block() -> Self {
        Self {
            r#type: MaterialType::SnowBlock,
            map_color: MapColor::snow(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: false,
            mobility_flag: PushabilityFlag::Normal,
        }
    }

    pub const fn cactus() -> Self {
        Self {
            r#type: MaterialType::Cactus,
            map_color: MapColor::foliage(),
            is_liquid: false,
            is_solid: true,
            is_opaque: false,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::NoPush,
        }
    }

    pub const fn clay() -> Self {
        Self {
            r#type: MaterialType::Clay,
            map_color: MapColor::clay(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::Normal,
        }
    }

    pub const fn pumpkin() -> Self {
        Self {
            r#type: MaterialType::Pumpkin,
            map_color: MapColor::foliage(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::NoPush,
        }
    }

    pub const fn portal() -> Self {
        Self {
            r#type: MaterialType::Portal,
            map_color: MapColor::air(),
            is_liquid: false,
            is_solid: false,
            is_opaque: false,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: false,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::Immovable,
        }
    }

    pub const fn cake() -> Self {
        Self {
            r#type: MaterialType::Cake,
            map_color: MapColor::air(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: true,
            mobility_flag: PushabilityFlag::NoPush,
        }
    }

    pub const fn web() -> Self {
        Self {
            r#type: MaterialType::Web,
            map_color: MapColor::cloth(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: false,
            mobility_flag: PushabilityFlag::NoPush,
        }
    }

    pub const fn piston() -> Self {
        Self {
            r#type: MaterialType::Piston,
            map_color: MapColor::stone(),
            is_liquid: false,
            is_solid: true,
            is_opaque: true,
            can_burn: false,
            is_ground_cover: false,
            can_block_grass: true,
            is_harvestable: false,
            mobility_flag: PushabilityFlag::Immovable,
        }
    }
}
