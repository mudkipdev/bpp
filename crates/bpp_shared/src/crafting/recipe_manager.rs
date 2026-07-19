/*
 * Copyright (c) 2026, jwaxy <jwaxy.is-a.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::base_types::{ItemDamage, ItemId};
use crate::enums::items;
use crate::helpers::hash::hash_combine;
use crate::inventory::item_stack::ItemStack;
use crate::items::item_properties;
use crate::logger::logger::global_logger;
use crate::numeric_structs::UInt8_2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ItemKey {
    pub id: ItemId,
    pub data: ItemDamage,
}

impl ItemKey {
    pub fn new(id: ItemId, data: ItemDamage) -> Self {
        ItemKey { id, data }
    }
}

impl Default for ItemKey {
    fn default() -> Self {
        ItemKey {
            id: items::INVALID,
            data: 0,
        }
    }
}

impl Hash for ItemKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut h: u64 = 0;
        hash_combine(&mut h, self.id);
        hash_combine(&mut h, self.data);
        h.hash(state);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShapedRecipeKey {
    pub width: u8,
    pub height: u8,
    pub cells: [ItemKey; 9],
}

impl Hash for ShapedRecipeKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut h: u64 = 0;

        hash_combine(&mut h, self.width);
        hash_combine(&mut h, self.height);

        for cell in self.cells {
            hash_combine(&mut h, cell);
        }

        h.hash(state);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShapelessRecipeKey {
    pub count: u8,
    pub items: [ItemKey; 9],
}

impl Hash for ShapelessRecipeKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut h: u64 = 0;

        hash_combine(&mut h, self.count);

        for item in self.items {
            hash_combine(&mut h, item);
        }

        h.hash(state);
    }
}

#[derive(Default)]
pub struct RecipeManager {
    shaped_recipes: HashMap<ShapedRecipeKey, ItemStack>,
    shapeless_recipes: HashMap<ShapelessRecipeKey, ItemStack>,
}

impl RecipeManager {
    pub fn add_shapeless_recipe(&mut self, items: &[ItemKey], output: ItemStack) {
        let key = Self::make_shapeless_key(items);
        let inserted = !self.shapeless_recipes.contains_key(&key);

        if !inserted {
            global_logger().warn(format!(
                "Overwriting existing shapeless recipe for output item {output}"
            ));
        }

        self.shapeless_recipes.insert(key, output);
    }

    // The space character ' ' is reserved for empty slots
    pub fn add_shaped_recipe(&mut self, rows: &[&str], mapping: &[(char, ItemKey)], output: ItemStack) {
        let mut grid: [ItemKey; 9] = [ItemKey::default(); 9];

        let mut table: HashMap<char, ItemKey> = HashMap::new();
        for &(symbol, item) in mapping {
            table.entry(symbol).or_insert(item);
        }

        let mut y: usize = 0;
        for &row in rows {
            if y >= 3 {
                global_logger().warn("Recipe has more than 3 rows. Skipping extra rows.");
                break;
            }

            let mut row = row;
            if row.len() > 3 {
                global_logger().warn("Recipe row has more than 3 columns. Skipping extra columns.");
                row = &row[..3];
            }

            for (x, c) in row.chars().enumerate() {
                if c == ' ' {
                    continue;
                }

                let mut mapped_item = ItemKey::default();

                for &(symbol, item) in mapping {
                    if symbol == c {
                        mapped_item = item;
                    }
                }

                if mapped_item.id == items::INVALID {
                    global_logger().warn(format!("Unknown recipe symbol '{c}'. Skipping recipe."));
                    return;
                }

                grid[y * 3 + x] = mapped_item;
            }

            y += 1;
        }

        let _ = table;

        let key = Self::make_shaped_key(&grid, UInt8_2::new(3, 3));
        let inserted = !self.shaped_recipes.contains_key(&key);

        if !inserted {
            global_logger().warn(format!(
                "Overwriting existing shaped recipe for output item {output}"
            ));
        }

        self.shaped_recipes.insert(key, output);
    }

    pub fn match_grid(&self, grid: &[ItemStack], size: UInt8_2) -> ItemStack {
        let mut key_grid: [ItemKey; 9] = [ItemKey::default(); 9];

        let total_size = size.total() as usize;

        for i in 0..total_size {
            key_grid[i] = ItemKey::new(grid[i].id, grid[i].data);
        }

        if let Some(shaped) = self
            .shaped_recipes
            .get(&Self::make_shaped_key(&key_grid[..total_size], size))
        {
            return *shaped;
        }

        if let Some(shapeless) = self
            .shapeless_recipes
            .get(&Self::make_shapeless_key(&key_grid[..total_size]))
        {
            return *shapeless;
        }

        ItemStack::default()
    }

    fn make_shaped_key(grid: &[ItemKey], size: UInt8_2) -> ShapedRecipeKey {
        let mut min_x: i32 = 3;
        let mut min_y: i32 = 3;
        let mut max_x: i32 = -1;
        let mut max_y: i32 = -1;

        // Find the bounding box of the recipe
        for y in 0..size.y as i32 {
            for x in 0..size.x as i32 {
                let item = grid[(y * size.x as i32 + x) as usize];

                if item.id == items::INVALID {
                    continue;
                }

                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }

        let mut key = ShapedRecipeKey::default();

        // Recipe is empty
        if max_x == -1 {
            return key;
        }

        key.width = (max_x - min_x + 1) as u8;
        key.height = (max_y - min_y + 1) as u8;

        // Copy the actual recipe to top left
        for y in 0..key.height as i32 {
            for x in 0..key.width as i32 {
                key.cells[(y * 3 + x) as usize] = grid[((min_y + y) * size.x as i32 + (min_x + x)) as usize];
            }
        }

        key
    }

    fn make_shapeless_key(items: &[ItemKey]) -> ShapelessRecipeKey {
        let mut key = ShapelessRecipeKey::default();

        for &item in items {
            if !item_properties::is_valid(item.id) {
                continue;
            }
            if key.count as usize >= key.items.len() {
                global_logger().error("Shapeless recipe has more than 9 valid items!\n");
                break;
            }
            key.items[key.count as usize] = item;
            key.count += 1;
        }

        key.items[..key.count as usize].sort();
        key
    }
}
