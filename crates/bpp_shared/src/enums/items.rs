/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
*/

use crate::base_types::ItemId;

// Items above this ID are pure items (not placeable blocks)
pub const THRESHOLD: i16 = 255;
// Maximum number of items in a stack
pub const STACK_MAX: i8 = 64;

pub const INVALID: ItemId = ItemId(-1);
pub const NONE: ItemId = ItemId(0); // This is usually not you want to use, use INVALID instead

// Tools - Iron
pub const SHOVEL_IRON: ItemId = ItemId(256);
pub const PICKAXE_IRON: ItemId = ItemId(257);
pub const AXE_IRON: ItemId = ItemId(258);
pub const FLINT_AND_STEEL: ItemId = ItemId(259);

// Food
pub const APPLE: ItemId = ItemId(260);

// Combat
pub const BOW: ItemId = ItemId(261);
pub const ARROW: ItemId = ItemId(262);

// Resources
pub const COAL: ItemId = ItemId(263);
pub const DIAMOND: ItemId = ItemId(264);
pub const IRON: ItemId = ItemId(265);
pub const GOLD: ItemId = ItemId(266);

// Tools/Weapons - Iron
pub const SWORD_IRON: ItemId = ItemId(267);

// Tools/Weapons - Wood
pub const SWORD_WOOD: ItemId = ItemId(268);
pub const SHOVEL_WOOD: ItemId = ItemId(269);
pub const PICKAXE_WOOD: ItemId = ItemId(270);
pub const AXE_WOOD: ItemId = ItemId(271);

// Tools/Weapons - Stone
pub const SWORD_STONE: ItemId = ItemId(272);
pub const SHOVEL_STONE: ItemId = ItemId(273);
pub const PICKAXE_STONE: ItemId = ItemId(274);
pub const AXE_STONE: ItemId = ItemId(275);

// Tools/Weapons - Diamond
pub const SWORD_DIAMOND: ItemId = ItemId(276);
pub const SHOVEL_DIAMOND: ItemId = ItemId(277);
pub const PICKAXE_DIAMOND: ItemId = ItemId(278);
pub const AXE_DIAMOND: ItemId = ItemId(279);

// Misc
pub const STICK: ItemId = ItemId(280);
pub const BOWL: ItemId = ItemId(281);
pub const MUSHROOM_STEW: ItemId = ItemId(282);

// Tools/Weapons - Gold
pub const SWORD_GOLD: ItemId = ItemId(283);
pub const SHOVEL_GOLD: ItemId = ItemId(284);
pub const PICKAXE_GOLD: ItemId = ItemId(285);
pub const AXE_GOLD: ItemId = ItemId(286);

// Resources
pub const STRING: ItemId = ItemId(287);
pub const FEATHER: ItemId = ItemId(288);
pub const GUNPOWDER: ItemId = ItemId(289);

// Hoes
pub const HOE_WOOD: ItemId = ItemId(290);
pub const HOE_STONE: ItemId = ItemId(291);
pub const HOE_IRON: ItemId = ItemId(292);
pub const HOE_DIAMOND: ItemId = ItemId(293);
pub const HOE_GOLD: ItemId = ItemId(294);

// Farming
pub const SEEDS_WHEAT: ItemId = ItemId(295);
pub const WHEAT: ItemId = ItemId(296);

// Food
pub const BREAD: ItemId = ItemId(297);

// Armor - Leather
pub const HELMET_LEATHER: ItemId = ItemId(298);
pub const CHESTPLATE_LEATHER: ItemId = ItemId(299);
pub const LEGGINGS_LEATHER: ItemId = ItemId(300);
pub const BOOTS_LEATHER: ItemId = ItemId(301);

// Armor - Chainmail
pub const HELMET_CHAINMAIL: ItemId = ItemId(302);
pub const CHESTPLATE_CHAINMAIL: ItemId = ItemId(303);
pub const LEGGINGS_CHAINMAIL: ItemId = ItemId(304);
pub const BOOTS_CHAINMAIL: ItemId = ItemId(305);

// Armor - Iron
pub const HELMET_IRON: ItemId = ItemId(306);
pub const CHESTPLATE_IRON: ItemId = ItemId(307);
pub const LEGGINGS_IRON: ItemId = ItemId(308);
pub const BOOTS_IRON: ItemId = ItemId(309);

// Armor - Diamond
pub const HELMET_DIAMOND: ItemId = ItemId(310);
pub const CHESTPLATE_DIAMOND: ItemId = ItemId(311);
pub const LEGGINGS_DIAMOND: ItemId = ItemId(312);
pub const BOOTS_DIAMOND: ItemId = ItemId(313);

// Armor - Gold
pub const HELMET_GOLD: ItemId = ItemId(314);
pub const CHESTPLATE_GOLD: ItemId = ItemId(315);
pub const LEGGINGS_GOLD: ItemId = ItemId(316);
pub const BOOTS_GOLD: ItemId = ItemId(317);

// Resources/Food
pub const FLINT: ItemId = ItemId(318);
pub const PORKCHOP: ItemId = ItemId(319);
pub const PORKCHOP_COOKED: ItemId = ItemId(320);
pub const PAINTING: ItemId = ItemId(321);
pub const APPLE_GOLDEN: ItemId = ItemId(322);

// Placeable items
pub const SIGN: ItemId = ItemId(323);
pub const DOOR_WOOD: ItemId = ItemId(324);

// Buckets
pub const BUCKET: ItemId = ItemId(325);
pub const BUCKET_WATER: ItemId = ItemId(326);
pub const BUCKET_LAVA: ItemId = ItemId(327);

// Vehicles
pub const MINECART: ItemId = ItemId(328);
pub const SADDLE: ItemId = ItemId(329);

// Misc
pub const DOOR_IRON: ItemId = ItemId(330);
pub const REDSTONE: ItemId = ItemId(331);
pub const SNOWBALL: ItemId = ItemId(332);
pub const BOAT: ItemId = ItemId(333);
pub const LEATHER: ItemId = ItemId(334);
pub const BUCKET_MILK: ItemId = ItemId(335);
pub const BRICK: ItemId = ItemId(336);
pub const CLAY: ItemId = ItemId(337);
pub const SUGARCANE: ItemId = ItemId(338);
pub const PAPER: ItemId = ItemId(339);
pub const BOOK: ItemId = ItemId(340);
pub const SLIME: ItemId = ItemId(341);
pub const MINECART_CHEST: ItemId = ItemId(342);
pub const MINECART_FURNACE: ItemId = ItemId(343);
pub const EGG: ItemId = ItemId(344);
pub const COMPASS: ItemId = ItemId(345);
pub const FISHING_ROD: ItemId = ItemId(346);
pub const CLOCK: ItemId = ItemId(347);
pub const GLOWSTONE_DUST: ItemId = ItemId(348);
pub const FISH: ItemId = ItemId(349);
pub const FISH_COOKED: ItemId = ItemId(350);
pub const DYE: ItemId = ItemId(351);
pub const BONE: ItemId = ItemId(352);
pub const SUGAR: ItemId = ItemId(353);
pub const CAKE: ItemId = ItemId(354);
pub const BED: ItemId = ItemId(355);
pub const REDSTONE_REPEATER: ItemId = ItemId(356);
pub const COOKIE: ItemId = ItemId(357);
pub const MAP: ItemId = ItemId(358);
pub const SHEARS: ItemId = ItemId(359);
pub const MAX: ItemId = ItemId(360);

// Records
pub const RECORD_13: ItemId = ItemId(2256);
pub const RECORD_CAT: ItemId = ItemId(2257);
pub const RECORD_MAX: ItemId = ItemId(2258);
