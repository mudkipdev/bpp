/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

pub const MAX_CROP_SIZE: i32 = 7;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockType(pub i8);

pub const BLOCK_INVALID: BlockType = BlockType(-1);
pub const BLOCK_AIR: BlockType = BlockType(0);
pub const BLOCK_STONE: BlockType = BlockType(1);
pub const BLOCK_GRASS: BlockType = BlockType(2);
pub const BLOCK_DIRT: BlockType = BlockType(3);
pub const BLOCK_COBBLESTONE: BlockType = BlockType(4);
pub const BLOCK_PLANKS: BlockType = BlockType(5);
pub const BLOCK_SAPLING: BlockType = BlockType(6);
pub const BLOCK_BEDROCK: BlockType = BlockType(7);
pub const BLOCK_WATER_FLOWING: BlockType = BlockType(8);
pub const BLOCK_WATER_STILL: BlockType = BlockType(9);
pub const BLOCK_LAVA_FLOWING: BlockType = BlockType(10);
pub const BLOCK_LAVA_STILL: BlockType = BlockType(11);
pub const BLOCK_SAND: BlockType = BlockType(12);
pub const BLOCK_GRAVEL: BlockType = BlockType(13);
pub const BLOCK_ORE_GOLD: BlockType = BlockType(14);
pub const BLOCK_ORE_IRON: BlockType = BlockType(15);
pub const BLOCK_ORE_COAL: BlockType = BlockType(16);
pub const BLOCK_LOG: BlockType = BlockType(17);
pub const BLOCK_LEAVES: BlockType = BlockType(18);
pub const BLOCK_SPONGE: BlockType = BlockType(19);
pub const BLOCK_GLASS: BlockType = BlockType(20);
pub const BLOCK_ORE_LAPIS_LAZULI: BlockType = BlockType(21);
pub const BLOCK_LAPIS_LAZULI: BlockType = BlockType(22);
pub const BLOCK_DISPENSER: BlockType = BlockType(23);
pub const BLOCK_SANDSTONE: BlockType = BlockType(24);
pub const BLOCK_NOTEBLOCK: BlockType = BlockType(25);
pub const BLOCK_BED: BlockType = BlockType(26);
pub const BLOCK_RAIL_POWERED: BlockType = BlockType(27);
pub const BLOCK_RAIL_DETECTOR: BlockType = BlockType(28);
pub const BLOCK_PISTON_STICKY: BlockType = BlockType(29);
pub const BLOCK_COBWEB: BlockType = BlockType(30);
pub const BLOCK_TALLGRASS: BlockType = BlockType(31);
pub const BLOCK_DEADBUSH: BlockType = BlockType(32);
pub const BLOCK_PISTON: BlockType = BlockType(33);
pub const BLOCK_PISTON_HEAD: BlockType = BlockType(34);
pub const BLOCK_WOOL: BlockType = BlockType(35);
// not a real block, used for piston animation
pub const BLOCK_PISTON_MOVING: BlockType = BlockType(36);
pub const BLOCK_DANDELION: BlockType = BlockType(37);
pub const BLOCK_ROSE: BlockType = BlockType(38);
pub const BLOCK_MUSHROOM_BROWN: BlockType = BlockType(39);
pub const BLOCK_MUSHROOM_RED: BlockType = BlockType(40);
pub const BLOCK_GOLD: BlockType = BlockType(41);
pub const BLOCK_IRON: BlockType = BlockType(42);
pub const BLOCK_DOUBLE_SLAB: BlockType = BlockType(43);
pub const BLOCK_SLAB: BlockType = BlockType(44);
pub const BLOCK_BRICKS: BlockType = BlockType(45);
pub const BLOCK_TNT: BlockType = BlockType(46);
pub const BLOCK_BOOKSHELF: BlockType = BlockType(47);
pub const BLOCK_COBBLESTONE_MOSSY: BlockType = BlockType(48);
pub const BLOCK_OBSIDIAN: BlockType = BlockType(49);
pub const BLOCK_TORCH: BlockType = BlockType(50);
pub const BLOCK_FIRE: BlockType = BlockType(51);
pub const BLOCK_MOB_SPAWNER: BlockType = BlockType(52);
pub const BLOCK_STAIRS_WOOD: BlockType = BlockType(53);
pub const BLOCK_CHEST: BlockType = BlockType(54);
pub const BLOCK_REDSTONE: BlockType = BlockType(55);
pub const BLOCK_ORE_DIAMOND: BlockType = BlockType(56);
pub const BLOCK_DIAMOND: BlockType = BlockType(57);
pub const BLOCK_CRAFTING_TABLE: BlockType = BlockType(58);
pub const BLOCK_CROP_WHEAT: BlockType = BlockType(59);
pub const BLOCK_FARMLAND: BlockType = BlockType(60);
pub const BLOCK_FURNACE: BlockType = BlockType(61);
pub const BLOCK_FURNACE_LIT: BlockType = BlockType(62);
pub const BLOCK_SIGN: BlockType = BlockType(63);
pub const BLOCK_DOOR_WOOD: BlockType = BlockType(64);
pub const BLOCK_LADDER: BlockType = BlockType(65);
pub const BLOCK_RAIL: BlockType = BlockType(66);
pub const BLOCK_STAIRS_COBBLESTONE: BlockType = BlockType(67);
pub const BLOCK_SIGN_WALL: BlockType = BlockType(68);
pub const BLOCK_LEVER: BlockType = BlockType(69);
pub const BLOCK_PRESSURE_PLATE_STONE: BlockType = BlockType(70);
pub const BLOCK_DOOR_IRON: BlockType = BlockType(71);
pub const BLOCK_PRESSURE_PLATE_WOOD: BlockType = BlockType(72);
pub const BLOCK_ORE_REDSTONE_OFF: BlockType = BlockType(73);
pub const BLOCK_ORE_REDSTONE_ON: BlockType = BlockType(74);
pub const BLOCK_REDSTONE_TORCH_OFF: BlockType = BlockType(75);
pub const BLOCK_REDSTONE_TORCH_ON: BlockType = BlockType(76);
pub const BLOCK_BUTTON_STONE: BlockType = BlockType(77);
pub const BLOCK_SNOW_LAYER: BlockType = BlockType(78);
pub const BLOCK_ICE: BlockType = BlockType(79);
pub const BLOCK_SNOW: BlockType = BlockType(80);
pub const BLOCK_CACTUS: BlockType = BlockType(81);
pub const BLOCK_CLAY: BlockType = BlockType(82);
pub const BLOCK_SUGARCANE: BlockType = BlockType(83);
pub const BLOCK_JUKEBOX: BlockType = BlockType(84);
pub const BLOCK_FENCE: BlockType = BlockType(85);
pub const BLOCK_PUMPKIN: BlockType = BlockType(86);
pub const BLOCK_NETHERRACK: BlockType = BlockType(87);
pub const BLOCK_SOULSAND: BlockType = BlockType(88);
pub const BLOCK_GLOWSTONE: BlockType = BlockType(89);
pub const BLOCK_NETHER_PORTAL: BlockType = BlockType(90);
pub const BLOCK_PUMPKIN_LIT: BlockType = BlockType(91);
pub const BLOCK_CAKE: BlockType = BlockType(92);
pub const BLOCK_REDSTONE_REPEATER_OFF: BlockType = BlockType(93);
pub const BLOCK_REDSTONE_REPEATER_ON: BlockType = BlockType(94);
pub const BLOCK_CHEST_LOCKED: BlockType = BlockType(95);
// 95 is stained glass, which did not exist until either
// the April Fools 2.0 update or officially Release 1.7.2
pub const BLOCK_TRAPDOOR: BlockType = BlockType(96);
// 97 - 109 were added in Beta 1.8 Prerelease
// 110 - 115 were added in Beta 1.9 Prerelease
// 116 - 122 were added in Release 1.0
// 123 - 124 Redstone Lamps wered added in Release 1.2.1
// 125 - 126 were added in Release 1.3.1
// 127 - 136 were added in Release 1.3.1
// 137 - 145 were added in Release 1.4.2
// 146 - 158 were added in Release 1.5
// etc.
pub const BLOCK_MAX: BlockType = BlockType(97);
