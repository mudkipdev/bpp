/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AABB {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl AABB {
    pub fn intersects(&self, other: &AABB) -> bool {
        other.max_x > self.min_x
            && other.min_x < self.max_x
            && other.max_y > self.min_y
            && other.min_y < self.max_y
            && other.max_z > self.min_z
            && other.min_z < self.max_z
    }

    pub fn offset(&self, dx: f64, dy: f64, dz: f64) -> AABB {
        AABB {
            min_x: self.min_x + dx,
            min_y: self.min_y + dy,
            min_z: self.min_z + dz,
            max_x: self.max_x + dx,
            max_y: self.max_y + dy,
            max_z: self.max_z + dz,
        }
    }

    pub fn expand(&self, dx: f64, dy: f64, dz: f64) -> AABB {
        AABB {
            min_x: self.min_x - dx,
            min_y: self.min_y - dy,
            min_z: self.min_z - dz,
            max_x: self.max_x + dx,
            max_y: self.max_y + dy,
            max_z: self.max_z + dz,
        }
    }

    pub fn add_coord(&self, dx: f64, dy: f64, dz: f64) -> AABB {
        let mut x0 = self.min_x;
        let mut y0 = self.min_y;
        let mut z0 = self.min_z;
        let mut x1 = self.max_x;
        let mut y1 = self.max_y;
        let mut z1 = self.max_z;
        if dx < 0.0 {
            x0 += dx;
        } else {
            x1 += dx;
        }
        if dy < 0.0 {
            y0 += dy;
        } else {
            y1 += dy;
        }
        if dz < 0.0 {
            z0 += dz;
        } else {
            z1 += dz;
        }
        AABB {
            min_x: x0,
            min_y: y0,
            min_z: z0,
            max_x: x1,
            max_y: y1,
            max_z: z1,
        }
    }

    // Checks YZ overlap first, then computes how far we can move in X
    pub fn calculate_x_offset(&self, other: &AABB, dx: f64) -> f64 {
        let mut dx = dx;
        if other.max_y > self.min_y && other.min_y < self.max_y {
            if other.max_z > self.min_z && other.min_z < self.max_z {
                if dx > 0.0 && other.max_x <= self.min_x {
                    let d = self.min_x - other.max_x;
                    if d < dx {
                        dx = d;
                    }
                }
                if dx < 0.0 && other.min_x >= self.max_x {
                    let d = self.max_x - other.min_x;
                    if d > dx {
                        dx = d;
                    }
                }
            }
        }
        dx
    }

    pub fn calculate_y_offset(&self, other: &AABB, dy: f64) -> f64 {
        let mut dy = dy;
        if other.max_x > self.min_x && other.min_x < self.max_x {
            if other.max_z > self.min_z && other.min_z < self.max_z {
                if dy > 0.0 && other.max_y <= self.min_y {
                    let d = self.min_y - other.max_y;
                    if d < dy {
                        dy = d;
                    }
                }
                if dy < 0.0 && other.min_y >= self.max_y {
                    let d = self.max_y - other.min_y;
                    if d > dy {
                        dy = d;
                    }
                }
            }
        }
        dy
    }

    pub fn calculate_z_offset(&self, other: &AABB, dz: f64) -> f64 {
        let mut dz = dz;
        if other.max_x > self.min_x && other.min_x < self.max_x {
            if other.max_y > self.min_y && other.min_y < self.max_y {
                if dz > 0.0 && other.max_z <= self.min_z {
                    let d = self.min_z - other.max_z;
                    if d < dz {
                        dz = d;
                    }
                }
                if dz < 0.0 && other.min_z >= self.max_z {
                    let d = self.max_z - other.min_z;
                    if d > dz {
                        dz = d;
                    }
                }
            }
        }
        dz
    }
}

// A collision shape is basically a container of AABB's, used for blocks like stairs and to generate the swept volume of an entity's movement.
#[derive(Clone, Debug, Default)]
pub struct CollisionShape {
    pub boxes: Vec<AABB>,
}

impl CollisionShape {
    pub fn add(&mut self, aabb: AABB) {
        self.boxes.push(aabb);
    }

    pub fn offset(&self, dx: f64, dy: f64, dz: f64) -> CollisionShape {
        let mut result = CollisionShape {
            boxes: Vec::with_capacity(self.boxes.len()),
        };
        for aabb in &self.boxes {
            result.boxes.push(aabb.offset(dx, dy, dz));
        }
        result
    }

    pub fn intersects(&self, other: &CollisionShape) -> bool {
        for box1 in &self.boxes {
            for box2 in &other.boxes {
                if box1.intersects(box2) {
                    return true;
                }
            }
        }
        false
    }

    pub fn expand(&self, dx: f64, dy: f64, dz: f64) -> CollisionShape {
        let mut result = CollisionShape {
            boxes: Vec::with_capacity(self.boxes.len()),
        };
        for aabb in &self.boxes {
            result.boxes.push(aabb.expand(dx, dy, dz));
        }
        result
    }

    pub fn calculate_x_offset(&self, entity: &AABB, dx: f64) -> f64 {
        let mut dx = dx;
        for aabb in &self.boxes {
            dx = aabb.calculate_x_offset(entity, dx);
        }
        dx
    }

    pub fn calculate_y_offset(&self, entity: &AABB, dy: f64) -> f64 {
        let mut dy = dy;
        for aabb in &self.boxes {
            dy = aabb.calculate_y_offset(entity, dy);
        }
        dy
    }

    pub fn calculate_z_offset(&self, entity: &AABB, dz: f64) -> f64 {
        let mut dz = dz;
        for aabb in &self.boxes {
            dz = aabb.calculate_z_offset(entity, dz);
        }
        dz
    }

    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }
}
