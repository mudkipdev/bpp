/*
 * Copyright (c) 2026, jwaxy <jwaxy.is-a.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use crate::base_types::{ItemDamage, ItemId};
use crate::crafting::recipe_manager::{ItemKey, RecipeManager};
use crate::enums::blocks::{
    BLOCK_BOOKSHELF, BLOCK_BRICKS, BLOCK_BUTTON_STONE, BLOCK_CHEST, BLOCK_CLAY, BLOCK_COBBLESTONE,
    BLOCK_CRAFTING_TABLE, BLOCK_DANDELION, BLOCK_DIAMOND, BLOCK_DISPENSER, BLOCK_FIRE, BLOCK_FURNACE,
    BLOCK_GLOWSTONE, BLOCK_GOLD, BLOCK_IRON, BLOCK_JUKEBOX, BLOCK_LADDER, BLOCK_LAPIS_LAZULI, BLOCK_LEVER,
    BLOCK_LOG, BLOCK_MUSHROOM_BROWN, BLOCK_MUSHROOM_RED, BLOCK_NOTEBLOCK, BLOCK_PISTON, BLOCK_PISTON_STICKY,
    BLOCK_PLANKS, BLOCK_PRESSURE_PLATE_STONE, BLOCK_PRESSURE_PLATE_WOOD, BLOCK_PUMPKIN, BLOCK_PUMPKIN_LIT,
    BLOCK_RAIL, BLOCK_RAIL_DETECTOR, BLOCK_RAIL_POWERED, BLOCK_REDSTONE_TORCH_ON, BLOCK_ROSE, BLOCK_SAND,
    BLOCK_SANDSTONE, BLOCK_SLAB, BLOCK_SNOW, BLOCK_STAIRS_COBBLESTONE, BLOCK_STAIRS_WOOD, BLOCK_STONE,
    BLOCK_TNT, BLOCK_TORCH, BLOCK_TRAPDOOR, BLOCK_WOOL, BlockType,
};
use crate::enums::items;
use crate::inventory::item_stack::ItemStack;

fn block_id(block: BlockType) -> ItemId {
    ItemId(block.0 as i16)
}

impl RecipeManager {
    pub fn add_vanilla_recipes(&mut self) {
        // Msc
        self.add_shaped_recipe(
            &["###", "# #", "###"],
            &[('#', ItemKey::new(block_id(BLOCK_PLANKS), 0))],
            ItemStack {
                id: block_id(BLOCK_CHEST),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###", "# #", "###"],
            &[('#', ItemKey::new(block_id(BLOCK_COBBLESTONE), 0))],
            ItemStack {
                id: block_id(BLOCK_FURNACE),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["##", "##"],
            &[('#', ItemKey::new(block_id(BLOCK_PLANKS), 0))],
            ItemStack {
                id: block_id(BLOCK_CRAFTING_TABLE),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#", "#"],
            &[('#', ItemKey::new(block_id(BLOCK_PLANKS), 0))],
            ItemStack {
                id: items::STICK,
                count: 4,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#"],
            &[('#', ItemKey::new(block_id(BLOCK_LOG), 0))],
            ItemStack {
                id: block_id(BLOCK_PLANKS),
                count: 4,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#"],
            &[('#', ItemKey::new(block_id(BLOCK_LOG), 1))],
            ItemStack {
                id: block_id(BLOCK_PLANKS),
                count: 4,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#"],
            &[('#', ItemKey::new(block_id(BLOCK_LOG), 2))],
            ItemStack {
                id: block_id(BLOCK_PLANKS),
                count: 4,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###"],
            &[('#', ItemKey::new(items::SUGARCANE, 0))],
            ItemStack {
                id: items::PAPER,
                count: 3,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#", "#", "#"],
            &[('#', ItemKey::new(items::PAPER, 0))],
            ItemStack {
                id: items::BOOK,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###", "#X#", "###"],
            &[
                ('#', ItemKey::new(block_id(BLOCK_PLANKS), 0)),
                ('X', ItemKey::new(items::DIAMOND, 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_JUKEBOX),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###", "#X#", "###"],
            &[
                ('#', ItemKey::new(block_id(BLOCK_PLANKS), 0)),
                ('X', ItemKey::new(items::REDSTONE, 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_NOTEBLOCK),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###", "XXX", "###"],
            &[
                ('#', ItemKey::new(block_id(BLOCK_PLANKS), 0)),
                ('X', ItemKey::new(items::BOOK, 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_BOOKSHELF),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["##", "##"],
            &[('#', ItemKey::new(items::SNOWBALL, 0))],
            ItemStack {
                id: block_id(BLOCK_SNOW),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["##", "##"],
            &[('#', ItemKey::new(items::CLAY, 0))],
            ItemStack {
                id: block_id(BLOCK_CLAY),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["##", "##"],
            &[('#', ItemKey::new(items::BRICK, 0))],
            ItemStack {
                id: block_id(BLOCK_BRICKS),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["##", "##"],
            &[('#', ItemKey::new(items::GLOWSTONE_DUST, 0))],
            ItemStack {
                id: block_id(BLOCK_GLOWSTONE),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["##", "##"],
            &[('#', ItemKey::new(items::STRING, 0))],
            ItemStack {
                id: block_id(BLOCK_WOOL),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["##", "##"],
            &[('#', ItemKey::new(block_id(BLOCK_SAND), 0))],
            ItemStack {
                id: block_id(BLOCK_SANDSTONE),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["X#X", "#X#", "X#X"],
            &[
                ('#', ItemKey::new(block_id(BLOCK_SAND), 0)),
                ('X', ItemKey::new(items::GUNPOWDER, 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_TNT),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#  ", "## ", "###"],
            &[('#', ItemKey::new(block_id(BLOCK_PLANKS), 0))],
            ItemStack {
                id: block_id(BLOCK_STAIRS_WOOD),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#  ", "## ", "###"],
            &[('#', ItemKey::new(block_id(BLOCK_COBBLESTONE), 0))],
            ItemStack {
                id: block_id(BLOCK_STAIRS_COBBLESTONE),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###"],
            &[('#', ItemKey::new(block_id(BLOCK_PLANKS), 0))],
            ItemStack {
                id: block_id(BLOCK_SLAB),
                count: 3,
                data: 2,
            },
        );
        self.add_shaped_recipe(
            &["###"],
            &[('#', ItemKey::new(block_id(BLOCK_COBBLESTONE), 0))],
            ItemStack {
                id: block_id(BLOCK_SLAB),
                count: 3,
                data: 3,
            },
        );
        self.add_shaped_recipe(
            &["###"],
            &[('#', ItemKey::new(block_id(BLOCK_STONE), 0))],
            ItemStack {
                id: block_id(BLOCK_SLAB),
                count: 3,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###"],
            &[('#', ItemKey::new(block_id(BLOCK_SANDSTONE), 0))],
            ItemStack {
                id: block_id(BLOCK_SLAB),
                count: 3,
                data: 1,
            },
        );
        self.add_shaped_recipe(
            &["# #", "###", "# #"],
            &[('#', ItemKey::new(items::STICK, 0))],
            ItemStack {
                id: block_id(BLOCK_LADDER),
                count: 2,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["##", "##", "##"],
            &[('#', ItemKey::new(block_id(BLOCK_PLANKS), 0))],
            ItemStack {
                id: items::DOOR_WOOD,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["##", "##", "##"],
            &[('#', ItemKey::new(items::IRON, 0))],
            ItemStack {
                id: items::DOOR_IRON,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###", "###"],
            &[('#', ItemKey::new(items::IRON, 0))],
            ItemStack {
                id: block_id(BLOCK_TRAPDOOR),
                count: 2,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###", "###", " X "],
            &[
                ('#', ItemKey::new(block_id(BLOCK_PLANKS), 0)),
                ('X', ItemKey::new(items::STICK, 0)),
            ],
            ItemStack {
                id: items::SIGN,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["AAA", "BEB", "CCC"],
            &[
                ('A', ItemKey::new(items::BUCKET_MILK, 0)),
                ('B', ItemKey::new(items::SUGAR, 0)),
                ('C', ItemKey::new(items::WHEAT, 0)),
                ('E', ItemKey::new(items::EGG, 0)),
            ],
            ItemStack {
                id: items::CAKE,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#"],
            &[('#', ItemKey::new(items::SUGARCANE, 0))],
            ItemStack {
                id: items::SUGAR,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#", "X"],
            &[
                ('#', ItemKey::new(items::COAL, 0)),
                ('X', ItemKey::new(items::STICK, 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_TORCH),
                count: 4,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#", "X"],
            &[
                ('#', ItemKey::new(items::COAL, 1)),
                ('X', ItemKey::new(items::STICK, 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_TORCH),
                count: 4,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["# #", " # "],
            &[('#', ItemKey::new(block_id(BLOCK_PLANKS), 0))],
            ItemStack {
                id: items::BOWL,
                count: 4,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["X X", "X#X", "X X"],
            &[
                ('X', ItemKey::new(items::IRON, 0)),
                ('#', ItemKey::new(items::STICK, 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_RAIL),
                count: 16,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["X X", "X#X", "XRX"],
            &[
                ('X', ItemKey::new(items::GOLD, 0)),
                ('#', ItemKey::new(items::STICK, 0)),
                ('R', ItemKey::new(items::REDSTONE, 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_RAIL_POWERED),
                count: 6,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["X X", "X#X", "XRX"],
            &[
                ('X', ItemKey::new(items::IRON, 0)),
                ('#', ItemKey::new(block_id(BLOCK_PRESSURE_PLATE_STONE), 0)),
                ('R', ItemKey::new(items::REDSTONE, 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_RAIL_DETECTOR),
                count: 6,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["# #", "###"],
            &[('#', ItemKey::new(items::IRON, 0))],
            ItemStack {
                id: items::MINECART,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#", "A"],
            &[
                ('#', ItemKey::new(block_id(BLOCK_PUMPKIN), 0)),
                ('A', ItemKey::new(block_id(BLOCK_TORCH), 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_PUMPKIN_LIT),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#", "A"],
            &[
                ('#', ItemKey::new(block_id(BLOCK_CHEST), 0)),
                ('A', ItemKey::new(items::MINECART, 0)),
            ],
            ItemStack {
                id: items::MINECART_CHEST,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#", "A"],
            &[
                ('#', ItemKey::new(block_id(BLOCK_FURNACE), 0)),
                ('A', ItemKey::new(items::MINECART, 0)),
            ],
            ItemStack {
                id: items::MINECART_FURNACE,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["# #", "###"],
            &[('#', ItemKey::new(block_id(BLOCK_PLANKS), 0))],
            ItemStack {
                id: items::BOAT,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["# #", " # "],
            &[('#', ItemKey::new(items::IRON, 0))],
            ItemStack {
                id: items::BUCKET,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["A ", " B"],
            &[
                ('A', ItemKey::new(items::IRON, 0)),
                ('B', ItemKey::new(items::FLINT, 0)),
            ],
            ItemStack {
                id: items::FLINT_AND_STEEL,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###"],
            &[('#', ItemKey::new(items::WHEAT, 0))],
            ItemStack {
                id: items::BREAD,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["  #", " #X", "# X"],
            &[
                ('#', ItemKey::new(items::STICK, 0)),
                ('X', ItemKey::new(items::STRING, 0)),
            ],
            ItemStack {
                id: items::FISHING_ROD,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###", "#X#", "###"],
            &[
                ('#', ItemKey::new(items::STICK, 0)),
                ('X', ItemKey::new(block_id(BLOCK_WOOL), 0)),
            ],
            ItemStack {
                id: items::PAINTING,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###", "#X#", "###"],
            &[
                ('#', ItemKey::new(block_id(BLOCK_GOLD), 0)),
                ('X', ItemKey::new(items::APPLE, 0)),
            ],
            ItemStack {
                id: items::APPLE_GOLDEN,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["X", "#"],
            &[
                ('X', ItemKey::new(items::STICK, 0)),
                ('#', ItemKey::new(block_id(BLOCK_COBBLESTONE), 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_LEVER),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["X", "#"],
            &[
                ('X', ItemKey::new(items::REDSTONE, 0)),
                ('#', ItemKey::new(items::STICK, 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_REDSTONE_TORCH_ON),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#X#", "III"],
            &[
                ('#', ItemKey::new(block_id(BLOCK_REDSTONE_TORCH_ON), 0)),
                ('X', ItemKey::new(items::REDSTONE, 0)),
                ('I', ItemKey::new(block_id(BLOCK_STONE), 0)),
            ],
            ItemStack {
                id: items::REDSTONE_REPEATER,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &[" # ", "#X#", " # "],
            &[
                ('#', ItemKey::new(items::GOLD, 0)),
                ('X', ItemKey::new(items::REDSTONE, 0)),
            ],
            ItemStack {
                id: items::CLOCK,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &[" # ", "#X#", " # "],
            &[
                ('#', ItemKey::new(items::IRON, 0)),
                ('X', ItemKey::new(items::REDSTONE, 0)),
            ],
            ItemStack {
                id: items::COMPASS,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###", "#X#", "###"],
            &[
                ('#', ItemKey::new(items::PAPER, 0)),
                ('X', ItemKey::new(items::COMPASS, 0)),
            ],
            ItemStack {
                id: items::MAP,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#", "#"],
            &[('#', ItemKey::new(block_id(BLOCK_STONE), 0))],
            ItemStack {
                id: block_id(BLOCK_BUTTON_STONE),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["##"],
            &[('#', ItemKey::new(block_id(BLOCK_STONE), 0))],
            ItemStack {
                id: block_id(BLOCK_PRESSURE_PLATE_STONE),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["##"],
            &[('#', ItemKey::new(block_id(BLOCK_PLANKS), 0))],
            ItemStack {
                id: block_id(BLOCK_PRESSURE_PLATE_WOOD),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###", "#X#", "#R#"],
            &[
                ('#', ItemKey::new(block_id(BLOCK_COBBLESTONE), 0)),
                ('X', ItemKey::new(items::BOW, 0)),
                ('R', ItemKey::new(items::REDSTONE, 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_DISPENSER),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["TTT", "#X#", "#R#"],
            &[
                ('#', ItemKey::new(block_id(BLOCK_COBBLESTONE), 0)),
                ('X', ItemKey::new(items::IRON, 0)),
                ('R', ItemKey::new(items::REDSTONE, 0)),
                ('T', ItemKey::new(block_id(BLOCK_PLANKS), 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_PISTON),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["S", "P"],
            &[
                ('S', ItemKey::new(items::SLIME, 0)),
                ('P', ItemKey::new(block_id(BLOCK_PISTON), 0)),
            ],
            ItemStack {
                id: block_id(BLOCK_PISTON_STICKY),
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["###", "XXX"],
            &[
                ('#', ItemKey::new(block_id(BLOCK_WOOL), 0)),
                ('X', ItemKey::new(block_id(BLOCK_PLANKS), 0)),
            ],
            ItemStack {
                id: items::BED,
                count: 1,
                data: 0,
            },
        );

        // Armor
        let mut add_armor = |material: ItemId, helmet_id: ItemId, chest_id: ItemId, leggings_id: ItemId,
                              boots_id: ItemId| {
            self.add_shaped_recipe(
                &["###", "# #"],
                &[('#', ItemKey::new(material, 0))],
                ItemStack {
                    id: helmet_id,
                    count: 1,
                    data: 0,
                },
            );
            self.add_shaped_recipe(
                &["# #", "###", "###"],
                &[('#', ItemKey::new(material, 0))],
                ItemStack {
                    id: chest_id,
                    count: 1,
                    data: 0,
                },
            );
            self.add_shaped_recipe(
                &["###", "# #", "# #"],
                &[('#', ItemKey::new(material, 0))],
                ItemStack {
                    id: leggings_id,
                    count: 1,
                    data: 0,
                },
            );
            self.add_shaped_recipe(
                &["# #", "# #"],
                &[('#', ItemKey::new(material, 0))],
                ItemStack {
                    id: boots_id,
                    count: 1,
                    data: 0,
                },
            );
        };

        add_armor(
            items::GOLD,
            items::HELMET_GOLD,
            items::CHESTPLATE_GOLD,
            items::LEGGINGS_GOLD,
            items::BOOTS_GOLD,
        );
        add_armor(
            items::IRON,
            items::HELMET_IRON,
            items::CHESTPLATE_IRON,
            items::LEGGINGS_IRON,
            items::BOOTS_IRON,
        );
        add_armor(
            items::DIAMOND,
            items::HELMET_DIAMOND,
            items::CHESTPLATE_DIAMOND,
            items::LEGGINGS_DIAMOND,
            items::BOOTS_DIAMOND,
        );
        add_armor(
            block_id(BLOCK_FIRE),
            items::HELMET_CHAINMAIL,
            items::CHESTPLATE_CHAINMAIL,
            items::LEGGINGS_CHAINMAIL,
            items::BOOTS_CHAINMAIL,
        );
        add_armor(
            items::LEATHER,
            items::HELMET_LEATHER,
            items::CHESTPLATE_LEATHER,
            items::LEGGINGS_LEATHER,
            items::BOOTS_LEATHER,
        );

        // Tools
        let mut add_tools = |tool_material: ItemId, sword_id: ItemId, pick_id: ItemId, shovel_id: ItemId,
                              axe_id: ItemId| {
            self.add_shaped_recipe(
                &["###", " A ", " A "],
                &[
                    ('#', ItemKey::new(tool_material, 0)),
                    ('A', ItemKey::new(items::STICK, 0)),
                ],
                ItemStack {
                    id: pick_id,
                    count: 1,
                    data: 0,
                },
            );
            self.add_shaped_recipe(
                &["#", "#", "A"],
                &[
                    ('#', ItemKey::new(tool_material, 0)),
                    ('A', ItemKey::new(items::STICK, 0)),
                ],
                ItemStack {
                    id: sword_id,
                    count: 1,
                    data: 0,
                },
            );
            self.add_shaped_recipe(
                &["## ", "#A ", " A "],
                &[
                    ('#', ItemKey::new(tool_material, 0)),
                    ('A', ItemKey::new(items::STICK, 0)),
                ],
                ItemStack {
                    id: axe_id,
                    count: 1,
                    data: 0,
                },
            );
            self.add_shaped_recipe(
                &[" ##", " A#", " A "],
                &[
                    ('#', ItemKey::new(tool_material, 0)),
                    ('A', ItemKey::new(items::STICK, 0)),
                ],
                ItemStack {
                    id: axe_id,
                    count: 1,
                    data: 0,
                },
            );
            self.add_shaped_recipe(
                &["#", "A", "A"],
                &[
                    ('#', ItemKey::new(tool_material, 0)),
                    ('A', ItemKey::new(items::STICK, 0)),
                ],
                ItemStack {
                    id: shovel_id,
                    count: 1,
                    data: 0,
                },
            );
        };

        add_tools(
            block_id(BLOCK_COBBLESTONE),
            items::SWORD_STONE,
            items::PICKAXE_STONE,
            items::SHOVEL_STONE,
            items::AXE_STONE,
        );
        add_tools(
            block_id(BLOCK_PLANKS),
            items::SWORD_WOOD,
            items::PICKAXE_WOOD,
            items::SHOVEL_WOOD,
            items::AXE_WOOD,
        );
        add_tools(
            items::IRON,
            items::SWORD_IRON,
            items::PICKAXE_IRON,
            items::SHOVEL_IRON,
            items::AXE_IRON,
        );
        add_tools(
            items::GOLD,
            items::SWORD_GOLD,
            items::PICKAXE_GOLD,
            items::SHOVEL_GOLD,
            items::AXE_GOLD,
        );
        add_tools(
            items::DIAMOND,
            items::SWORD_DIAMOND,
            items::PICKAXE_DIAMOND,
            items::SHOVEL_DIAMOND,
            items::AXE_DIAMOND,
        );

        // Blocks -> ingots, ingots -> blocks
        let mut add_material = |material: ItemId, material_meta: u8, stored_material: ItemId| {
            self.add_shaped_recipe(
                &["###", "###", "###"],
                &[('#', ItemKey::new(material, material_meta as ItemDamage))],
                ItemStack {
                    id: stored_material,
                    count: 1,
                    data: 0,
                },
            );
            self.add_shaped_recipe(
                &["#"],
                &[('#', ItemKey::new(stored_material, 0))],
                ItemStack {
                    id: material,
                    count: 9,
                    data: material_meta as ItemDamage,
                },
            );
        };

        add_material(items::IRON, 0, block_id(BLOCK_IRON));
        add_material(items::DIAMOND, 0, block_id(BLOCK_DIAMOND));
        add_material(items::GOLD, 0, block_id(BLOCK_GOLD));
        add_material(items::DYE, 4, block_id(BLOCK_LAPIS_LAZULI));

        // Food items
        self.add_shaped_recipe(
            &["Y", "X", "#"],
            &[
                ('X', ItemKey::new(block_id(BLOCK_MUSHROOM_BROWN), 0)),
                ('Y', ItemKey::new(block_id(BLOCK_MUSHROOM_RED), 0)),
                ('#', ItemKey::new(items::BOWL, 0)),
            ],
            ItemStack {
                id: items::MUSHROOM_STEW,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["Y", "X", "#"],
            &[
                ('X', ItemKey::new(block_id(BLOCK_MUSHROOM_RED), 0)),
                ('Y', ItemKey::new(block_id(BLOCK_MUSHROOM_BROWN), 0)),
                ('#', ItemKey::new(items::BOWL, 0)),
            ],
            ItemStack {
                id: items::MUSHROOM_STEW,
                count: 1,
                data: 0,
            },
        );
        self.add_shaped_recipe(
            &["#X#"],
            &[
                ('X', ItemKey::new(items::DYE, 3)),
                ('#', ItemKey::new(items::WHEAT, 0)),
            ],
            ItemStack {
                id: items::COOKIE,
                count: 8,
                data: 0,
            },
        );

        // Wool + Dye -> redyed Wool (dye meta and wool meta are inverse: 15 - dyeMeta)
        for i in 0u8..16 {
            self.add_shapeless_recipe(
                &[
                    ItemKey::new(items::DYE, i as ItemDamage),
                    ItemKey::new(block_id(BLOCK_WOOL), 0),
                ],
                ItemStack {
                    id: block_id(BLOCK_WOOL),
                    count: 1,
                    data: (15 - i) as ItemDamage,
                },
            );
        }

        // Dye meta reference: 0=Black 1=Red 2=Green 3=Brown 4=Blue 5=Purple 6=Cyan
        //                     7=LightGray 8=Gray 9=Pink 10=Lime 11=Yellow 12=LightBlue
        //                     13=Magenta 14=Orange 15=White

        self.add_shapeless_recipe(
            &[ItemKey::new(block_id(BLOCK_DANDELION), 0)],
            ItemStack {
                id: items::DYE,
                count: 2,
                data: 11,
            },
        ); // Dandelion -> Yellow
        self.add_shapeless_recipe(
            &[ItemKey::new(block_id(BLOCK_ROSE), 0)],
            ItemStack {
                id: items::DYE,
                count: 2,
                data: 1,
            },
        ); // Rose -> Red
        self.add_shapeless_recipe(
            &[ItemKey::new(items::BONE, 0)],
            ItemStack {
                id: items::DYE,
                count: 3,
                data: 15,
            },
        ); // Bone -> Bone Meal (White)

        self.add_shapeless_recipe(
            &[ItemKey::new(items::DYE, 1), ItemKey::new(items::DYE, 15)],
            ItemStack {
                id: items::DYE,
                count: 2,
                data: 9,
            },
        ); // Red + White -> Pink
        self.add_shapeless_recipe(
            &[ItemKey::new(items::DYE, 1), ItemKey::new(items::DYE, 11)],
            ItemStack {
                id: items::DYE,
                count: 2,
                data: 14,
            },
        ); // Red + Yellow -> Orange
        self.add_shapeless_recipe(
            &[ItemKey::new(items::DYE, 2), ItemKey::new(items::DYE, 15)],
            ItemStack {
                id: items::DYE,
                count: 2,
                data: 10,
            },
        ); // Green + White -> Lime
        self.add_shapeless_recipe(
            &[ItemKey::new(items::DYE, 0), ItemKey::new(items::DYE, 15)],
            ItemStack {
                id: items::DYE,
                count: 2,
                data: 8,
            },
        ); // Black + White -> Gray
        self.add_shapeless_recipe(
            &[ItemKey::new(items::DYE, 8), ItemKey::new(items::DYE, 15)],
            ItemStack {
                id: items::DYE,
                count: 2,
                data: 7,
            },
        ); // Gray + White -> Light Gray
        self.add_shapeless_recipe(
            &[
                ItemKey::new(items::DYE, 0),
                ItemKey::new(items::DYE, 15),
                ItemKey::new(items::DYE, 15),
            ],
            ItemStack {
                id: items::DYE,
                count: 3,
                data: 7,
            },
        ); // Black + White + White -> Light Gray (alt)
        self.add_shapeless_recipe(
            &[ItemKey::new(items::DYE, 4), ItemKey::new(items::DYE, 15)],
            ItemStack {
                id: items::DYE,
                count: 2,
                data: 12,
            },
        ); // Blue + White -> Light Blue
        self.add_shapeless_recipe(
            &[ItemKey::new(items::DYE, 4), ItemKey::new(items::DYE, 2)],
            ItemStack {
                id: items::DYE,
                count: 2,
                data: 6,
            },
        ); // Blue + Green -> Cyan
        self.add_shapeless_recipe(
            &[ItemKey::new(items::DYE, 4), ItemKey::new(items::DYE, 1)],
            ItemStack {
                id: items::DYE,
                count: 2,
                data: 5,
            },
        ); // Blue + Red -> Purple
        self.add_shapeless_recipe(
            &[ItemKey::new(items::DYE, 5), ItemKey::new(items::DYE, 9)],
            ItemStack {
                id: items::DYE,
                count: 2,
                data: 13,
            },
        ); // Purple + Pink -> Magenta
        self.add_shapeless_recipe(
            &[
                ItemKey::new(items::DYE, 4),
                ItemKey::new(items::DYE, 1),
                ItemKey::new(items::DYE, 9),
            ],
            ItemStack {
                id: items::DYE,
                count: 3,
                data: 13,
            },
        ); // Blue + Red + Pink -> Magenta (alt)
        self.add_shapeless_recipe(
            &[
                ItemKey::new(items::DYE, 4),
                ItemKey::new(items::DYE, 1),
                ItemKey::new(items::DYE, 1),
                ItemKey::new(items::DYE, 15),
            ],
            ItemStack {
                id: items::DYE,
                count: 4,
                data: 13,
            },
        ); // Blue + Red + Red + White -> Magenta (alt 2)
    }
}
