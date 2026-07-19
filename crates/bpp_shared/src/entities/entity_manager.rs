/*
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use crate::base_types::EntityId;
use crate::entities::entity::EntityBehavior;
use crate::entities::entity_item::ItemEntity;
use crate::enums::entities::EntityType;
use crate::helpers::aabb::AABB;
use crate::helpers::java::java_math::MathHelper;
use crate::logger::logger::global_logger;
use crate::nbt::nbt::Tag;
use crate::numeric_structs::{Int2, Int3, Vec3};
use crate::world::world::WorldManager;

pub struct EntityBucket {
    // 16 blocks tall
    pub entities: Vec<Weak<Mutex<dyn EntityBehavior + Send>>>,
}

impl Default for EntityBucket {
    fn default() -> Self {
        Self { entities: Vec::new() }
    }
}

pub struct EntityContainer {
    pub bucket_pos: Int2,
    pub buckets: [EntityBucket; 10], // 0 = lowest bucket (below the world), 1 = Y lvl 0; 8 = y lvl 127, 9 = above the world
}

impl Default for EntityContainer {
    fn default() -> Self {
        Self { bucket_pos: Int2::new(0, 0), buckets: Default::default() }
    }
}

// For ticking all entities and keeping track of them in the world
pub struct EntityManager {
    pub next_entity_id: EntityId, // Minecraft seems to reserve 0 and 1
    pub entities: Vec<Arc<Mutex<dyn EntityBehavior + Send>>>,
    pub entity_containers: HashMap<Int2, EntityContainer>,
    pub world: Weak<Mutex<WorldManager>>, // we need to bind a pointer to this later

    // Callbacks that we can link into
    pub on_entity_spawn: Option<Box<dyn FnMut(Arc<Mutex<dyn EntityBehavior + Send>>) + Send>>,
    pub on_entity_despawn: Option<Box<dyn FnMut(Arc<Mutex<dyn EntityBehavior + Send>>) + Send>>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            next_entity_id: EntityId(2),
            entities: Vec::new(),
            entity_containers: HashMap::new(),
            world: Weak::new(),
            on_entity_spawn: None,
            on_entity_despawn: None,
        }
    }

    pub fn compute_bucket_pos(pos_x: f64, pos_y: f64, pos_z: f64) -> Int3 {
        let mut pos = Int3::new(
            MathHelper::floor_double(pos_x / 16.0),
            MathHelper::floor_double(pos_z / 16.0),
            MathHelper::floor_double(pos_y / 16.0),
        );

        // Entity collisions below and above the world are just gonna be inefficient
        pos.z = pos.z.max(0);
        pos.z = pos.z.min(9);
        pos
    }

    pub fn get_entities_within_aabb_excluding(
        &self,
        area: AABB,
        entity_id: EntityId,
    ) -> Vec<Arc<Mutex<dyn EntityBehavior + Send>>> {
        // Get all entities within an AABB excluding this entity id
        let mut entities = self.get_entities_within_aabb(area);
        entities.retain(|entity| entity.lock().unwrap().base().id != entity_id);
        entities
    }

    pub fn get_entities_within_aabb(&self, area: AABB) -> Vec<Arc<Mutex<dyn EntityBehavior + Send>>> {
        // Get all entities within an AABB
        let mut colliding_entities = Vec::new();

        // Normalize to block coordinates
        let block_min_x = MathHelper::floor_double((area.min_x - 2.0) / 16.0);
        let block_min_z = MathHelper::floor_double((area.min_z - 2.0) / 16.0);
        let block_max_x = MathHelper::floor_double((area.max_x + 2.0) / 16.0);
        let block_max_z = MathHelper::floor_double((area.max_z + 2.0) / 16.0);

        // Get our start and end bucket
        let mut bucket_min_y = MathHelper::floor_double((area.min_y - 2.0) / 16.0);
        let mut bucket_max_y = MathHelper::floor_double((area.max_y + 2.0) / 16.0);
        bucket_min_y = bucket_min_y.max(0).min(9);
        bucket_max_y = bucket_max_y.max(0).min(9);

        // Go through each block position
        for x in block_min_x..=block_max_x {
            for z in block_min_z..=block_max_z {
                let container = match self.entity_containers.get(&Int2::new(x, z)) {
                    Some(container) => container,
                    None => continue,
                };
                for by in bucket_min_y..=bucket_max_y {
                    // Get every entity within every bucket
                    for entity_weak in container.buckets[by as usize].entities.iter() {
                        if let Some(entity) = entity_weak.upgrade() {
                            colliding_entities.push(entity);
                        }
                    }
                }
            }
        }
        colliding_entities
    }

    pub fn tick(&mut self) {
        // Make a copy so we aren't modifying the vector while iterating over it
        let copy = self.entities.clone();

        // Tick EVERY entity
        for entity in copy {
            // Remove dead entities from the system
            let is_dead = entity.lock().unwrap().base().is_dead;
            if is_dead {
                entity.lock().unwrap().base_mut().world = Weak::new();
                self.entities.retain(|e| !Arc::ptr_eq(e, &entity));
                let bucket_pos = entity.lock().unwrap().base().bucket_pos;
                if let Some(container) = self.entity_containers.get_mut(&Int2::new(bucket_pos.x, bucket_pos.y)) {
                    let bucket = &mut container.buckets[bucket_pos.z as usize];
                    bucket.entities.retain(|weak| match weak.upgrade() {
                        Some(locked) => !Arc::ptr_eq(&locked, &entity), // Remove if expired or matches our entity
                        None => false,
                    });
                }

                if let Some(callback) = self.on_entity_despawn.as_mut() {
                    callback(entity);
                }
                continue;
            }
            entity.lock().unwrap().tick();

            // Check to see if this entity went into another container or bucket
            let (pos_x, pos_y, pos_z, old_bucket_pos) = {
                let guard = entity.lock().unwrap();
                let base = guard.base();
                (base.pos_x, base.pos_y, base.pos_z, base.bucket_pos)
            };
            let new_bucket_pos = Self::compute_bucket_pos(pos_x, pos_y, pos_z);

            if new_bucket_pos != old_bucket_pos {
                // Remove from the old bucket
                if let Some(container) = self.entity_containers.get_mut(&Int2::new(old_bucket_pos.x, old_bucket_pos.y)) {
                    let bucket = &mut container.buckets[old_bucket_pos.z as usize];
                    bucket.entities.retain(|weak| match weak.upgrade() {
                        Some(locked) => !Arc::ptr_eq(&locked, &entity), // Remove if expired or matches our entity
                        None => false,
                    });
                }

                // Put in the new bucket
                let new_container = self.entity_containers.entry(Int2::new(new_bucket_pos.x, new_bucket_pos.y)).or_default();
                new_container.buckets[new_bucket_pos.z as usize].entities.push(Arc::downgrade(&entity));
                entity.lock().unwrap().base_mut().bucket_pos = new_bucket_pos;
            }
        }
    }

    pub fn add_entity(&mut self, entity: Arc<Mutex<dyn EntityBehavior + Send>>, force_entity_id: EntityId) {
        let world = match self.world.upgrade() {
            Some(world) => world,
            None => {
                global_logger().error("Attempted to add an entity before EntityManager was bound to a world!\n");
                return;
            }
        };

        // Assign an ID if we weren't forced to use one
        let id = if force_entity_id == EntityId(-1) { self.get_next_entity_id() } else { force_entity_id };

        let bucket_pos = {
            let mut guard = entity.lock().unwrap();
            let base = guard.base_mut();
            base.id = id;
            base.world = self.world.clone(); // Bind the world pointer so the entity can interact with the world
            base.dim = world.lock().unwrap().this_dimension;

            // Register the entity into its initial bucket
            base.bucket_pos = Self::compute_bucket_pos(base.pos_x, base.pos_y, base.pos_z);
            base.bucket_pos
        };

        let container = self.entity_containers.entry(Int2::new(bucket_pos.x, bucket_pos.y)).or_default();
        container.buckets[bucket_pos.z as usize].entities.push(Arc::downgrade(&entity));

        self.entities.push(Arc::clone(&entity));
        if let Some(callback) = self.on_entity_spawn.as_mut() {
            callback(entity);
        }
    }

    pub fn remove_entity(&mut self, id: EntityId) {
        // Find the entity for this ID
        let entity = match self.entities.iter().find(|e| e.lock().unwrap().base().id == id) {
            Some(entity) => Arc::clone(entity),
            None => return, // Not found, nothing to do
        };

        // Remove from its bucket
        let bucket_pos = entity.lock().unwrap().base().bucket_pos;
        if let Some(container) = self.entity_containers.get_mut(&Int2::new(bucket_pos.x, bucket_pos.y)) {
            let bucket = &mut container.buckets[bucket_pos.z as usize];
            bucket.entities.retain(|weak| match weak.upgrade() {
                Some(locked) => !Arc::ptr_eq(&locked, &entity),
                None => false,
            });
        }

        // Remove from the master list
        self.entities.retain(|e| !Arc::ptr_eq(e, &entity));

        // Set as dead for cleanup
        entity.lock().unwrap().base_mut().is_dead = true;
        if let Some(callback) = self.on_entity_despawn.as_mut() {
            callback(entity);
        }
    }

    pub fn chunk_has_entities(&self, cpos: Int2) -> bool {
        let container = match self.entity_containers.get(&cpos) {
            Some(container) => container,
            None => return false,
        };
        container.buckets.iter().any(|bucket| !bucket.entities.is_empty())
    }

    pub fn collect_entities_for_save(&mut self, cpos: Int2, clear_collected_entities: bool) -> Vec<Tag> {
        // Collect entities for this chunk coordinates (basically our entity container)
        // We then serialize these entities and return the vector of nbt tags
        // We mark the entities as dead for cleanup afterwards
        let mut collected_entities = Vec::new();

        let container = self.entity_containers.entry(cpos).or_default();

        for bucket in container.buckets.iter() {
            for entity_weak in bucket.entities.iter() {
                // Is this entity dead but not collected?
                if let Some(entity_shared) = entity_weak.upgrade() {
                    let mut guard = entity_shared.lock().unwrap();
                    if guard.base().is_dead {
                        continue; // We are dead so no save
                    }
                    if guard.base().r#type == EntityType::Player {
                        continue; // players cannot be saved
                    }
                    if clear_collected_entities {
                        guard.base_mut().is_dead = true; // Mark the entity as dead for cleanup
                    }
                    let compound = guard.serialize_to_nbt();
                    drop(guard);
                    match compound {
                        Some(compound) => collected_entities.push(compound),
                        None => continue, // If something went wrong abort save
                    }
                }
            }
        }
        collected_entities
    }

    pub fn create_entity_from_nbt(&mut self, nbt: &mut Tag) {
        // Load an entity from the nbt list
        let id = nbt.get("id").get_string().to_string();

        // TODO: load other entity types
        if id == "Item" {
            let mut item = ItemEntity::new(Vec3::new(0.0, 0.0, 0.0));
            item.load_from_nbt(nbt);
            self.add_entity(Arc::new(Mutex::new(item)), EntityId(-1));
        }
    }

    pub fn get_next_entity_id(&mut self) -> EntityId {
        let id = self.next_entity_id;
        self.next_entity_id = EntityId(self.next_entity_id.0 + 1);
        id
    }

    pub fn get_entity_nbt_id(entity_type: EntityType) -> Option<&'static str> {
        match entity_type {
            EntityType::Item => Some("Item"),
            EntityType::Boat => Some("Boat"),
            EntityType::LitTnt => Some("PrimedTnt"),
            EntityType::Arrow => Some("Arrow"),
            EntityType::ThrownSnowball => Some("Snowball"),
            EntityType::Painting => Some("Painting"),
            EntityType::Creeper => Some("Creeper"),
            EntityType::Skeleton => Some("Skeleton"),
            EntityType::Spider => Some("Spider"),
            EntityType::GiantZombie => Some("Giant"),
            EntityType::Zombie => Some("Zombie"),
            EntityType::Slime => Some("Slime"),
            EntityType::Ghast => Some("Ghast"),
            EntityType::ZombiePigman => Some("PigZombie"),
            EntityType::Pig => Some("Pig"),
            EntityType::Sheep => Some("Sheep"),
            EntityType::Cow => Some("Cow"),
            EntityType::Chicken => Some("Chicken"),
            EntityType::Squid => Some("Squid"),
            EntityType::Wolf => Some("Wolf"),

            // Vanilla only has ONE minecart entity/string
            // There is a type field in the nbt itself
            EntityType::Minecart | EntityType::StorageMinecart | EntityType::FurnaceMinecart => Some("Minecart"),

            // Same deal as minecarts
            // not a separate string.
            EntityType::FallingSand | EntityType::FallingGravel => Some("FallingSand"),

            // These have no mapping!
            EntityType::None
            | EntityType::Player // Note: Players are saved differently (thanks notch)
            | EntityType::Fish
            | EntityType::Fireball
            | EntityType::ThrownEgg
            | EntityType::FishingBobber => None,
        }
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}
