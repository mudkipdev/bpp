/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Mojang Studios (2011)
*/

use crate::constants::CHUNK_HEIGHT;
use crate::enums::blocks::{BLOCK_AIR, BLOCK_DIRT, BLOCK_GRASS, BLOCK_LEAVES, BLOCK_LOG, BlockType};
use crate::helpers::java::java_math::{JavaMath, MathHelper, double_to_int32};
use crate::helpers::java::java_random::Random;
use crate::numeric_structs::{INT3_ZERO, Int3};
use crate::world::generator::shared::feature_gen::{WorldWrapper, is_opaque};

/// @brief Used for generating Oak or Birch Trees
///
pub struct TreeGenerator;

impl TreeGenerator {
    pub fn new() -> Self {
        Self
    }
}

pub trait TreeGeneratorBehavior {
    fn generate(&mut self, world: &mut WorldWrapper, rand: &mut Random, pos: Int3, birch: bool) -> bool;

    fn configure(&mut self, _tree_height: f64, _branch_length: f64, _trunk_shape: f64) {}
}

impl TreeGeneratorBehavior for TreeGenerator {
    /**
     * @brief Attempts to generate an oak or birch tree.
     *
     * @param world Pointer to the world where it'll generate
     * @param rand Pointer to the Java::Random that'll be utilized
     * @param pos Position of the lowest trunk-block
     * @param birch If the tree should be birch or oak
     * @return If tree successfully generated
     */
    fn generate(&mut self, world: &mut WorldWrapper, rand: &mut Random, pos: Int3, birch: bool) -> bool {
        // Decide on the tree height (birches are one block taller)
        let mut tree_height = rand.next_int_bound(3) + 4;
        if birch {
            tree_height += 1;
        }

        // Check if there's space to generate the tree
        // If any block in the desired area is neither air or
        // leaves, we fail the placement check
        if pos.y >= 1 && pos.y + tree_height + 1 <= CHUNK_HEIGHT {
            for y in pos.y..=(pos.y + 1 + tree_height) {
                let mut width = 1;
                if y == pos.y {
                    width = 0;
                }

                if y >= pos.y + 1 + tree_height - 2 {
                    width = 2;
                }

                for x in (pos.x - width)..=(pos.x + width) {
                    for z in (pos.z - width)..=(pos.z + width) {
                        // Only test blocks that're within chunk boundaries
                        if y >= 0 && y < CHUNK_HEIGHT {
                            let block_test = world.get_block_id(Int3::new(x, y, z));
                            if block_test != BLOCK_AIR && block_test != BLOCK_LEAVES {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                }
            }

            // Check if the bock under the source block is grass or dirt
            let soil_type = world.get_block_id(Int3::new(pos.x, pos.y - 1, pos.z));
            if (soil_type == BLOCK_GRASS || soil_type == BLOCK_DIRT) && pos.y < CHUNK_HEIGHT - tree_height - 1 {
                // Replace the underlying block with dir
                world.set_block(Int3::new(pos.x, pos.y - 1, pos.z), BLOCK_DIRT, 0);

                // Place leaves
                for y in (pos.y - 3 + tree_height)..=(pos.y + tree_height) {
                    let width_base = y - (pos.y + tree_height);
                    let tree_width = 1 - width_base / 2;

                    for x in (pos.x - tree_width)..=(pos.x + tree_width) {
                        let x_leaf = x - pos.x;

                        for z in (pos.z - tree_width)..=(pos.z + tree_width) {
                            let z_leaf = z - pos.z;
                            // Leaves are placed within the tree width
                            // and replace any non-opaque block
                            if (JavaMath::abs(x_leaf) != tree_width
                                || JavaMath::abs(z_leaf) != tree_width
                                || (rand.next_int_bound(2) != 0 && width_base != 0))
                                && !is_opaque(world.get_block_id(Int3::new(x, y, z)))
                            {
                                world.set_block(Int3::new(x, y, z), BLOCK_LEAVES, if birch { 2 } else { 0 });
                            }
                        }
                    }
                }

                // Replace air and leaves with trunk
                for h in 0..tree_height {
                    let future_log = world.get_block_id(Int3::new(pos.x, pos.y + h, pos.z));
                    if future_log == BLOCK_AIR || future_log == BLOCK_LEAVES {
                        world.set_block(Int3::new(pos.x, pos.y + h, pos.z), BLOCK_LOG, if birch { 2 } else { 0 });
                    }
                }

                return true;
            }
            return false;
        }
        false
    }
}

#[derive(Clone, Copy)]
struct BranchPos {
    pos: Int3,
    trunk_y: i32,
}

type BranchAxis = i8;
const AXIS_X: BranchAxis = 0;
const AXIS_Y: BranchAxis = 1;
const AXIS_Z: BranchAxis = 2;
//TRUNK_Y = 3

/// @brief Used for generating Big Oak Trees
///
pub struct BigTreeGenerator {
    rand: Random,
    base_pos: Int3,
    total_height: i32,
    height: i32,
    height_factor: f64,
    field_753_h: f64,
    trunk_slope_factor: f64,
    branch_length: f64,
    trunk_shape: f64,
    branch_density: i32,
    maximum_tree_height: i32,
    trunk_thickness: i32,
    branch_start_end: Vec<BranchPos>,
}

impl BigTreeGenerator {
    const AXIS_OFFSET: BranchAxis = 3;
    const BRANCH_ORIENTATION: [BranchAxis; 6] = [
        AXIS_Z, // X to Z
        AXIS_X, // Y to X
        AXIS_X, // Z to X
        AXIS_Y, // X to Y
        AXIS_Z, // Y to Z
        AXIS_Y, // Z to Y
    ];

    pub fn new() -> Self {
        Self {
            rand: Random::new(),
            base_pos: INT3_ZERO,
            total_height: 0,
            height: 0,
            height_factor: 0.618,
            field_753_h: 1.0,
            trunk_slope_factor: 0.381,
            branch_length: 1.0,
            trunk_shape: 1.0,
            branch_density: 1,
            maximum_tree_height: 12,
            trunk_thickness: 4,
            branch_start_end: Vec::new(),
        }
    }

    /// @brief Test if the desired tree placement is valid along the vertical axis
    ///
    /// @return If the placement is valid
    fn valid_placement(&mut self, world: &mut WorldWrapper) -> bool {
        let end_pos = Int3::new(self.base_pos.x, self.base_pos.y + self.total_height - 1, self.base_pos.z);
        // Check if ground block is valid
        let soil_type = world.get_block_id(Int3::new(self.base_pos.x, self.base_pos.y - 1, self.base_pos.z));
        if soil_type != BLOCK_GRASS && soil_type != BLOCK_DIRT {
            return false;
        }
        let clear_length = self.check_if_path_clear(world, self.base_pos, end_pos);
        // Path isn't clear
        if clear_length == -1 {
            return true;
        }
        // Path is too short
        if clear_length < 6 {
            return false;
        }
        // Path is valid
        self.total_height = clear_length;
        true
    }

    /// @brief Determine where branches will go
    ///
    fn generate_branch_positions(&mut self, world: &mut WorldWrapper) {
        self.height = double_to_int32(f64::from(self.total_height) * self.height_factor);
        if self.height >= self.total_height {
            self.height = self.total_height - 1;
        }

        let mut branches_per_layer =
            double_to_int32(1.382 + (self.trunk_shape * f64::from(self.total_height) / 13.0).powf(2.0));
        if branches_per_layer < 1 {
            branches_per_layer = 1;
        }

        let mut candidate_branches =
            vec![BranchPos { pos: INT3_ZERO, trunk_y: 0 }; (branches_per_layer * self.total_height) as usize];
        let mut current_y = self.base_pos.y + self.total_height - self.trunk_thickness;
        let mut branch_count: usize = 1;
        let target_y = self.base_pos.y + self.height;
        let mut canopy_layer = current_y - self.base_pos.y;
        candidate_branches[0].pos.x = self.base_pos.x;
        candidate_branches[0].pos.y = current_y;
        candidate_branches[0].pos.z = self.base_pos.z;
        candidate_branches[0].trunk_y = target_y;
        current_y -= 1;

        #[allow(clippy::never_loop)]
        loop {
            while canopy_layer >= 0 {
                let canopy_radius = self.get_canopy_radius(canopy_layer);
                if canopy_radius >= 0.0 {
                    for _attempts in 0..branches_per_layer {
                        let radial_distance =
                            self.branch_length * f64::from(canopy_radius) * (f64::from(self.rand.next_float()) + 0.328);
                        // Oh hey, look! An approximation of pi!
                        #[allow(clippy::approx_constant)]
                        let angle = f64::from(self.rand.next_float()) * 2.0 * 3.14159;
                        let branch_x =
                            MathHelper::floor_double(radial_distance * angle.sin() + f64::from(self.base_pos.x) + 0.5);
                        let branch_z =
                            MathHelper::floor_double(radial_distance * angle.cos() + f64::from(self.base_pos.z) + 0.5);
                        let branch_base = Int3::new(branch_x, current_y, branch_z);
                        let branch_top = Int3::new(branch_x, current_y + self.trunk_thickness, branch_z);
                        if self.check_if_path_clear(world, branch_base, branch_top) == -1 {
                            let mut trunk_connection = self.base_pos;
                            let horizontal_distance = (f64::from(JavaMath::abs(self.base_pos.x - branch_base.x)).powf(2.0)
                                + f64::from(JavaMath::abs(self.base_pos.z - branch_base.z)).powf(2.0))
                            .sqrt();
                            let vertical_drop = horizontal_distance * self.trunk_slope_factor;
                            if f64::from(branch_base.y) - vertical_drop > f64::from(target_y) {
                                trunk_connection.y = target_y;
                            } else {
                                trunk_connection.y = double_to_int32(f64::from(branch_base.y) - vertical_drop);
                            }

                            if self.check_if_path_clear(world, trunk_connection, branch_base) == -1 {
                                candidate_branches[branch_count].pos.x = branch_x;
                                candidate_branches[branch_count].pos.y = current_y;
                                candidate_branches[branch_count].pos.z = branch_z;
                                candidate_branches[branch_count].trunk_y = trunk_connection.y;
                                branch_count += 1;
                            }
                        }
                    }
                }
                current_y -= 1;
                canopy_layer -= 1;
            }

            self.branch_start_end = candidate_branches[..branch_count].to_vec();
            // Not present in OG Code, but probably good to do
            self.branch_start_end.shrink_to_fit();
            return;
        }
    }

    /// @brief Place a circle of blocks along the specified plane
    ///
    /// @param centerPos Center position of the circle
    /// @param radius Radius of the circle
    /// @param axis Axis along which the circle will grow
    /// @param blockType Blocktype of the circle
    fn place_circular_layer(
        &self,
        world: &mut WorldWrapper,
        center_pos: Int3,
        radius: f32,
        axis: BranchAxis,
        block_type: BlockType,
    ) {
        let int_radius = double_to_int32(f64::from(radius) + 0.618);
        let axis_u = Self::BRANCH_ORIENTATION[axis as usize];
        let axis_v = Self::BRANCH_ORIENTATION[(axis + Self::AXIS_OFFSET) as usize];
        let mut current_pos = Int3::new(0, 0, 0);
        current_pos[axis as usize] = center_pos[axis as usize];
        for du in -int_radius..=int_radius {
            current_pos[axis_u as usize] = center_pos[axis_u as usize] + du;

            for dv in -int_radius..=int_radius {
                // Pythagorean Theorem
                let distance = ((f64::from(JavaMath::abs(du)) + 0.5).powf(2.0)
                    + (f64::from(JavaMath::abs(dv)) + 0.5).powf(2.0))
                .sqrt();

                if distance > f64::from(radius) {
                    continue;
                }

                current_pos[axis_v as usize] = center_pos[axis_v as usize] + dv;
                let other_type = world.get_block_id(current_pos);

                // Only place if block can be overwritten
                if other_type == BLOCK_AIR || other_type == BLOCK_LEAVES {
                    world.set_block(current_pos, block_type, 0);
                }
            }
        }
    }

    /// @brief Get the radius of the leaf canopy at the desired height
    ///
    /// @param y Height to check
    /// @return Radius in blocks
    fn get_canopy_radius(&self, y: i32) -> f32 {
        if f64::from(y) < f64::from(self.total_height as f32) * 0.3 {
            -1.618
        } else {
            let half_height = self.total_height as f32 / 2.0;
            let distance_from_center = self.total_height as f32 / 2.0 - y as f32;
            let mut radius;
            if distance_from_center == 0.0 {
                radius = half_height;
            } else if MathHelper::abs(distance_from_center) >= half_height {
                radius = 0.0;
            } else {
                radius = (f64::from(MathHelper::abs(half_height)).powf(2.0)
                    - f64::from(MathHelper::abs(distance_from_center)).powf(2.0))
                .sqrt() as f32;
            }

            radius *= 0.5;
            radius
        }
    }

    /// @brief Get the radius of the current trunk layer
    ///
    /// @param layerIndex The index of the layer
    /// @return float Trunk layer radius
    fn get_trunk_layer_radius(&self, layer_index: i32) -> f32 {
        if layer_index < 0 || layer_index >= self.trunk_thickness {
            return -1.0;
        }
        // Top and bottom of trunk are thinner
        if layer_index == 0 || layer_index == self.trunk_thickness - 1 {
            return 2.0;
        }
        // Middle of trunk is thicker
        3.0
    }

    /// @brief Iterate over the height
    ///
    /// @param base
    fn place_leaves_around_point(&self, world: &mut WorldWrapper, base: Int3) {
        let base_height = base.y + self.trunk_thickness;

        for y in base.y..base_height {
            let trunk_radius = self.get_trunk_layer_radius(y - base.y);
            self.place_circular_layer(world, Int3::new(base.x, y, base.z), trunk_radius, AXIS_Y, BLOCK_LEAVES);
        }
    }

    /// @brief Draws a line of blockType between two coordinates
    ///
    /// @param startPos The start position
    /// @param endPos The end position
    /// @param blockType The block that should be drawn along this line
    fn draw_block_line(&self, world: &mut WorldWrapper, start_pos: Int3, end_pos: Int3, block_type: BlockType) {
        let delta = end_pos - start_pos;
        // Determine which axis was the largest magnitude
        let mut dominant_axis: BranchAxis = AXIS_X;
        for axis in 0..3i8 {
            if JavaMath::abs(delta[axis as usize]) > JavaMath::abs(delta[dominant_axis as usize]) {
                dominant_axis = axis;
            }
        }
        // If an axis was chosen, we can continue
        if delta[dominant_axis as usize] == 0 {
            return;
        }
        // Determine secondary axes
        // X -> Y/Z
        // Y -> X/Z
        // Z -> X/Y
        let secondary_a = Self::BRANCH_ORIENTATION[dominant_axis as usize];
        let secondary_b = Self::BRANCH_ORIENTATION[(dominant_axis + Self::AXIS_OFFSET) as usize];
        let mut step: i8 = -1;
        if delta[dominant_axis as usize] > 0 {
            step = 1;
        }

        let secondary_ratio_a = f64::from(delta[secondary_a as usize]) / f64::from(delta[dominant_axis as usize]);
        let secondary_ratio_b = f64::from(delta[secondary_b as usize]) / f64::from(delta[dominant_axis as usize]);
        let mut block_pos = INT3_ZERO;
        let mut distance_along_axis: i32 = 0;

        let total_steps = delta[dominant_axis as usize] + i32::from(step);
        while distance_along_axis != total_steps {
            block_pos[dominant_axis as usize] =
                MathHelper::floor_double(f64::from(start_pos[dominant_axis as usize] + distance_along_axis) + 0.5);
            block_pos[secondary_a as usize] = MathHelper::floor_double(
                f64::from(start_pos[secondary_a as usize]) + f64::from(distance_along_axis) * secondary_ratio_a + 0.5,
            );
            block_pos[secondary_b as usize] = MathHelper::floor_double(
                f64::from(start_pos[secondary_b as usize]) + f64::from(distance_along_axis) * secondary_ratio_b + 0.5,
            );
            world.set_block(block_pos, block_type, 0);
            distance_along_axis += i32::from(step);
        }
    }

    /// @brief Iterate through the branches and generate the leaves
    ///
    fn generate_leaf_clusters(&self, world: &mut WorldWrapper) {
        let max_branch_nodes = self.branch_start_end.len();
        for i in 0..max_branch_nodes {
            let pos = self.branch_start_end[i].pos;
            self.place_leaves_around_point(world, pos);
        }
    }

    fn can_generate_branch_at_height(&self, y: i32) -> bool {
        f64::from(y) >= (f64::from(self.total_height) * 0.2)
    }

    fn generate_trunk(&self, world: &mut WorldWrapper) {
        let mut start_pos = self.base_pos;
        let mut end_pos = self.base_pos + Int3::new(0, self.height, 0);
        self.draw_block_line(world, start_pos, end_pos, BLOCK_LOG);
        if self.branch_density == 2 {
            start_pos.x += 1;
            end_pos.x += 1;
            self.draw_block_line(world, start_pos, end_pos, BLOCK_LOG);
            start_pos.z += 1;
            end_pos.z += 1;
            self.draw_block_line(world, start_pos, end_pos, BLOCK_LOG);
            start_pos.x -= 1;
            end_pos.x -= 1;
            self.draw_block_line(world, start_pos, end_pos, BLOCK_LOG);
        }
    }

    fn generate_branches(&self, world: &mut WorldWrapper) {
        let mut base = self.base_pos;
        for branch_index in 0..self.branch_start_end.len() {
            let branch_pos = self.branch_start_end[branch_index].pos;
            let trunk_y = self.branch_start_end[branch_index].trunk_y;
            base.y = trunk_y;
            let y_height = base.y - self.base_pos.y;
            if self.can_generate_branch_at_height(y_height) {
                self.draw_block_line(world, base, branch_pos, BLOCK_LOG);
            }
        }
    }

    /// @brief Check if the path is unobstructed between start and end in a straight line
    ///
    /// @param startPos The start position
    /// @param endPos The end position
    /// @return int32_t
    fn check_if_path_clear(&self, world: &WorldWrapper, start_pos: Int3, end_pos: Int3) -> i32 {
        let delta = end_pos - start_pos;

        let mut dominant_axis: BranchAxis = AXIS_X;
        for axis in 0..3i8 {
            if JavaMath::abs(delta[axis as usize]) > JavaMath::abs(delta[dominant_axis as usize]) {
                dominant_axis = axis;
            }
        }

        if delta[dominant_axis as usize] == 0 {
            return -1;
        }
        // Determine secondary axes
        let secondary_a = Self::BRANCH_ORIENTATION[dominant_axis as usize];
        let secondary_b = Self::BRANCH_ORIENTATION[(dominant_axis + Self::AXIS_OFFSET) as usize];
        let mut step: i8 = -1;
        if delta[dominant_axis as usize] > 0 {
            step = 1;
        }

        let secondary_ratio_a = f64::from(delta[secondary_a as usize]) / f64::from(delta[dominant_axis as usize]);
        let secondary_ratio_b = f64::from(delta[secondary_b as usize]) / f64::from(delta[dominant_axis as usize]);
        let mut current_pos = INT3_ZERO;
        let mut distance_along_axis: i32 = 0;

        let total_steps = delta[dominant_axis as usize] + i32::from(step);
        while distance_along_axis != total_steps {
            current_pos[dominant_axis as usize] = start_pos[dominant_axis as usize] + distance_along_axis;
            current_pos[secondary_a as usize] = MathHelper::floor_double(
                f64::from(start_pos[secondary_a as usize]) + f64::from(distance_along_axis) * secondary_ratio_a,
            );
            current_pos[secondary_b as usize] = MathHelper::floor_double(
                f64::from(start_pos[secondary_b as usize]) + f64::from(distance_along_axis) * secondary_ratio_b,
            );
            let block_type = world.get_block_id(current_pos);
            if block_type != BLOCK_AIR && block_type != BLOCK_LEAVES {
                break;
            }
            distance_along_axis += i32::from(step);
        }

        if distance_along_axis == total_steps {
            return -1;
        }
        JavaMath::abs(distance_along_axis)
    }
}

impl TreeGeneratorBehavior for BigTreeGenerator {
    /**
     * @brief Attempts to generate a big oak tree
     *
     * @param pWorld Pointer to the world where it'll generate
     * @param pRand Pointer to the Java::Random that'll be utilized
     * @param pPos Position of the lowest trunk-block
     * @param pBirch If the tree should be birch or oak (not used for big trees)
     * @return If tree successfully generated
     */
    fn generate(&mut self, world: &mut WorldWrapper, rand: &mut Random, pos: Int3, _birch: bool) -> bool {
        let seed = rand.next_long();
        self.rand.set_seed(seed);
        self.base_pos = pos;
        // If the height wasn't set, generate a new random height
        if self.total_height == 0 {
            self.total_height = 5 + self.rand.next_int_bound(self.maximum_tree_height);
        }

        // Check if tree can be placed
        if !self.valid_placement(world) {
            return false;
        }
        self.generate_branch_positions(world);
        self.generate_leaf_clusters(world);
        self.generate_trunk(world);
        self.generate_branches(world);
        true
    }

    /**
     * @brief Configures the settings of a BigTree
     *
     * @param pTreeHeight Sets the maximum tree height (this is internally multiplied by 12)
     * @param pBranchLength Sets the maximum branch length
     * @param pTrunkShape Determines the trunk shape
     */
    fn configure(&mut self, tree_height: f64, branch_length: f64, trunk_shape: f64) {
        self.maximum_tree_height = double_to_int32(tree_height * 12.0);
        if tree_height > 0.5 {
            self.trunk_thickness = 5;
        }

        self.branch_length = branch_length;
        self.trunk_shape = trunk_shape;
    }
}

/// @brief Used for generating Taiga Trees
///
pub struct TaigaTreeGenerator;

impl TaigaTreeGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl TreeGeneratorBehavior for TaigaTreeGenerator {
    /**
     * @brief Attempts to generate a taiga tree
     *
     * @param pWorld Pointer to the world where it'll generate
     * @param pRand Pointer to the Java::Random that'll be utilized
     * @param pPos Position of the lowest trunk-block
     * @param pBirch If the tree should be birch or oak (not used for taiga trees)
     * @return If tree successfully generated
     */
    fn generate(&mut self, world: &mut WorldWrapper, rand: &mut Random, pos: Int3, _birch: bool) -> bool {
        let height = rand.next_int_bound(5) + 7;
        let trunk_height = height - rand.next_int_bound(2) - 3;
        let leaves_height = height - trunk_height;
        let max_leaf_radius = 1 + rand.next_int_bound(leaves_height + 1);
        // Check if tree can be placed
        if pos.y >= 1 && pos.y + height + 1 <= CHUNK_HEIGHT {
            let mut current_leaf_radius;
            for y in pos.y..=(pos.y + 1 + height) {
                if y - pos.y < trunk_height {
                    current_leaf_radius = 0;
                } else {
                    current_leaf_radius = max_leaf_radius;
                }

                for x in (pos.x - current_leaf_radius)..=(pos.x + current_leaf_radius) {
                    for z in (pos.z - current_leaf_radius)..=(pos.z + current_leaf_radius) {
                        if y >= 0 && y < CHUNK_HEIGHT {
                            let block_type = world.get_block_id(Int3::new(x, y, z));
                            if block_type != BLOCK_AIR && block_type != BLOCK_LEAVES {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                }
            }

            // Check if we're attempting to place it on a valid soil block
            let soil_type = world.get_block_id(Int3::new(pos.x, pos.y - 1, pos.z));
            if (soil_type == BLOCK_GRASS || soil_type == BLOCK_DIRT) && pos.y < CHUNK_HEIGHT - height - 1 {
                world.set_block(Int3::new(pos.x, pos.y - 1, pos.z), BLOCK_DIRT, 0);
                current_leaf_radius = 0;
                // Generate the leaves
                for y in ((pos.y + trunk_height)..=(pos.y + height)).rev() {
                    for x in (pos.x - current_leaf_radius)..=(pos.x + current_leaf_radius) {
                        let x_offset = x - pos.x;

                        for z in (pos.z - current_leaf_radius)..=(pos.z + current_leaf_radius) {
                            let z_offset = z - pos.z;
                            if (JavaMath::abs(x_offset) != current_leaf_radius
                                || JavaMath::abs(z_offset) != current_leaf_radius
                                || current_leaf_radius <= 0)
                                && !is_opaque(world.get_block_id(Int3::new(x, y, z)))
                            {
                                // Spruce leaves
                                world.set_block(Int3::new(x, y, z), BLOCK_LEAVES, 1);
                            }
                        }
                    }

                    if current_leaf_radius >= 1 && y == pos.y + trunk_height + 1 {
                        current_leaf_radius -= 1;
                    } else if current_leaf_radius < max_leaf_radius {
                        current_leaf_radius += 1;
                    }
                }
                // Place the trunk
                for log_y in 0..(height - 1) {
                    let r#type = world.get_block_id(Int3::new(pos.x, pos.y + log_y, pos.z));
                    if r#type == BLOCK_AIR || r#type == BLOCK_LEAVES {
                        world.set_block(Int3::new(pos.x, pos.y + log_y, pos.z), BLOCK_LOG, 1);
                    }
                }

                return true;
            }
            return false;
        }
        false
    }
}

/// @brief Used for generating Alternative Taiga Trees
///
pub struct AltTaigaTreeGenerator;

impl AltTaigaTreeGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl TreeGeneratorBehavior for AltTaigaTreeGenerator {
    /**
     * @brief Attempts to generate an alt taiga tree
     *
     * @param pWorld Pointer to the world where it'll generate
     * @param pRand Pointer to the Java::Random that'll be utilized
     * @param pPos Position of the lowest trunk-block
     * @param pBirch If the tree should be birch or oak (not used for alt taiga trees)
     * @return If tree successfully generated
     */
    fn generate(&mut self, world: &mut WorldWrapper, rand: &mut Random, pos: Int3, _birch: bool) -> bool {
        let height = rand.next_int_bound(4) + 6;
        let trunk_height = 1 + rand.next_int_bound(2);
        let leaves_height = height - trunk_height;
        let max_leaf_radius = 2 + rand.next_int_bound(2);
        // Check if tree can be placed
        if pos.y >= 1 && pos.y + height + 1 <= CHUNK_HEIGHT {
            for y in pos.y..=(pos.y + 1 + height) {
                let mut current_leaf_radius = max_leaf_radius;
                if y - pos.y < trunk_height {
                    current_leaf_radius = 0;
                }

                for x_offset in (pos.x - current_leaf_radius)..=(pos.x + current_leaf_radius) {
                    for z_offset in (pos.z - current_leaf_radius)..=(pos.z + current_leaf_radius) {
                        if y >= 0 && y < CHUNK_HEIGHT {
                            let r#type = world.get_block_id(Int3::new(x_offset, y, z_offset));
                            if r#type != BLOCK_AIR && r#type != BLOCK_LEAVES {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                }
            }

            // Check if we're attempting to place it on a valid soil block
            let soil_type = world.get_block_id(Int3::new(pos.x, pos.y - 1, pos.z));
            if (soil_type == BLOCK_GRASS || soil_type == BLOCK_DIRT) && pos.y < CHUNK_HEIGHT - height - 1 {
                world.set_block(Int3::new(pos.x, pos.y - 1, pos.z), BLOCK_DIRT, 0);
                let mut current_leaf_radius = rand.next_int_bound(2);
                let mut leaf_radius_increment_threshold = 1;
                let mut leaf_radius_switch = 0;

                for leaf_layer in 0..=leaves_height {
                    let y_level = pos.y + height - leaf_layer;

                    for x in (pos.x - current_leaf_radius)..=(pos.x + current_leaf_radius) {
                        let x_offset = x - pos.x;

                        for z in (pos.z - current_leaf_radius)..=(pos.z + current_leaf_radius) {
                            let z_offset = z - pos.z;
                            if (JavaMath::abs(x_offset) != current_leaf_radius
                                || JavaMath::abs(z_offset) != current_leaf_radius
                                || current_leaf_radius <= 0)
                                && !is_opaque(world.get_block_id(Int3::new(x, y_level, z)))
                            {
                                world.set_block(Int3::new(x, y_level, z), BLOCK_LEAVES, 1);
                            }
                        }
                    }

                    if current_leaf_radius >= leaf_radius_increment_threshold {
                        current_leaf_radius = leaf_radius_switch;
                        leaf_radius_switch = 1;
                        leaf_radius_increment_threshold += 1;
                        if leaf_radius_increment_threshold > max_leaf_radius {
                            leaf_radius_increment_threshold = max_leaf_radius;
                        }
                    } else {
                        current_leaf_radius += 1;
                    }
                }

                let log_offset = rand.next_int_bound(3);
                for log_y in 0..(height - log_offset) {
                    let r#type = world.get_block_id(Int3::new(pos.x, pos.y + log_y, pos.z));
                    if r#type == BLOCK_AIR || r#type == BLOCK_LEAVES {
                        world.set_block(Int3::new(pos.x, pos.y + log_y, pos.z), BLOCK_LOG, 1);
                    }
                }

                return true;
            }
            return false;
        }
        false
    }
}
