/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use crate::base_types::{ItemAmount, ItemDamage, ItemId};
use crate::blocks::block_properties::{self, BlockBehavior, BlockProperties, StepSound};
use crate::blocks::materials::Material;
use crate::entities::entity::Entity;
use crate::enums::blocks::*;
use crate::enums::items;
use crate::enums::network::packet_data::{self as packet_data, FaceDirection};
use crate::helpers::aabb::{AABB, CollisionShape};
use crate::helpers::java::java_random::Random;
use crate::inventory::item_stack::ItemStack;
use crate::numeric_structs::{Int3, Vec3};
use crate::world::world::WorldManager;

// Global table definitions; declared extern in the header

// Behavior helper functions

fn get_fluid_flow_vector(world: &WorldManager, pos: Int3) -> Vec3 {
    let water_material = Material::water();
    let mut flow_vector = Vec3::new(0.0, 0.0, 0.0);
    let get_effective_flow_decay = |l_world: &WorldManager, l_pos: Int3, l_material: Material| -> i32 {
        if l_world.get_material(pos) != l_material {
            return -1;
        }
        let mut meta = l_world.get_metadata(l_pos) as i32;
        if meta >= 8 {
            meta = 0;
        }

        meta
    };

    let my_flow_contribution = get_effective_flow_decay(world, pos, water_material);

    // Get the contribution of our horizontal neighbors
    let ndx: [i32; 4] = [-1, 1, 0, 0];
    let ndz: [i32; 4] = [0, 0, -1, 1];
    for i in 0..4 {
        let dx = pos.x + ndx[i];
        let dz = pos.z + ndz[i];
        let neighbor_flow_contribution = get_effective_flow_decay(world, Int3::new(dx, pos.y, dz), water_material);
        let mut flow_difference = 0;
        // Our neighbor block didn't have the same material
        if neighbor_flow_contribution < 0 {
            if !world.get_material(Int3::new(dx, pos.y, dz)).is_solid {
                // Check the block below us to see if its water, if it is, STRONGLY pull down
                let below_flow_contribution = get_effective_flow_decay(world, Int3::new(dx, pos.y - 1, dz), water_material);
                if below_flow_contribution >= 0 {
                    flow_difference = below_flow_contribution - (my_flow_contribution - 8);
                    flow_vector.x += ((dx - pos.x) * flow_difference) as f64;
                    flow_vector.z += ((dz - pos.z) * flow_difference) as f64;
                }
            }
        } else {
            flow_difference = neighbor_flow_contribution - my_flow_contribution;
            flow_vector.x += ((dx - pos.x) * flow_difference) as f64;
            flow_vector.z += ((dz - pos.z) * flow_difference) as f64;
        }
    }

    let is_fluid_wall = |check_pos: Int3| -> bool {
        let neighbor_material = world.get_material(check_pos);
        if neighbor_material == water_material {
            return false;
        }
        if neighbor_material == Material::ice() {
            return false;
        }
        neighbor_material.is_solid
    };

    // If we're a falling fluid segment, check whether we're clinging to a wall
    if world.get_metadata(pos) >= 8 {
        let mut near_wall = false;

        if !near_wall && is_fluid_wall(Int3::new(pos.x, pos.y, pos.z - 1)) {
            near_wall = true;
        }
        if !near_wall && is_fluid_wall(Int3::new(pos.x, pos.y, pos.z + 1)) {
            near_wall = true;
        }
        if !near_wall && is_fluid_wall(Int3::new(pos.x - 1, pos.y, pos.z)) {
            near_wall = true;
        }
        if !near_wall && is_fluid_wall(Int3::new(pos.x + 1, pos.y, pos.z)) {
            near_wall = true;
        }
        if !near_wall && is_fluid_wall(Int3::new(pos.x, pos.y + 1, pos.z - 1)) {
            near_wall = true;
        }
        if !near_wall && is_fluid_wall(Int3::new(pos.x, pos.y + 1, pos.z + 1)) {
            near_wall = true;
        }
        if !near_wall && is_fluid_wall(Int3::new(pos.x - 1, pos.y + 1, pos.z)) {
            near_wall = true;
        }
        if !near_wall && is_fluid_wall(Int3::new(pos.x + 1, pos.y + 1, pos.z)) {
            near_wall = true;
        }

        if near_wall {
            // Normalize what we have so far, then let the huge -6 dominate the normalization after this
            let len_sq = flow_vector.x * flow_vector.x + flow_vector.y * flow_vector.y + flow_vector.z * flow_vector.z;
            if len_sq > 0.0 {
                let inv_len = 1.0 / len_sq.sqrt();
                flow_vector.x *= inv_len;
                flow_vector.y *= inv_len;
                flow_vector.z *= inv_len;
            }
            flow_vector.y += -6.0;
        }
    }

    // Final normalize
    let len_sq = flow_vector.x * flow_vector.x + flow_vector.y * flow_vector.y + flow_vector.z * flow_vector.z;
    if len_sq > 0.0 {
        let inv_len = 1.0 / len_sq.sqrt();
        flow_vector.x *= inv_len;
        flow_vector.y *= inv_len;
        flow_vector.z *= inv_len;
    }

    flow_vector
}

// defaults
fn default_aabb(_meta: u8) -> AABB {
    AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 }
}
fn default_collider(_meta: u8) -> CollisionShape {
    let mut s = CollisionShape::default();
    s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 });
    s
}

// slab
fn slab_aabb(_meta: u8) -> AABB {
    AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 0.5, max_z: 1.0 }
}
fn slab_collider(_meta: u8) -> CollisionShape {
    let mut s = CollisionShape::default();
    s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 0.5, max_z: 1.0 });
    s
}

// stairs
fn stair_collider(meta: u8) -> CollisionShape {
    let mut s = CollisionShape::default();
    match meta & 3 {
        0 => {
            s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 0.5, max_y: 0.5, max_z: 1.0 });
            s.add(AABB { min_x: 0.5, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 });
        }
        1 => {
            s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 0.5, max_y: 1.0, max_z: 1.0 });
            s.add(AABB { min_x: 0.5, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 0.5, max_z: 1.0 });
        }
        2 => {
            s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 0.5, max_z: 0.5 });
            s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.5, max_x: 1.0, max_y: 1.0, max_z: 1.0 });
        }
        3 => {
            s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 0.5 });
            s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.5, max_x: 1.0, max_y: 0.5, max_z: 1.0 });
        }
        _ => {}
    }
    s
}

// cactus
fn cactus_aabb(_meta: u8) -> AABB {
    const I: f64 = 0.0625;
    AABB { min_x: I, min_y: 0.0, min_z: I, max_x: 1.0 - I, max_y: 1.0, max_z: 1.0 - I }
}
fn cactus_collider(_meta: u8) -> CollisionShape {
    const I: f64 = 0.0625;
    let mut s = CollisionShape::default();
    s.add(AABB { min_x: I, min_y: 0.0, min_z: I, max_x: 1.0 - I, max_y: 1.0 - I, max_z: 1.0 - I });
    s
}

// snow layer
fn snow_layer_aabb(meta: u8) -> AABB {
    let h: f32 = (2.0f32 * ((1 + (meta & 7) as i32) as f32)) / 16.0;
    AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: h as f64, max_z: 1.0 }
}
fn snow_layer_collider(meta: u8) -> CollisionShape {
    let mut s = CollisionShape::default();
    if (meta & 7) >= 3 {
        s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 0.5, max_z: 1.0 });
    }
    s
}

// ladder
fn ladder_aabb(meta: u8) -> AABB {
    const T: f64 = 0.125;
    match meta {
        2 => AABB { min_x: 0.0, min_y: 0.0, min_z: 1.0 - T, max_x: 1.0, max_y: 1.0, max_z: 1.0 },
        3 => AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: T },
        4 => AABB { min_x: 1.0 - T, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 },
        5 => AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: T, max_y: 1.0, max_z: 1.0 },
        _ => AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 },
    }
}
fn ladder_collider(meta: u8) -> CollisionShape {
    const T: f64 = 0.125;
    let mut s = CollisionShape::default();
    match meta {
        2 => s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 1.0 - T, max_x: 1.0, max_y: 1.0, max_z: 1.0 }),
        3 => s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: T }),
        4 => s.add(AABB { min_x: 1.0 - T, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 }),
        5 => s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: T, max_y: 1.0, max_z: 1.0 }),
        _ => {}
    }
    s
}

// door
// bits 0-1 = facing when closed, bit 2 = open, bit 3 = top half
fn door_state(meta: u8) -> i32 {
    if (meta & 4) == 0 { ((meta as i32) - 1) & 3 } else { (meta as i32) & 3 }
}
fn door_aabb(meta: u8) -> AABB {
    const T: f64 = 0.1875;
    match door_state(meta) {
        0 => AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: T },
        1 => AABB { min_x: 1.0 - T, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 },
        2 => AABB { min_x: 0.0, min_y: 0.0, min_z: 1.0 - T, max_x: 1.0, max_y: 1.0, max_z: 1.0 },
        3 => AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: T, max_y: 1.0, max_z: 1.0 },
        _ => AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 },
    }
}
fn door_collider(meta: u8) -> CollisionShape {
    const T: f64 = 0.1875;
    let mut s = CollisionShape::default();
    match door_state(meta) {
        0 => s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: T }),
        1 => s.add(AABB { min_x: 1.0 - T, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 }),
        2 => s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 1.0 - T, max_x: 1.0, max_y: 1.0, max_z: 1.0 }),
        3 => s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: T, max_y: 1.0, max_z: 1.0 }),
        _ => {}
    }
    s
}

// trapdoor
fn trapdoor_aabb(meta: u8) -> AABB {
    const T: f64 = 0.1875;
    if (meta & 4) == 0 {
        return AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: T, max_z: 1.0 };
    }
    match meta & 3 {
        0 => AABB { min_x: 0.0, min_y: 0.0, min_z: 1.0 - T, max_x: 1.0, max_y: 1.0, max_z: 1.0 },
        1 => AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: T },
        2 => AABB { min_x: 1.0 - T, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 },
        3 => AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: T, max_y: 1.0, max_z: 1.0 },
        _ => AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 },
    }
}

fn farmland_collider(_meta: u8) -> CollisionShape {
    let mut s = CollisionShape::default();
    s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 0.9375, max_z: 1.0 });
    s
}

fn trapdoor_collider(meta: u8) -> CollisionShape {
    const T: f64 = 0.1875;
    let mut s = CollisionShape::default();
    if (meta & 4) == 0 {
        s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: T, max_z: 1.0 });
        return s;
    }
    match meta & 3 {
        0 => s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 1.0 - T, max_x: 1.0, max_y: 1.0, max_z: 1.0 }),
        1 => s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: T }),
        2 => s.add(AABB { min_x: 1.0 - T, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 }),
        3 => s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: T, max_y: 1.0, max_z: 1.0 }),
        _ => {}
    }
    s
}

// bed
fn bed_aabb(_meta: u8) -> AABB {
    AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 0.5625, max_z: 1.0 }
}
fn bed_collider(_meta: u8) -> CollisionShape {
    let mut s = CollisionShape::default();
    s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 0.5625, max_z: 1.0 });
    s
}

// fence
fn fence_collider(_meta: u8) -> CollisionShape {
    let mut s = CollisionShape::default();
    s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.5, max_z: 1.0 });
    s
}

// cake
fn cake_aabb(meta: u8) -> AABB {
    let x0 = ((1 + (meta as i32) * 2) as f64) / 16.0;
    AABB { min_x: x0, min_y: 0.0, min_z: 0.0625, max_x: 1.0 - 0.0625, max_y: 0.5 - 0.0625, max_z: 1.0 - 0.0625 }
}
fn cake_collider(meta: u8) -> CollisionShape {
    let x0 = ((1 + (meta as i32) * 2) as f64) / 16.0;
    let mut s = CollisionShape::default();
    s.add(AABB { min_x: x0, min_y: 0.0, min_z: 0.0625, max_x: 1.0 - 0.0625, max_y: 0.5 - 0.0625, max_z: 1.0 - 0.0625 });
    s
}

// repeater
fn repeater_aabb(_meta: u8) -> AABB {
    AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 0.125, max_z: 1.0 }
}
fn empty_collider(_meta: u8) -> CollisionShape {
    CollisionShape::default()
}

// button
fn button_aabb(meta: u8) -> AABB {
    let face = meta & 7;
    let pressed = (meta & 8) != 0;
    const LO: f64 = 0.375;
    const HI: f64 = 0.625;
    const HW: f64 = 0.1875;
    let depth = if pressed { 0.0625 } else { 0.125 };
    match face {
        1 => AABB { min_x: 0.0, min_y: LO, min_z: 0.5 - HW, max_x: depth, max_y: HI, max_z: 0.5 + HW },
        2 => AABB { min_x: 1.0 - depth, min_y: LO, min_z: 0.5 - HW, max_x: 1.0, max_y: HI, max_z: 0.5 + HW },
        3 => AABB { min_x: 0.5 - HW, min_y: LO, min_z: 0.0, max_x: 0.5 + HW, max_y: HI, max_z: depth },
        4 => AABB { min_x: 0.5 - HW, min_y: LO, min_z: 1.0 - depth, max_x: 0.5 + HW, max_y: HI, max_z: 1.0 },
        _ => AABB::default(),
    }
}

// lever
fn lever_aabb(meta: u8) -> AABB {
    const F: f64 = 0.1875;
    match meta & 7 {
        1 => AABB { min_x: 0.0, min_y: 0.2, min_z: 0.5 - F, max_x: F * 2.0, max_y: 0.8, max_z: 0.5 + F },
        2 => AABB { min_x: 1.0 - F * 2.0, min_y: 0.2, min_z: 0.5 - F, max_x: 1.0, max_y: 0.8, max_z: 0.5 + F },
        3 => AABB { min_x: 0.5 - F, min_y: 0.2, min_z: 0.0, max_x: 0.5 + F, max_y: 0.8, max_z: F * 2.0 },
        4 => AABB { min_x: 0.5 - F, min_y: 0.2, min_z: 1.0 - F * 2.0, max_x: 0.5 + F, max_y: 0.8, max_z: 1.0 },
        _ => {
            const G: f64 = 0.25;
            AABB { min_x: 0.5 - G, min_y: 0.0, min_z: 0.5 - G, max_x: 0.5 + G, max_y: 0.6, max_z: 0.5 + G }
        }
    }
}

// pressure plate
fn pressure_plate_aabb(meta: u8) -> AABB {
    const F: f64 = 0.0625;
    AABB {
        min_x: F,
        min_y: 0.0,
        min_z: F,
        max_x: 1.0 - F,
        max_y: if meta == 1 { 0.03125 } else { 0.0625 },
        max_z: 1.0 - F,
    }
}

// torch (normal + redstone, same box)
fn torch_aabb(meta: u8) -> AABB {
    const F: f64 = 0.15;
    match meta & 7 {
        1 => AABB { min_x: 0.0, min_y: 0.2, min_z: 0.5 - F, max_x: F * 2.0, max_y: 0.8, max_z: 0.5 + F },
        2 => AABB { min_x: 1.0 - F * 2.0, min_y: 0.2, min_z: 0.5 - F, max_x: 1.0, max_y: 0.8, max_z: 0.5 + F },
        3 => AABB { min_x: 0.5 - F, min_y: 0.2, min_z: 0.0, max_x: 0.5 + F, max_y: 0.8, max_z: F * 2.0 },
        4 => AABB { min_x: 0.5 - F, min_y: 0.2, min_z: 1.0 - F * 2.0, max_x: 0.5 + F, max_y: 0.8, max_z: 1.0 },
        _ => {
            const G: f64 = 0.1;
            AABB { min_x: 0.5 - G, min_y: 0.0, min_z: 0.5 - G, max_x: 0.5 + G, max_y: 0.6, max_z: 0.5 + G }
        }
    }
}

// rail
fn rail_aabb(_meta: u8) -> AABB {
    AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 0.125, max_z: 1.0 }
}

// redstone dust
fn redstone_dust_aabb(_meta: u8) -> AABB {
    AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 0.0625, max_z: 1.0 }
}

// farmland
// Collider is full cube; ray/selection use visual height 0.937
fn farmland_aabb(_meta: u8) -> AABB {
    AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 }
}

// crop
fn crop_aabb(_meta: u8) -> AABB {
    AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 0.25, max_z: 1.0 } // 4/16
}

// sapling / deadbush (f=0.4)
fn sapling_aabb(_meta: u8) -> AABB {
    const F: f32 = 0.4;
    AABB {
        min_x: (0.5 - F) as f64,
        min_y: 0.0,
        min_z: (0.5 - F) as f64,
        max_x: (0.5 + F) as f64,
        max_y: (F * 2.0) as f64,
        max_z: (0.5 + F) as f64,
    }
}

// tall grass
fn tall_grass_aabb(_meta: u8) -> AABB {
    const F: f32 = 0.4;
    AABB {
        min_x: (0.5 - F) as f64,
        min_y: 0.0,
        min_z: (0.5 - F) as f64,
        max_x: (0.5 + F) as f64,
        max_y: 0.8,
        max_z: (0.5 + F) as f64,
    }
}

// mushroom (f=0.2)
fn mushroom_aabb(_meta: u8) -> AABB {
    const F: f32 = 0.2;
    AABB {
        min_x: (0.5 - F) as f64,
        min_y: 0.0,
        min_z: (0.5 - F) as f64,
        max_x: (0.5 + F) as f64,
        max_y: (F * 2.0) as f64,
        max_z: (0.5 + F) as f64,
    }
}

// plant / flower (rose, dandelion) (f=0.2, h=f*3)
fn plant_aabb(_meta: u8) -> AABB {
    const F: f32 = 0.2;
    AABB {
        min_x: (0.5 - F) as f64,
        min_y: 0.0,
        min_z: (0.5 - F) as f64,
        max_x: (0.5 + F) as f64,
        max_y: (F * 3.0) as f64,
        max_z: (0.5 + F) as f64,
    }
}

// sugarcane
fn sugarcane_aabb(_meta: u8) -> AABB {
    const F: f32 = 0.375;
    AABB {
        min_x: (0.5 - F) as f64,
        min_y: 0.0,
        min_z: (0.5 - F) as f64,
        max_x: (0.5 + F) as f64,
        max_y: 1.0,
        max_z: (0.5 + F) as f64,
    }
}

// Liquids have no collision
fn liquid_aabb(_meta: u8) -> AABB {
    AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 0.0, max_y: 0.0, max_z: 0.0 }
}

// piston head
fn piston_head_aabb(meta: u8) -> AABB {
    match meta & 7 {
        0 => AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 0.25, max_z: 1.0 },
        1 => AABB { min_x: 0.0, min_y: 0.75, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 },
        2 => AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 0.25 },
        3 => AABB { min_x: 0.0, min_y: 0.0, min_z: 0.75, max_x: 1.0, max_y: 1.0, max_z: 1.0 },
        4 => AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 0.25, max_y: 1.0, max_z: 1.0 },
        5 => AABB { min_x: 0.75, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 },
        _ => AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 },
    }
}
fn piston_head_collider(meta: u8) -> CollisionShape {
    let mut s = CollisionShape::default();
    match meta & 7 {
        0 => {
            s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 0.25, max_z: 1.0 });
            s.add(AABB { min_x: 0.375, min_y: 0.25, min_z: 0.375, max_x: 0.625, max_y: 1.0, max_z: 0.625 });
        }
        1 => {
            s.add(AABB { min_x: 0.0, min_y: 0.75, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 });
            s.add(AABB { min_x: 0.375, min_y: 0.0, min_z: 0.375, max_x: 0.625, max_y: 0.75, max_z: 0.625 });
        }
        2 => {
            s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 0.25 });
            s.add(AABB { min_x: 0.25, min_y: 0.375, min_z: 0.25, max_x: 0.75, max_y: 0.625, max_z: 1.0 });
        }
        3 => {
            s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.75, max_x: 1.0, max_y: 1.0, max_z: 1.0 });
            s.add(AABB { min_x: 0.25, min_y: 0.375, min_z: 0.0, max_x: 0.75, max_y: 0.625, max_z: 0.75 });
        }
        4 => {
            s.add(AABB { min_x: 0.0, min_y: 0.0, min_z: 0.0, max_x: 0.25, max_y: 1.0, max_z: 1.0 });
            s.add(AABB { min_x: 0.375, min_y: 0.25, min_z: 0.25, max_x: 0.625, max_y: 0.75, max_z: 1.0 });
        }
        5 => {
            s.add(AABB { min_x: 0.75, min_y: 0.0, min_z: 0.0, max_x: 1.0, max_y: 1.0, max_z: 1.0 });
            s.add(AABB { min_x: 0.0, min_y: 0.375, min_z: 0.25, max_x: 0.75, max_y: 0.625, max_z: 0.75 });
        }
        _ => {}
    }
    s
}

pub fn get_block_drops(block_id: BlockType, meta: u8, rng: &mut Random) -> Vec<ItemStack> {
    let mut drops = Vec::new();

    if block_id == BLOCK_AIR {
        return drops;
    }

    // headache: crops drop multiple items of different types (wheat + seeds)
    if block_id == BLOCK_CROP_WHEAT {
        if i32::from(meta) == MAX_CROP_SIZE {
            drops.push(ItemStack { id: items::WHEAT, count: 1, data: 0 });
        }

        for _ in 0..3 {
            if rng.next_int_bound(15) <= i32::from(meta) {
                drops.push(ItemStack { id: items::SEEDS_WHEAT, count: 1, data: 0 });
            }
        }

        return drops;
    }

    let behavior = &block_properties::block_behaviors()[block_id.0 as usize];
    let count: i32 = behavior.quantity_dropped.map_or(1, |f| i32::from(f(rng)));
    let damage: ItemDamage = behavior.damage_dropped.map_or(0, |f| f(meta));

    for _ in 0..count {
        let id = behavior.id_dropped.map_or(ItemId(block_id.0 as i16), |f| f(meta, rng));

        if id.0 > 0 {
            drops.push(ItemStack { id, count: 1, data: damage });
        }
    }

    drops
}

fn wood_door_on_block_activated(world: &mut WorldManager, pos: Int3) -> bool {
    let meta = world.get_metadata(pos);
    if (meta & 8) != 0 {
        // We are the top half of the door
        if world.get_block_id(Int3::new(pos.x, pos.y - 1, pos.z)) != BLOCK_DOOR_WOOD {
            // Below us is not the bottom of a door! This is bad!
            return false;
        }
        // Recall this function on the bottom of the door
        wood_door_on_block_activated(world, Int3::new(pos.x, pos.y - 1, pos.z));
        return false;
    }
    // We are the top half so lets open
    let top = Int3::new(pos.x, pos.y + 1, pos.z);
    if world.get_block_id(top) == BLOCK_DOOR_WOOD && (world.get_metadata(top) & 8) != 0 {
        world.set_meta(top, (((meta as i32) ^ 4) + 8) as u8);
    }
    world.set_meta(pos, meta ^ 4); // XOR bit 2; flips open/closed
    false
}

pub fn register_all() {
    let mut properties = [BlockProperties::default(); 256];
    let mut behaviors = [BlockBehavior::default(); 256];

    // Default all behavior slots to full-cube before per-block overrides
    for behavior in behaviors.iter_mut() {
        behavior.get_selection_box = Some(default_aabb);
        behavior.get_ray_bounds = Some(default_aabb);
        behavior.get_collider = Some(default_collider);
    }

    // block properties

    // Air
    properties[BLOCK_AIR.0 as usize] = BlockProperties {
        material: Material::air(),
        light_opacity: 0,
        hardness: 0.0,
        resistance: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        can_block_grass: false,
        enable_stats: false,
        ..Default::default()
    };

    // Stone
    properties[BLOCK_STONE.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 1.5,
        resistance: 10.0,
        ..Default::default()
    };

    // Grass
    properties[BLOCK_GRASS.0 as usize] = BlockProperties {
        material: Material::grass(),
        step_sound: StepSound::Grass,
        light_opacity: 255,
        hardness: 0.6,
        ticks_on_load: true,
        ..Default::default()
    };

    // Dirt
    properties[BLOCK_DIRT.0 as usize] = BlockProperties {
        material: Material::ground(),
        step_sound: StepSound::Gravel,
        light_opacity: 255,
        hardness: 0.5,
        ..Default::default()
    };

    // Cobblestone
    properties[BLOCK_COBBLESTONE.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 2.0,
        resistance: 10.0,
        ..Default::default()
    };

    // Planks (Oak Wood)
    properties[BLOCK_PLANKS.0 as usize] = BlockProperties {
        material: Material::wood(),
        step_sound: StepSound::Wood,
        light_opacity: 255,
        hardness: 2.0,
        resistance: 5.0,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Sapling
    properties[BLOCK_SAPLING.0 as usize] = BlockProperties {
        material: Material::plants(),
        step_sound: StepSound::Grass,
        light_opacity: 0,
        hardness: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ticks_on_load: true,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Bedrock
    properties[BLOCK_BEDROCK.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        hardness: -1.0, // unbreakable
        resistance: 6000000.0,
        enable_stats: false,
        ..Default::default()
    };

    // Water (flowing)
    properties[BLOCK_WATER_FLOWING.0 as usize] = BlockProperties {
        material: Material::water(),
        light_opacity: 3,
        hardness: 100.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Water (still/stationary)
    properties[BLOCK_WATER_STILL.0 as usize] = BlockProperties {
        material: Material::water(),
        light_opacity: 3,
        hardness: 100.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Lava (flowing)
    properties[BLOCK_LAVA_FLOWING.0 as usize] = BlockProperties {
        material: Material::lava(),
        light_emission: 15, // setLightValue(1.0f) -> 15*1.0 = 15
        light_opacity: 255,
        hardness: 0.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Lava (still/stationary)
    properties[BLOCK_LAVA_STILL.0 as usize] = BlockProperties {
        material: Material::lava(),
        light_emission: 15,
        light_opacity: 255,
        hardness: 100.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Sand
    properties[BLOCK_SAND.0 as usize] = BlockProperties {
        material: Material::sand(),
        step_sound: StepSound::Sand,
        light_opacity: 255,
        hardness: 0.5,
        ..Default::default()
    };

    // Gravel
    properties[BLOCK_GRAVEL.0 as usize] = BlockProperties {
        material: Material::sand(),
        step_sound: StepSound::Gravel,
        light_opacity: 255,
        hardness: 0.6,
        ..Default::default()
    };

    // Gold Ore
    properties[BLOCK_ORE_GOLD.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 3.0,
        resistance: 5.0,
        ..Default::default()
    };

    // Iron Ore
    properties[BLOCK_ORE_IRON.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 3.0,
        resistance: 5.0,
        ..Default::default()
    };

    // Coal Ore
    properties[BLOCK_ORE_COAL.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 3.0,
        resistance: 5.0,
        ..Default::default()
    };

    // Wood Log
    properties[BLOCK_LOG.0 as usize] = BlockProperties {
        material: Material::wood(),
        step_sound: StepSound::Wood,
        light_opacity: 255,
        hardness: 2.0,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Leaves
    properties[BLOCK_LEAVES.0 as usize] = BlockProperties {
        material: Material::leaves(),
        step_sound: StepSound::Grass,
        light_opacity: 1,
        hardness: 0.2,
        is_opaque_cube: false,
        is_normal_cube: false,
        ticks_on_load: true,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Sponge
    properties[BLOCK_SPONGE.0 as usize] = BlockProperties {
        material: Material::sponge(),
        step_sound: StepSound::Grass,
        light_opacity: 255,
        hardness: 0.6,
        ..Default::default()
    };

    // Glass
    properties[BLOCK_GLASS.0 as usize] = BlockProperties {
        material: Material::glass(),
        step_sound: StepSound::Glass,
        light_opacity: 0,
        hardness: 0.3,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ..Default::default()
    };

    // Lapis Lazuli Ore
    properties[BLOCK_ORE_LAPIS_LAZULI.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 3.0,
        resistance: 5.0,
        ..Default::default()
    };

    // Lapis Lazuli Block
    properties[BLOCK_LAPIS_LAZULI.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 3.0,
        resistance: 5.0,
        ..Default::default()
    };

    // Dispenser
    properties[BLOCK_DISPENSER.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 3.5,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Sandstone
    properties[BLOCK_SANDSTONE.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 0.8,
        ..Default::default()
    };

    // Note Block
    properties[BLOCK_NOTEBLOCK.0 as usize] = BlockProperties {
        material: Material::wood(),
        step_sound: StepSound::Wood,
        light_opacity: 255,
        hardness: 0.8,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Bed
    properties[BLOCK_BED.0 as usize] = BlockProperties {
        material: Material::cloth(),
        step_sound: StepSound::Wood,
        light_opacity: 0,
        hardness: 0.2,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Powered Rail (Golden Rail)
    properties[BLOCK_RAIL_POWERED.0 as usize] = BlockProperties {
        material: Material::circuits(),
        step_sound: StepSound::Stone,
        light_opacity: 0,
        hardness: 0.7,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Detector Rail
    properties[BLOCK_RAIL_DETECTOR.0 as usize] = BlockProperties {
        material: Material::circuits(),
        step_sound: StepSound::Stone,
        light_opacity: 0,
        hardness: 0.7,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Sticky Piston Base
    properties[BLOCK_PISTON_STICKY.0 as usize] = BlockProperties {
        material: Material::piston(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 0.5,
        is_opaque_cube: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Cobweb
    properties[BLOCK_COBWEB.0 as usize] = BlockProperties {
        material: Material::web(),
        step_sound: StepSound::Cloth,
        light_opacity: 1,
        hardness: 4.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ..Default::default()
    };

    // Tall Grass
    properties[BLOCK_TALLGRASS.0 as usize] = BlockProperties {
        material: Material::plants(),
        step_sound: StepSound::Grass,
        light_opacity: 0,
        hardness: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ..Default::default()
    };

    // Dead Bush
    properties[BLOCK_DEADBUSH.0 as usize] = BlockProperties {
        material: Material::plants(),
        step_sound: StepSound::Grass,
        light_opacity: 0,
        hardness: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ..Default::default()
    };

    // Piston Base
    properties[BLOCK_PISTON.0 as usize] = BlockProperties {
        material: Material::piston(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 0.5,
        is_opaque_cube: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Piston Extension (head)
    properties[BLOCK_PISTON_HEAD.0 as usize] = BlockProperties {
        material: Material::piston(),
        step_sound: StepSound::Stone,
        light_opacity: 0,
        hardness: 0.5,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Wool (Cloth)
    properties[BLOCK_WOOL.0 as usize] = BlockProperties {
        material: Material::cloth(),
        step_sound: StepSound::Cloth,
        light_opacity: 255,
        hardness: 0.8,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Piston Moving (tile entity placeholder)
    properties[BLOCK_PISTON_MOVING.0 as usize] = BlockProperties {
        material: Material::piston(),
        light_opacity: 0,
        hardness: -1.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        enable_stats: false,
        ..Default::default()
    };

    // Dandelion (Yellow Flower)
    properties[BLOCK_DANDELION.0 as usize] = BlockProperties {
        material: Material::plants(),
        step_sound: StepSound::Grass,
        light_opacity: 0,
        hardness: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ..Default::default()
    };

    // Rose (Red Flower)
    properties[BLOCK_ROSE.0 as usize] = BlockProperties {
        material: Material::plants(),
        step_sound: StepSound::Grass,
        light_opacity: 0,
        hardness: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ..Default::default()
    };

    // Brown Mushroom
    properties[BLOCK_MUSHROOM_BROWN.0 as usize] = BlockProperties {
        material: Material::plants(),
        step_sound: StepSound::Grass,
        light_emission: 1,
        light_opacity: 0,
        hardness: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ..Default::default()
    };

    // Red Mushroom
    properties[BLOCK_MUSHROOM_RED.0 as usize] = BlockProperties {
        material: Material::plants(),
        step_sound: StepSound::Grass,
        light_opacity: 0,
        hardness: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ..Default::default()
    };

    // Gold Block
    properties[BLOCK_GOLD.0 as usize] = BlockProperties {
        material: Material::iron(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 3.0,
        resistance: 10.0,
        ..Default::default()
    };

    // Iron Block
    properties[BLOCK_IRON.0 as usize] = BlockProperties {
        material: Material::iron(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 5.0,
        resistance: 10.0,
        ..Default::default()
    };

    // Double Stone Slab
    properties[BLOCK_DOUBLE_SLAB.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 2.0,
        resistance: 10.0,
        ..Default::default()
    };

    // Stone Slab (single)
    properties[BLOCK_SLAB.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 2.0,
        resistance: 10.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ..Default::default()
    };

    // Bricks
    properties[BLOCK_BRICKS.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 2.0,
        resistance: 10.0,
        ..Default::default()
    };

    // TNT
    properties[BLOCK_TNT.0 as usize] = BlockProperties {
        material: Material::tnt(),
        step_sound: StepSound::Grass,
        light_opacity: 255,
        hardness: 0.0,
        ..Default::default()
    };

    // Bookshelf
    properties[BLOCK_BOOKSHELF.0 as usize] = BlockProperties {
        material: Material::wood(),
        step_sound: StepSound::Wood,
        light_opacity: 255,
        hardness: 1.5,
        ..Default::default()
    };

    // Mossy Cobblestone
    properties[BLOCK_COBBLESTONE_MOSSY.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 2.0,
        resistance: 10.0,
        ..Default::default()
    };

    // Obsidian
    properties[BLOCK_OBSIDIAN.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 10.0,
        resistance: 2000.0,
        ..Default::default()
    };

    // Torch
    properties[BLOCK_TORCH.0 as usize] = BlockProperties {
        material: Material::circuits(),
        step_sound: StepSound::Wood,
        light_emission: 14,
        light_opacity: 0,
        hardness: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Fire
    properties[BLOCK_FIRE.0 as usize] = BlockProperties {
        material: Material::fire(),
        light_emission: 15,
        light_opacity: 0,
        hardness: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        can_block_grass: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Monster Spawner
    properties[BLOCK_MOB_SPAWNER.0 as usize] = BlockProperties {
        material: Material::iron(),
        step_sound: StepSound::Stone,
        light_opacity: 0,
        hardness: 5.0,
        enable_stats: false,
        ..Default::default()
    };

    // Oak Wood Stairs
    properties[BLOCK_STAIRS_WOOD.0 as usize] = BlockProperties {
        material: Material::wood(),
        step_sound: StepSound::Wood,
        light_opacity: 255,
        hardness: 2.0,
        resistance: 5.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Chest
    properties[BLOCK_CHEST.0 as usize] = BlockProperties {
        material: Material::wood(),
        step_sound: StepSound::Wood,
        light_opacity: 0,
        hardness: 2.5,
        is_opaque_cube: false,
        is_normal_cube: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Redstone Wire
    properties[BLOCK_REDSTONE.0 as usize] = BlockProperties {
        material: Material::circuits(),
        step_sound: StepSound::Stone,
        light_opacity: 0,
        hardness: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Diamond Ore
    properties[BLOCK_ORE_DIAMOND.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 3.0,
        resistance: 5.0,
        ..Default::default()
    };

    // Diamond Block
    properties[BLOCK_DIAMOND.0 as usize] = BlockProperties {
        material: Material::iron(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 5.0,
        resistance: 10.0,
        ..Default::default()
    };

    // Crafting Table (Workbench)
    properties[BLOCK_CRAFTING_TABLE.0 as usize] = BlockProperties {
        material: Material::wood(),
        step_sound: StepSound::Wood,
        light_opacity: 255,
        hardness: 2.5,
        ..Default::default()
    };

    // Crops / Wheat
    properties[BLOCK_CROP_WHEAT.0 as usize] = BlockProperties {
        material: Material::plants(),
        step_sound: StepSound::Grass,
        light_opacity: 0,
        hardness: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ticks_on_load: true,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Farmland (Tilled Field)
    properties[BLOCK_FARMLAND.0 as usize] = BlockProperties {
        material: Material::ground(),
        step_sound: StepSound::Gravel,
        light_opacity: 255,
        hardness: 0.6,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ..Default::default()
    };

    // Furnace (idle)
    properties[BLOCK_FURNACE.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 3.5,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Furnace (active/lit)
    properties[BLOCK_FURNACE_LIT.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_emission: 13,
        light_opacity: 255,
        hardness: 3.5,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Sign (standing)
    properties[BLOCK_SIGN.0 as usize] = BlockProperties {
        material: Material::wood(),
        step_sound: StepSound::Wood,
        light_opacity: 0,
        hardness: 1.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Wooden Door
    properties[BLOCK_DOOR_WOOD.0 as usize] = BlockProperties {
        material: Material::wood(),
        step_sound: StepSound::Wood,
        light_opacity: 0,
        hardness: 3.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Ladder
    properties[BLOCK_LADDER.0 as usize] = BlockProperties {
        material: Material::circuits(),
        step_sound: StepSound::Wood,
        light_opacity: 0,
        hardness: 0.4,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Rail (normal)
    properties[BLOCK_RAIL.0 as usize] = BlockProperties {
        material: Material::circuits(),
        step_sound: StepSound::Stone,
        light_opacity: 0,
        hardness: 0.7,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Cobblestone Stairs
    properties[BLOCK_STAIRS_COBBLESTONE.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 2.0,
        resistance: 10.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Wall Sign
    properties[BLOCK_SIGN_WALL.0 as usize] = BlockProperties {
        material: Material::wood(),
        step_sound: StepSound::Wood,
        light_opacity: 0,
        hardness: 1.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Lever
    properties[BLOCK_LEVER.0 as usize] = BlockProperties {
        material: Material::circuits(),
        step_sound: StepSound::Wood,
        light_opacity: 0,
        hardness: 0.5,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Stone Pressure Plate
    properties[BLOCK_PRESSURE_PLATE_STONE.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 0,
        hardness: 0.5,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Iron Door
    properties[BLOCK_DOOR_IRON.0 as usize] = BlockProperties {
        material: Material::iron(),
        step_sound: StepSound::Stone,
        light_opacity: 0,
        hardness: 5.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Wooden Pressure Plate
    properties[BLOCK_PRESSURE_PLATE_WOOD.0 as usize] = BlockProperties {
        material: Material::wood(),
        step_sound: StepSound::Wood,
        light_opacity: 0,
        hardness: 0.5,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Redstone Ore
    properties[BLOCK_ORE_REDSTONE_OFF.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 3.0,
        resistance: 5.0,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Redstone Ore (glowing/lit)
    properties[BLOCK_ORE_REDSTONE_ON.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_emission: 9,
        light_opacity: 255,
        hardness: 3.0,
        resistance: 5.0,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Redstone Torch (off)
    properties[BLOCK_REDSTONE_TORCH_OFF.0 as usize] = BlockProperties {
        material: Material::circuits(),
        step_sound: StepSound::Wood,
        light_opacity: 0,
        hardness: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Redstone Torch (on)
    properties[BLOCK_REDSTONE_TORCH_ON.0 as usize] = BlockProperties {
        material: Material::circuits(),
        step_sound: StepSound::Wood,
        light_emission: 7,
        light_opacity: 0,
        hardness: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Stone Button
    properties[BLOCK_BUTTON_STONE.0 as usize] = BlockProperties {
        material: Material::circuits(),
        step_sound: StepSound::Stone,
        light_opacity: 0,
        hardness: 0.5,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Snow (layer)
    properties[BLOCK_SNOW_LAYER.0 as usize] = BlockProperties {
        material: Material::snow_layer(),
        step_sound: StepSound::Cloth,
        light_opacity: 0,
        hardness: 0.1,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        can_block_grass: false,
        ..Default::default()
    };

    // Ice
    properties[BLOCK_ICE.0 as usize] = BlockProperties {
        material: Material::ice(),
        step_sound: StepSound::Glass,
        light_opacity: 3,
        hardness: 0.5,
        slipperiness: 0.98,
        ..Default::default()
    };

    // Snow Block
    properties[BLOCK_SNOW.0 as usize] = BlockProperties {
        material: Material::snow_block(),
        step_sound: StepSound::Cloth,
        light_opacity: 255,
        hardness: 0.2,
        ..Default::default()
    };

    // Cactus
    properties[BLOCK_CACTUS.0 as usize] = BlockProperties {
        material: Material::cactus(),
        step_sound: StepSound::Cloth,
        light_opacity: 0,
        hardness: 0.4,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ticks_on_load: true,
        ..Default::default()
    };

    // Clay Block
    properties[BLOCK_CLAY.0 as usize] = BlockProperties {
        material: Material::clay(),
        step_sound: StepSound::Gravel,
        light_opacity: 255,
        hardness: 0.6,
        ..Default::default()
    };

    // Sugar Cane (Reed)
    properties[BLOCK_SUGARCANE.0 as usize] = BlockProperties {
        material: Material::plants(),
        step_sound: StepSound::Grass,
        light_opacity: 0,
        hardness: 0.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ticks_on_load: true,
        enable_stats: false,
        ..Default::default()
    };

    // Jukebox
    properties[BLOCK_JUKEBOX.0 as usize] = BlockProperties {
        material: Material::wood(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 2.0,
        resistance: 10.0,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Fence
    properties[BLOCK_FENCE.0 as usize] = BlockProperties {
        material: Material::wood(),
        step_sound: StepSound::Wood,
        light_opacity: 0,
        hardness: 2.0,
        resistance: 5.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Pumpkin
    properties[BLOCK_PUMPKIN.0 as usize] = BlockProperties {
        material: Material::pumpkin(),
        step_sound: StepSound::Wood,
        light_opacity: 255,
        hardness: 1.0,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Netherrack
    properties[BLOCK_NETHERRACK.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Stone,
        light_opacity: 255,
        hardness: 0.4,
        ..Default::default()
    };

    // Soul Sand
    properties[BLOCK_SOULSAND.0 as usize] = BlockProperties {
        material: Material::sand(),
        step_sound: StepSound::Sand,
        light_opacity: 255,
        hardness: 0.5,
        ..Default::default()
    };

    // Glowstone
    properties[BLOCK_GLOWSTONE.0 as usize] = BlockProperties {
        material: Material::rock(),
        step_sound: StepSound::Glass,
        light_emission: 15,
        light_opacity: 255,
        hardness: 0.3,
        ..Default::default()
    };

    // Nether Portal
    properties[BLOCK_NETHER_PORTAL.0 as usize] = BlockProperties {
        material: Material::portal(),
        step_sound: StepSound::Glass,
        light_emission: 11,
        light_opacity: 0,
        hardness: -1.0,
        is_collidable: false,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        ..Default::default()
    };

    // Jack-o-Lantern (Lit Pumpkin)
    properties[BLOCK_PUMPKIN_LIT.0 as usize] = BlockProperties {
        material: Material::pumpkin(),
        step_sound: StepSound::Wood,
        light_emission: 15,
        light_opacity: 255,
        hardness: 1.0,
        notify_neighbors_on_meta_change: false,
        ..Default::default()
    };

    // Cake
    properties[BLOCK_CAKE.0 as usize] = BlockProperties {
        material: Material::cake(),
        step_sound: StepSound::Cloth,
        light_opacity: 0,
        hardness: 0.5,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Redstone Repeater (off)
    properties[BLOCK_REDSTONE_REPEATER_OFF.0 as usize] = BlockProperties {
        material: Material::circuits(),
        step_sound: StepSound::Wood,
        light_opacity: 0,
        hardness: 0.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Redstone Repeater (on)
    properties[BLOCK_REDSTONE_REPEATER_ON.0 as usize] = BlockProperties {
        material: Material::circuits(),
        step_sound: StepSound::Wood,
        light_emission: 9,
        light_opacity: 0,
        hardness: 0.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // Trapdoor
    properties[BLOCK_TRAPDOOR.0 as usize] = BlockProperties {
        material: Material::wood(),
        step_sound: StepSound::Wood,
        light_opacity: 0,
        hardness: 3.0,
        is_opaque_cube: false,
        is_normal_cube: false,
        render_as_normal_block: false,
        notify_neighbors_on_meta_change: false,
        enable_stats: false,
        ..Default::default()
    };

    // block behaviors (non-default shapes)

    // Liquids/zero-size AABBs
    behaviors[BLOCK_WATER_FLOWING.0 as usize] = BlockBehavior {
        get_selection_box: Some(liquid_aabb),
        get_ray_bounds: Some(liquid_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };
    behaviors[BLOCK_WATER_STILL.0 as usize] = BlockBehavior {
        get_selection_box: Some(liquid_aabb),
        get_ray_bounds: Some(liquid_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };
    behaviors[BLOCK_LAVA_FLOWING.0 as usize] = BlockBehavior {
        get_selection_box: Some(liquid_aabb),
        get_ray_bounds: Some(liquid_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };
    behaviors[BLOCK_LAVA_STILL.0 as usize] = BlockBehavior {
        get_selection_box: Some(liquid_aabb),
        get_ray_bounds: Some(liquid_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };
    behaviors[BLOCK_COBWEB.0 as usize] = BlockBehavior {
        get_collider: Some(empty_collider),
        ..Default::default()
    };

    // Rails
    behaviors[BLOCK_RAIL.0 as usize] = BlockBehavior {
        get_ray_bounds: Some(rail_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };
    behaviors[BLOCK_RAIL_POWERED.0 as usize] = BlockBehavior {
        get_ray_bounds: Some(rail_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };
    behaviors[BLOCK_RAIL_DETECTOR.0 as usize] = BlockBehavior {
        get_ray_bounds: Some(rail_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };

    // Redstone dust
    behaviors[BLOCK_REDSTONE.0 as usize] = BlockBehavior {
        get_selection_box: Some(redstone_dust_aabb),
        get_ray_bounds: Some(redstone_dust_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };

    // Farmland
    behaviors[BLOCK_FARMLAND.0 as usize] = BlockBehavior {
        get_selection_box: Some(farmland_aabb),
        get_ray_bounds: Some(farmland_aabb),
        get_collider: Some(farmland_collider),
        ..Default::default()
    };

    // Crops
    behaviors[BLOCK_CROP_WHEAT.0 as usize] = BlockBehavior {
        get_selection_box: Some(crop_aabb),
        get_ray_bounds: Some(crop_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };

    // Sapling
    behaviors[BLOCK_SAPLING.0 as usize] = BlockBehavior {
        get_selection_box: Some(sapling_aabb),
        get_ray_bounds: Some(sapling_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };

    // Tall grass
    behaviors[BLOCK_TALLGRASS.0 as usize] = BlockBehavior {
        get_selection_box: Some(tall_grass_aabb),
        get_ray_bounds: Some(tall_grass_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };

    // Mushrooms
    behaviors[BLOCK_MUSHROOM_BROWN.0 as usize] = BlockBehavior {
        get_selection_box: Some(mushroom_aabb),
        get_ray_bounds: Some(mushroom_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };
    behaviors[BLOCK_MUSHROOM_RED.0 as usize] = BlockBehavior {
        get_selection_box: Some(mushroom_aabb),
        get_ray_bounds: Some(mushroom_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };

    // Flowers (rose, dandelion)
    behaviors[BLOCK_ROSE.0 as usize] = BlockBehavior {
        get_selection_box: Some(plant_aabb),
        get_ray_bounds: Some(plant_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };
    behaviors[BLOCK_DANDELION.0 as usize] = BlockBehavior {
        get_selection_box: Some(plant_aabb),
        get_ray_bounds: Some(plant_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };

    // Dead bush
    behaviors[BLOCK_DEADBUSH.0 as usize] = BlockBehavior {
        get_selection_box: Some(sapling_aabb), // same f=0.4 box as sapling
        get_ray_bounds: Some(sapling_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };

    // Sugar cane
    behaviors[BLOCK_SUGARCANE.0 as usize] = BlockBehavior {
        get_selection_box: Some(sugarcane_aabb),
        get_ray_bounds: Some(sugarcane_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };

    behaviors[BLOCK_SLAB.0 as usize] = BlockBehavior {
        get_selection_box: Some(slab_aabb),
        get_ray_bounds: Some(slab_aabb),
        get_collider: Some(slab_collider),
        ..Default::default()
    };

    behaviors[BLOCK_STAIRS_WOOD.0 as usize] = BlockBehavior {
        get_collider: Some(stair_collider),
        // ray/selection stay as defaultAABB (full cube is correct)
        ..Default::default()
    };
    behaviors[BLOCK_STAIRS_COBBLESTONE.0 as usize] = BlockBehavior {
        get_collider: Some(stair_collider),
        ..Default::default()
    };

    behaviors[BLOCK_CACTUS.0 as usize] = BlockBehavior {
        get_selection_box: Some(cactus_aabb),
        get_ray_bounds: Some(cactus_aabb),
        get_collider: Some(cactus_collider),
        ..Default::default()
    };

    behaviors[BLOCK_SNOW_LAYER.0 as usize] = BlockBehavior {
        get_ray_bounds: Some(snow_layer_aabb),
        get_collider: Some(snow_layer_collider),
        // getSelectionBox stays defaultAABB
        ..Default::default()
    };

    behaviors[BLOCK_LADDER.0 as usize] = BlockBehavior {
        get_selection_box: Some(ladder_aabb),
        get_ray_bounds: Some(ladder_aabb),
        get_collider: Some(ladder_collider),
        ..Default::default()
    };

    behaviors[BLOCK_DOOR_WOOD.0 as usize] = BlockBehavior {
        get_selection_box: Some(door_aabb),
        get_ray_bounds: Some(door_aabb),
        get_collider: Some(door_collider),
        ..Default::default()
    };
    behaviors[BLOCK_DOOR_IRON.0 as usize] = BlockBehavior {
        get_selection_box: Some(door_aabb),
        get_ray_bounds: Some(door_aabb),
        get_collider: Some(door_collider),
        ..Default::default()
    };

    behaviors[BLOCK_TRAPDOOR.0 as usize] = BlockBehavior {
        get_selection_box: Some(trapdoor_aabb),
        get_ray_bounds: Some(trapdoor_aabb),
        get_collider: Some(trapdoor_collider),
        ..Default::default()
    };

    behaviors[BLOCK_BED.0 as usize] = BlockBehavior {
        get_selection_box: Some(bed_aabb),
        get_ray_bounds: Some(bed_aabb),
        get_collider: Some(bed_collider),
        ..Default::default()
    };

    behaviors[BLOCK_FENCE.0 as usize] = BlockBehavior {
        get_collider: Some(fence_collider),
        // ray/selection stay as defaultAABB (full cube)
        ..Default::default()
    };

    behaviors[BLOCK_CAKE.0 as usize] = BlockBehavior {
        get_selection_box: Some(cake_aabb),
        get_ray_bounds: Some(cake_aabb),
        get_collider: Some(cake_collider),
        ..Default::default()
    };

    behaviors[BLOCK_REDSTONE_REPEATER_OFF.0 as usize] = BlockBehavior {
        get_selection_box: Some(repeater_aabb),
        get_ray_bounds: Some(repeater_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };
    behaviors[BLOCK_REDSTONE_REPEATER_ON.0 as usize] = BlockBehavior {
        get_selection_box: Some(repeater_aabb),
        get_ray_bounds: Some(repeater_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };

    behaviors[BLOCK_BUTTON_STONE.0 as usize] = BlockBehavior {
        get_selection_box: Some(button_aabb),
        get_ray_bounds: Some(button_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };

    behaviors[BLOCK_LEVER.0 as usize] = BlockBehavior {
        get_ray_bounds: Some(lever_aabb),
        get_collider: Some(empty_collider),
        // getSelectionBox stays defaultAABB
        ..Default::default()
    };

    behaviors[BLOCK_PRESSURE_PLATE_STONE.0 as usize] = BlockBehavior {
        get_selection_box: Some(pressure_plate_aabb),
        get_ray_bounds: Some(pressure_plate_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };
    behaviors[BLOCK_PRESSURE_PLATE_WOOD.0 as usize] = BlockBehavior {
        get_selection_box: Some(pressure_plate_aabb),
        get_ray_bounds: Some(pressure_plate_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };

    behaviors[BLOCK_TORCH.0 as usize] = BlockBehavior {
        get_selection_box: Some(torch_aabb),
        get_ray_bounds: Some(torch_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };
    behaviors[BLOCK_REDSTONE_TORCH_OFF.0 as usize] = BlockBehavior {
        get_selection_box: Some(torch_aabb),
        get_ray_bounds: Some(torch_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };
    behaviors[BLOCK_REDSTONE_TORCH_ON.0 as usize] = BlockBehavior {
        get_selection_box: Some(torch_aabb),
        get_ray_bounds: Some(torch_aabb),
        get_collider: Some(empty_collider),
        ..Default::default()
    };

    behaviors[BLOCK_PISTON_HEAD.0 as usize] = BlockBehavior {
        get_selection_box: Some(piston_head_aabb),
        get_ray_bounds: Some(piston_head_aabb),
        get_collider: Some(piston_head_collider),
        ..Default::default()
    };

    // specific behavioral overrides
    behaviors[BLOCK_WATER_FLOWING.0 as usize].velocity_to_add_to_entity =
        Some(|world: &mut WorldManager, pos: Int3, push_vector: &mut Vec3| {
            let flow_vector = get_fluid_flow_vector(world, pos);
            *push_vector += flow_vector;
        });
    behaviors[BLOCK_WATER_STILL.0 as usize].velocity_to_add_to_entity =
        Some(|world: &mut WorldManager, pos: Int3, push_vector: &mut Vec3| {
            let flow_vector = get_fluid_flow_vector(world, pos);
            *push_vector += flow_vector;
        });
    behaviors[BLOCK_CACTUS.0 as usize].on_entity_collided_with_block =
        Some(|_world: &mut WorldManager, _pos: Int3, entity: &mut Entity| {
            entity.attack_entity_from(None, 1);
        });
    behaviors[BLOCK_COBWEB.0 as usize].on_entity_collided_with_block =
        Some(|_world: &mut WorldManager, _pos: Int3, entity: &mut Entity| {
            entity.in_web = true;
        });
    behaviors[BLOCK_SOULSAND.0 as usize].on_entity_collided_with_block =
        Some(|_world: &mut WorldManager, _pos: Int3, entity: &mut Entity| {
            entity.motion_x *= 0.4;
            entity.motion_z *= 0.4;
        });

    // placement overrides
    behaviors[BLOCK_TORCH.0 as usize].on_block_placed =
        Some(|world: &mut WorldManager, pos: Int3, _placer: &mut Entity, face: FaceDirection| {
            let mut meta = world.get_metadata(pos);
            if face == packet_data::Y_PLUS
                && (world.is_block_normal_cube(Int3::new(pos.x, pos.y - 1, pos.z))
                    || world.get_block_id(Int3::new(pos.x, pos.y - 1, pos.z)) == BLOCK_FENCE)
            {
                meta = 5;
            }
            if face == packet_data::Z_MINUS && world.is_block_normal_cube(Int3::new(pos.x, pos.y, pos.z + 1)) {
                meta = 4;
            }
            if face == packet_data::Z_PLUS && world.is_block_normal_cube(Int3::new(pos.x, pos.y, pos.z - 1)) {
                meta = 3;
            }
            if face == packet_data::X_MINUS && world.is_block_normal_cube(Int3::new(pos.x + 1, pos.y, pos.z)) {
                meta = 2;
            }
            if face == packet_data::X_PLUS && world.is_block_normal_cube(Int3::new(pos.x - 1, pos.y, pos.z)) {
                meta = 1;
            }
            world.set_meta(pos, meta);
        });

    // for when the block is interacted with!
    behaviors[BLOCK_DOOR_WOOD.0 as usize].on_block_activated = Some(wood_door_on_block_activated);

    // --------------- block drops, only exceptions are included (something that doesn't drop itself) ---------------
    behaviors[BLOCK_STONE.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { ItemId(BLOCK_COBBLESTONE.0 as i16) });
    behaviors[BLOCK_GRASS.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { ItemId(BLOCK_DIRT.0 as i16) });
    behaviors[BLOCK_FARMLAND.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { ItemId(BLOCK_DIRT.0 as i16) });
    behaviors[BLOCK_ORE_COAL.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::COAL });
    behaviors[BLOCK_ORE_DIAMOND.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::DIAMOND });
    behaviors[BLOCK_REDSTONE.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::REDSTONE });
    behaviors[BLOCK_SUGARCANE.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::SUGARCANE });
    behaviors[BLOCK_COBWEB.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::STRING });
    behaviors[BLOCK_DEADBUSH.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::INVALID });
    behaviors[BLOCK_STAIRS_WOOD.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { ItemId(BLOCK_PLANKS.0 as i16) });
    behaviors[BLOCK_STAIRS_COBBLESTONE.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { ItemId(BLOCK_COBBLESTONE.0 as i16) });

    behaviors[BLOCK_SIGN.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::SIGN });
    behaviors[BLOCK_SIGN_WALL.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::SIGN });
    behaviors[BLOCK_FURNACE_LIT.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { ItemId(BLOCK_FURNACE.0 as i16) });
    behaviors[BLOCK_REDSTONE_REPEATER_OFF.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::REDSTONE_REPEATER });
    behaviors[BLOCK_REDSTONE_REPEATER_ON.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::REDSTONE_REPEATER });
    behaviors[BLOCK_REDSTONE_TORCH_OFF.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { ItemId(BLOCK_REDSTONE_TORCH_ON.0 as i16) });

    // --------------- drop themselves but pass their metadata onto the item ---------------
    behaviors[BLOCK_WOOL.0 as usize].damage_dropped = Some(|meta: u8| -> ItemDamage { meta as ItemDamage });
    behaviors[BLOCK_LOG.0 as usize].damage_dropped = Some(|meta: u8| -> ItemDamage { meta as ItemDamage });
    behaviors[BLOCK_SAPLING.0 as usize].damage_dropped = Some(|meta: u8| -> ItemDamage { (meta & 3) as ItemDamage });

    // --------------- don't drop anything ---------------
    behaviors[BLOCK_ICE.0 as usize].quantity_dropped = Some(|_rng: &mut Random| -> ItemAmount { 0 });
    behaviors[BLOCK_GLASS.0 as usize].quantity_dropped = Some(|_rng: &mut Random| -> ItemAmount { 0 });
    behaviors[BLOCK_BOOKSHELF.0 as usize].quantity_dropped = Some(|_rng: &mut Random| -> ItemAmount { 0 });
    behaviors[BLOCK_CAKE.0 as usize].quantity_dropped = Some(|_rng: &mut Random| -> ItemAmount { 0 });
    behaviors[BLOCK_MOB_SPAWNER.0 as usize].quantity_dropped = Some(|_rng: &mut Random| -> ItemAmount { 0 });
    behaviors[BLOCK_FIRE.0 as usize].quantity_dropped = Some(|_rng: &mut Random| -> ItemAmount { 0 });
    behaviors[BLOCK_PISTON_HEAD.0 as usize].quantity_dropped = Some(|_rng: &mut Random| -> ItemAmount { 0 });
    behaviors[BLOCK_PISTON_MOVING.0 as usize].quantity_dropped = Some(|_rng: &mut Random| -> ItemAmount { 0 });
    behaviors[BLOCK_NETHER_PORTAL.0 as usize].quantity_dropped = Some(|_rng: &mut Random| -> ItemAmount { 0 });
    behaviors[BLOCK_SNOW_LAYER.0 as usize].quantity_dropped = Some(|_rng: &mut Random| -> ItemAmount { 0 });

    // --------------- drops influenced by RNG ---------------
    behaviors[BLOCK_GRAVEL.0 as usize].id_dropped = Some(|_meta: u8, rng: &mut Random| -> ItemId {
        if rng.next_int_bound(10) == 0 { items::FLINT } else { ItemId(BLOCK_GRAVEL.0 as i16) }
    });

    behaviors[BLOCK_ORE_LAPIS_LAZULI.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::DYE });
    behaviors[BLOCK_ORE_LAPIS_LAZULI.0 as usize].damage_dropped = Some(|_meta: u8| -> ItemDamage { 4 });
    behaviors[BLOCK_ORE_LAPIS_LAZULI.0 as usize].quantity_dropped =
        Some(|rng: &mut Random| -> ItemAmount { (4 + rng.next_int_bound(5)) as ItemAmount });

    behaviors[BLOCK_ORE_REDSTONE_OFF.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::REDSTONE });
    behaviors[BLOCK_ORE_REDSTONE_OFF.0 as usize].quantity_dropped =
        Some(|rng: &mut Random| -> ItemAmount { (4 + rng.next_int_bound(2)) as ItemAmount });
    behaviors[BLOCK_ORE_REDSTONE_ON.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::REDSTONE });
    behaviors[BLOCK_ORE_REDSTONE_ON.0 as usize].quantity_dropped =
        Some(|rng: &mut Random| -> ItemAmount { (4 + rng.next_int_bound(2)) as ItemAmount });

    behaviors[BLOCK_GLOWSTONE.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::GLOWSTONE_DUST });
    behaviors[BLOCK_GLOWSTONE.0 as usize].quantity_dropped =
        Some(|rng: &mut Random| -> ItemAmount { (2 + rng.next_int_bound(3)) as ItemAmount });

    behaviors[BLOCK_LEAVES.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { ItemId(BLOCK_SAPLING.0 as i16) });
    behaviors[BLOCK_LEAVES.0 as usize].damage_dropped = Some(|meta: u8| -> ItemDamage { (meta & 3) as ItemDamage });
    behaviors[BLOCK_LEAVES.0 as usize].quantity_dropped =
        Some(|rng: &mut Random| -> ItemAmount { if rng.next_int_bound(20) == 0 { 1 } else { 0 } });

    behaviors[BLOCK_TALLGRASS.0 as usize].id_dropped = Some(|_meta: u8, rng: &mut Random| -> ItemId {
        if rng.next_int_bound(8) == 0 { items::SEEDS_WHEAT } else { items::INVALID }
    });

    // --------------- only drop if it's the correct half of the block being broken ---------------
    // TODO: other half of the block should be removed automatically
    behaviors[BLOCK_DOOR_WOOD.0 as usize].id_dropped = Some(|meta: u8, _rng: &mut Random| -> ItemId {
        if (meta & 8) != 0 { items::INVALID } else { items::DOOR_WOOD }
    });

    behaviors[BLOCK_DOOR_IRON.0 as usize].id_dropped = Some(|meta: u8, _rng: &mut Random| -> ItemId {
        if (meta & 8) != 0 { items::INVALID } else { items::DOOR_IRON }
    });

    behaviors[BLOCK_BED.0 as usize].id_dropped = Some(|meta: u8, _rng: &mut Random| -> ItemId {
        if (meta & 8) != 0 { items::INVALID } else { items::BED }
    });

    behaviors[BLOCK_CLAY.0 as usize].id_dropped = Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::CLAY });
    behaviors[BLOCK_CLAY.0 as usize].quantity_dropped = Some(|_rng: &mut Random| -> ItemAmount { 4 });

    behaviors[BLOCK_SLAB.0 as usize].damage_dropped = Some(|meta: u8| -> ItemDamage { meta as ItemDamage });

    behaviors[BLOCK_DOUBLE_SLAB.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { ItemId(BLOCK_SLAB.0 as i16) });
    behaviors[BLOCK_DOUBLE_SLAB.0 as usize].damage_dropped = Some(|meta: u8| -> ItemDamage { meta as ItemDamage });
    behaviors[BLOCK_DOUBLE_SLAB.0 as usize].quantity_dropped = Some(|_rng: &mut Random| -> ItemAmount { 2 });

    behaviors[BLOCK_SNOW.0 as usize].id_dropped =
        Some(|_meta: u8, _rng: &mut Random| -> ItemId { items::SNOWBALL });
    behaviors[BLOCK_SNOW.0 as usize].quantity_dropped = Some(|_rng: &mut Random| -> ItemAmount { 4 });

    block_properties::set_tables(properties, behaviors);
}
