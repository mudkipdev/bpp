/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::base_types::{ItemAmount, ItemDamage, ItemId};
use crate::entities::entity::Entity;
use crate::enums::blocks::{BLOCK_DIRT, BLOCK_FARMLAND, BLOCK_GRASS, BlockType};
use crate::enums::items;
use crate::numeric_structs::Int3;
use crate::world::world::WorldManager;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolLevel {
    None = -1,
    Wooden = 0,
    Stone = 1,
    Iron = 2,
    Diamond = 3,
}

impl ToolLevel {
    #[allow(non_upper_case_globals)]
    pub const Gold: ToolLevel = ToolLevel::Wooden;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolType {
    None = -1,
    Hoe = 0,
    Shovel = 1,
    Pickaxe = 2,
    Axe = 3,
    Sword = 4,
}

#[derive(Clone, Copy, Debug)]
pub struct ToolProperties {
    pub r#type: ToolType,
    pub level: ToolLevel,
    pub max_damage: ItemDamage,
}

impl Default for ToolProperties {
    fn default() -> Self {
        ToolProperties {
            r#type: ToolType::None,
            level: ToolLevel::None,
            max_damage: -1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ItemProperties {
    pub max_stack: ItemAmount,
}

impl Default for ItemProperties {
    fn default() -> Self {
        ItemProperties {
            max_stack: items::STACK_MAX,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ItemBehavior {
    pub on_block_start_mining: Option<fn(&mut WorldManager, Int3)>,
    pub on_block_stop_mining: Option<fn(&mut WorldManager, Int3)>,
    pub on_block_use: Option<fn(&mut WorldManager, Int3)>,
    pub on_entity_attack: Option<fn(&mut Entity)>,
    pub on_entity_use: Option<fn(&mut Entity)>,
}

// Tool material durability
const DURABILITY_WOOD: ItemDamage = 59;
const DURABILITY_STONE: ItemDamage = 131;
const DURABILITY_IRON: ItemDamage = 250;
const DURABILITY_DIAMOND: ItemDamage = 1561;
const DURABILITY_GOLD: ItemDamage = 32;

// Armor max damage = maxDamageArray[slot] * 3 << material
// material: leather=0, chain/gold=1, iron=2, diamond=3
// slot: helmet=0(x11), chest=1(x16), legs=2(x15), boots=3(x13)
const DURABILITY_HELMET_LEATHER: ItemDamage = 11 * 3 * 1; // 33
const DURABILITY_CHEST_LEATHER: ItemDamage = 16 * 3 * 1; // 48
const DURABILITY_LEGS_LEATHER: ItemDamage = 15 * 3 * 1; // 45
const DURABILITY_BOOTS_LEATHER: ItemDamage = 13 * 3 * 1; // 39

const DURABILITY_HELMET_CHAINMAIL: ItemDamage = 11 * 3 * 2; // 66
const DURABILITY_CHEST_CHAINMAIL: ItemDamage = 16 * 3 * 2; // 96
const DURABILITY_LEGS_CHAINMAIL: ItemDamage = 15 * 3 * 2; // 90
const DURABILITY_BOOTS_CHAINMAIL: ItemDamage = 13 * 3 * 2; // 78

const DURABILITY_HELMET_IRON: ItemDamage = 11 * 3 * 4; // 132
const DURABILITY_CHEST_IRON: ItemDamage = 16 * 3 * 4; // 192
const DURABILITY_LEGS_IRON: ItemDamage = 15 * 3 * 4; // 180
const DURABILITY_BOOTS_IRON: ItemDamage = 13 * 3 * 4; // 156

const DURABILITY_HELMET_DIAMOND: ItemDamage = 11 * 3 * 8; // 264
const DURABILITY_CHEST_DIAMOND: ItemDamage = 16 * 3 * 8; // 384
const DURABILITY_LEGS_DIAMOND: ItemDamage = 15 * 3 * 8; // 360
const DURABILITY_BOOTS_DIAMOND: ItemDamage = 13 * 3 * 8; // 312

const DURABILITY_HELMET_GOLD: ItemDamage = 11 * 3 * 2; // 66
const DURABILITY_CHEST_GOLD: ItemDamage = 16 * 3 * 2; // 96
const DURABILITY_LEGS_GOLD: ItemDamage = 15 * 3 * 2; // 90
const DURABILITY_BOOTS_GOLD: ItemDamage = 13 * 3 * 2; // 78

const DURABILITY_FISHING_ROD: ItemDamage = 64;
const DURABILITY_FLINT_AND_STEEL: ItemDamage = 64;
const DURABILITY_SHEARS: ItemDamage = 238;
const DURABILITY_BOW: ItemDamage = 384;

// Global table definitions; declared extern in the header
static ITEM_BEHAVIOR: OnceLock<Mutex<HashMap<ItemId, ItemBehavior>>> = OnceLock::new();
static ITEM_PROPERTIES: OnceLock<Mutex<HashMap<ItemId, ItemProperties>>> = OnceLock::new();
static TOOL_PROPERTIES: OnceLock<Mutex<HashMap<ItemId, ToolProperties>>> = OnceLock::new();

pub fn item_behavior() -> &'static Mutex<HashMap<ItemId, ItemBehavior>> {
    ITEM_BEHAVIOR.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn item_properties() -> &'static Mutex<HashMap<ItemId, ItemProperties>> {
    ITEM_PROPERTIES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn tool_properties() -> &'static Mutex<HashMap<ItemId, ToolProperties>> {
    TOOL_PROPERTIES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn is_valid(id: ItemId) -> bool {
    (id >= items::SHOVEL_IRON && id < items::MAX) || (id >= items::RECORD_13 && id < items::RECORD_MAX)
}

pub fn is_armor(id: ItemId) -> bool {
    id >= items::HELMET_LEATHER && id <= items::BOOTS_GOLD
}

pub fn is_hoe(id: ItemId) -> bool {
    id >= items::HOE_WOOD && id <= items::HOE_GOLD
}

pub fn is_sword(id: ItemId) -> bool {
    id == items::SWORD_IRON
        || id == items::SWORD_WOOD
        || id == items::SWORD_STONE
        || id == items::SWORD_DIAMOND
        || id == items::SWORD_GOLD
}

pub fn is_pickaxe(id: ItemId) -> bool {
    id == items::PICKAXE_IRON
        || id == items::PICKAXE_WOOD
        || id == items::PICKAXE_STONE
        || id == items::PICKAXE_DIAMOND
        || id == items::PICKAXE_GOLD
}

pub fn is_axe(id: ItemId) -> bool {
    id == items::AXE_IRON
        || id == items::AXE_WOOD
        || id == items::AXE_STONE
        || id == items::AXE_DIAMOND
        || id == items::AXE_GOLD
}

pub fn is_shovel(id: ItemId) -> bool {
    id == items::SHOVEL_IRON
        || id == items::SHOVEL_WOOD
        || id == items::SHOVEL_STONE
        || id == items::SHOVEL_DIAMOND
        || id == items::SHOVEL_GOLD
}

pub fn is_weapon(id: ItemId) -> bool {
    is_sword(id) || id == items::BOW
}

pub fn is_tool(id: ItemId) -> bool {
    is_shovel(id) || is_axe(id) || is_pickaxe(id) || is_hoe(id) || id == items::FLINT_AND_STEEL
        || id == items::FISHING_ROD
        || id == items::SHEARS
}

pub fn is_throwable(id: ItemId) -> bool {
    id == items::SNOWBALL || id == items::EGG
}

pub fn is_block(id: ItemId) -> bool {
    id.value() > 0 && id.value() <= items::THRESHOLD
}

// max stack > 1
pub fn is_stackable(id: ItemId) -> bool {
    get_max_stack(id) > 1
}

// Returns max stack size for this item/block id
pub fn get_max_stack(id: ItemId) -> i32 {
    // Stack size 1
    match id {
        // Food (ItemFood sets maxStackSize=1 in constructor)
        items::APPLE
        | items::APPLE_GOLDEN
        | items::BREAD
        | items::PORKCHOP
        | items::PORKCHOP_COOKED
        | items::FISH
        | items::FISH_COOKED
        | items::MUSHROOM_STEW // ItemSoup extends ItemFood

        // Containers / vehicles / misc unstackables
        | items::CAKE // ItemReed.setMaxStackSize(1)
        | items::BED // ItemBed.setMaxStackSize(1)
        | items::SADDLE
        | items::BUCKET
        | items::BUCKET_WATER
        | items::BUCKET_LAVA
        | items::BUCKET_MILK
        | items::MINECART
        | items::MINECART_CHEST
        | items::MINECART_FURNACE
        | items::BOAT
        | items::DOOR_WOOD
        | items::DOOR_IRON
        | items::SIGN // ItemSign
        | items::MAP // ItemMap.setMaxStackSize(1)
        | items::RECORD_13 // ItemRecord
        | items::RECORD_CAT => return 1, // ItemRecord

        _ => {}
    }

    // Tools, weapons, armor all set maxStackSize=1 in their constructors
    if is_tool(id) || is_weapon(id) || is_armor(id) {
        return 1;
    }

    // Stack size 16
    if id == items::SNOWBALL || id == items::EGG {
        return 16;
    }

    if id == items::COOKIE {
        return 8;
    }

    // Item, ItemCoal, ItemSeeds, ItemRedstone, ItemDye, ItemPainting,
    // ItemReed (sugarcane & repeater item), ItemRecord (never reached above),
    // all blocks, and any resource item not listed above.
    items::STACK_MAX as i32
}

// Returns max durability (0 = not damageable)
pub fn get_max_durability(id: ItemId) -> ItemDamage {
    match id {
        // Swords
        items::SWORD_WOOD => DURABILITY_WOOD,
        items::SWORD_STONE => DURABILITY_STONE,
        items::SWORD_IRON => DURABILITY_IRON,
        items::SWORD_DIAMOND => DURABILITY_DIAMOND,
        items::SWORD_GOLD => DURABILITY_GOLD,

        // Shovels
        items::SHOVEL_WOOD => DURABILITY_WOOD,
        items::SHOVEL_STONE => DURABILITY_STONE,
        items::SHOVEL_IRON => DURABILITY_IRON,
        items::SHOVEL_DIAMOND => DURABILITY_DIAMOND,
        items::SHOVEL_GOLD => DURABILITY_GOLD,

        // Pickaxes
        items::PICKAXE_WOOD => DURABILITY_WOOD,
        items::PICKAXE_STONE => DURABILITY_STONE,
        items::PICKAXE_IRON => DURABILITY_IRON,
        items::PICKAXE_DIAMOND => DURABILITY_DIAMOND,
        items::PICKAXE_GOLD => DURABILITY_GOLD,

        // Axes
        items::AXE_WOOD => DURABILITY_WOOD,
        items::AXE_STONE => DURABILITY_STONE,
        items::AXE_IRON => DURABILITY_IRON,
        items::AXE_DIAMOND => DURABILITY_DIAMOND,
        items::AXE_GOLD => DURABILITY_GOLD,

        // Hoes
        items::HOE_WOOD => DURABILITY_WOOD,
        items::HOE_STONE => DURABILITY_STONE,
        items::HOE_IRON => DURABILITY_IRON,
        items::HOE_DIAMOND => DURABILITY_DIAMOND,
        items::HOE_GOLD => DURABILITY_GOLD,

        // Armor - Leather
        items::HELMET_LEATHER => DURABILITY_HELMET_LEATHER,
        items::CHESTPLATE_LEATHER => DURABILITY_CHEST_LEATHER,
        items::LEGGINGS_LEATHER => DURABILITY_LEGS_LEATHER,
        items::BOOTS_LEATHER => DURABILITY_BOOTS_LEATHER,

        // Armor - Chainmail
        items::HELMET_CHAINMAIL => DURABILITY_HELMET_CHAINMAIL,
        items::CHESTPLATE_CHAINMAIL => DURABILITY_CHEST_CHAINMAIL,
        items::LEGGINGS_CHAINMAIL => DURABILITY_LEGS_CHAINMAIL,
        items::BOOTS_CHAINMAIL => DURABILITY_BOOTS_CHAINMAIL,

        // Armor - Iron
        items::HELMET_IRON => DURABILITY_HELMET_IRON,
        items::CHESTPLATE_IRON => DURABILITY_CHEST_IRON,
        items::LEGGINGS_IRON => DURABILITY_LEGS_IRON,
        items::BOOTS_IRON => DURABILITY_BOOTS_IRON,

        // Armor - Diamond
        items::HELMET_DIAMOND => DURABILITY_HELMET_DIAMOND,
        items::CHESTPLATE_DIAMOND => DURABILITY_CHEST_DIAMOND,
        items::LEGGINGS_DIAMOND => DURABILITY_LEGS_DIAMOND,
        items::BOOTS_DIAMOND => DURABILITY_BOOTS_DIAMOND,

        // Armor - Gold
        items::HELMET_GOLD => DURABILITY_HELMET_GOLD,
        items::CHESTPLATE_GOLD => DURABILITY_CHEST_GOLD,
        items::LEGGINGS_GOLD => DURABILITY_LEGS_GOLD,
        items::BOOTS_GOLD => DURABILITY_BOOTS_GOLD,

        // Misc damageable
        items::FLINT_AND_STEEL => DURABILITY_FLINT_AND_STEEL,
        items::FISHING_ROD => DURABILITY_FISHING_ROD,
        items::SHEARS => DURABILITY_SHEARS,
        items::BOW => DURABILITY_BOW,

        _ => 0, // not damageable
    }
}

fn use_hoe(world: &mut WorldManager, pos: Int3) {
    let b: BlockType = world.get_block_id(pos);
    if b == BLOCK_GRASS || b == BLOCK_DIRT {
        world.set_block(pos, BLOCK_FARMLAND, 0);
    }
}

pub fn register_all() {
    let mut behavior = item_behavior().lock().unwrap();
    behavior.insert(
        items::HOE_WOOD,
        ItemBehavior {
            on_block_use: Some(use_hoe),
            ..Default::default()
        },
    );
    behavior.insert(
        items::HOE_STONE,
        ItemBehavior {
            on_block_use: Some(use_hoe),
            ..Default::default()
        },
    );
    behavior.insert(
        items::HOE_IRON,
        ItemBehavior {
            on_block_use: Some(use_hoe),
            ..Default::default()
        },
    );
    behavior.insert(
        items::HOE_GOLD,
        ItemBehavior {
            on_block_use: Some(use_hoe),
            ..Default::default()
        },
    );
    behavior.insert(
        items::HOE_DIAMOND,
        ItemBehavior {
            on_block_use: Some(use_hoe),
            ..Default::default()
        },
    );
}
