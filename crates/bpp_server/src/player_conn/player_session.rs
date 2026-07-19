/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use std::sync::{Arc, Mutex, Weak};
use std::time::Instant;

use bpp_shared::base_types::{ItemId, NbtSlotId, NetworkSlotId, TickTime, TransactionId, WindowId};
use bpp_shared::enums::blocks::{BLOCK_AIR, BlockType};
use bpp_shared::enums::items;
use bpp_shared::inventory::inventories::InventoryPlayer;
use bpp_shared::inventory::inventory::InventoryBehavior;
use bpp_shared::inventory::inventory_interaction::{InventoryInteraction, InventoryInteractionBehavior};
use bpp_shared::inventory::interactions::chest::ChestInventoryInteraction;
use bpp_shared::inventory::interactions::crafting_table::CraftingTableInventoryInteraction;
use bpp_shared::inventory::interactions::large_chest::LargeChestInventoryInteraction;
use bpp_shared::inventory::interactions::player::PlayerInventoryInteraction;
use bpp_shared::inventory::item_stack::ItemStack;
use bpp_shared::nbt::nbt::{TAG_COMPOUND, TAG_DOUBLE, TAG_FLOAT, Tag};
use bpp_shared::networking::network_stream::NetworkStream;
use bpp_shared::numeric_structs::{FLOAT2_ZERO, Float2, Int3, Int32_2};
use bpp_shared::runtime::Runtime;
use bpp_shared::tile_entities::tile_entity::TileEntityBehavior;
use bpp_shared::world::client_pos::ClientPosition;
use bpp_shared::world::world::{PendingBlock, WorldManager};

use crate::entities::entity_mp_player::EntityMPPlayer;
use crate::entities::entity_tracker::EntityTracker;

#[derive(Clone, Default)]
pub enum ActiveContainer {
    #[default]
    None,
    Chest(Weak<Mutex<dyn TileEntityBehavior + Send>>),
    LargeChest(Weak<Mutex<dyn TileEntityBehavior + Send>>, Weak<Mutex<dyn TileEntityBehavior + Send>>),
    CraftingTable(Int3),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ConnectionState {
    Handshaking,
    LoggingIn,
    WaitingForSpawnChunks,
    Playing,
}

pub struct PlayerSession {
    pub stream: NetworkStream,
    pub position: ClientPosition,

    // Our player entity
    pub entity: Option<Arc<Mutex<EntityMPPlayer>>>,
    pub entity_tracker: Weak<Mutex<EntityTracker>>,
    pub entity_registered: bool,

    // rotation.x = yaw, rotation.y = pitch
    pub rotation: Float2,

    pub sent_chunks: HashSet<Int32_2>,
    pub flushed_chunks: HashSet<Int32_2>, // Actually written to stream

    // Chunks that were written to the stream during the last flush() call.
    pub newly_flushed: Vec<Int32_2>,

    // Chunks that were unloaded during the last enqueue() call.
    pub newly_unloaded: Vec<Int32_2>,

    // Block updates that arrived while the chunk was enqueued but not yet flushed.
    pub pending_block_changes: HashMap<Int32_2, Vec<PendingBlock>>,

    pub conn_state: ConnectionState,
    pub username: String,
    pub last_packet_time: Instant,

    // Inventory
    pub inventory: InventoryPlayer,
    pub inventory_interaction_state: InventoryInteraction,
    pub active_container: ActiveContainer,
    pub active_interaction_state: InventoryInteraction,

    // windowId = 0 is always the player inventory. Non-zero means a container is open.
    // ranges from 0-127 and wraps
    pub open_window_id: WindowId,

    // Lock after a rejected click until client acknowledges the resync.
    // While locked, all incoming clicks are rejected to prevent state corruption.
    pub inventory_locked: bool,
    pub pending_transaction_id: TransactionId,
    pub pending_window_id: WindowId,

    pub dimension: i8, // 0 = overworld, -1 = nether

    pub last_targeted_block: BlockType,
    pub started_mining_at_tick: TickTime,
}

impl PlayerSession {
    pub fn get_next_window_id(&mut self) -> WindowId {
        self.open_window_id = WindowId((((self.open_window_id.value() as i32) + 1) % 128) as i8);
        self.open_window_id
    }

    pub fn has_active_container(&self) -> bool {
        !matches!(self.active_container, ActiveContainer::None)
    }

    pub fn with_own_interaction<R>(&mut self, runtime: &Runtime, f: impl FnOnce(&mut PlayerInventoryInteraction) -> R) -> R {
        let mut interaction = PlayerInventoryInteraction::new(&mut self.inventory, runtime);
        *interaction.base_mut() = std::mem::take(&mut self.inventory_interaction_state);
        let result = f(&mut interaction);
        self.inventory_interaction_state = std::mem::take(interaction.base_mut());
        result
    }

    pub fn with_active_interaction<R>(
        &mut self,
        runtime: &Runtime,
        world: &mut WorldManager,
        f: impl FnOnce(&mut dyn InventoryInteractionBehavior) -> R,
    ) -> Option<R> {
        let container = self.active_container.clone();
        let state = std::mem::take(&mut self.active_interaction_state);
        let (result, new_state) = match container {
            ActiveContainer::None => {
                self.active_interaction_state = state;
                return None;
            }
            ActiveContainer::Chest(handle) => {
                let chest = handle.upgrade()?;
                let mut interaction = ChestInventoryInteraction::new(&mut self.inventory, chest);
                *interaction.base_mut() = state;
                let r = f(&mut interaction);
                (r, std::mem::take(interaction.base_mut()))
            }
            ActiveContainer::LargeChest(upper, lower) => {
                let upper = upper.upgrade()?;
                let lower = lower.upgrade()?;
                let mut interaction = LargeChestInventoryInteraction::new(&mut self.inventory, upper, lower);
                *interaction.base_mut() = state;
                let r = f(&mut interaction);
                (r, std::mem::take(interaction.base_mut()))
            }
            ActiveContainer::CraftingTable(pos) => {
                let mut interaction = CraftingTableInventoryInteraction::new(&mut self.inventory, world, runtime, pos);
                *interaction.base_mut() = state;
                let r = f(&mut interaction);
                (r, std::mem::take(interaction.base_mut()))
            }
        };
        self.active_interaction_state = new_state;
        Some(result)
    }

    pub fn new(socket: TcpStream) -> Self {
        let inventory = InventoryPlayer::new();
        PlayerSession {
            stream: NetworkStream::new(socket),
            position: ClientPosition::default(),

            entity: None,
            entity_tracker: Weak::new(),
            entity_registered: false,

            rotation: FLOAT2_ZERO,

            sent_chunks: HashSet::new(),
            flushed_chunks: HashSet::new(),

            newly_flushed: Vec::new(),
            newly_unloaded: Vec::new(),

            pending_block_changes: HashMap::new(),

            conn_state: ConnectionState::Handshaking,
            username: String::new(),
            last_packet_time: Instant::now(),

            inventory,
            inventory_interaction_state: InventoryInteraction::new(),
            active_container: ActiveContainer::None,
            active_interaction_state: InventoryInteraction::new(),

            open_window_id: WindowId(0),

            inventory_locked: false,
            pending_transaction_id: TransactionId(0),
            pending_window_id: WindowId(0),

            dimension: 0,

            last_targeted_block: BLOCK_AIR,
            started_mining_at_tick: 0,
        }
    }

    // Load our player data from file
    pub fn load_player_nbt(&mut self, nbt: &Tag) {
        // Very basic but just stuff we care about for now
        let it = nbt.get("Pos").get_list();
        self.position.pos.x = it[0].get_double();
        self.position.pos.y = it[1].get_double();
        self.position.pos.z = it[2].get_double();

        let it2 = nbt.get("Rotation").get_list();
        self.rotation.x = it2[0].get_float();
        self.rotation.y = it2[1].get_float();

        self.dimension = nbt.get("Dimension").get_int() as i8;

        let it3 = nbt.get("Inventory").get_list();
        for item in it3 {
            let nbt_slot = NbtSlotId(item.get("Slot").get_byte());
            let network_slot = self.inventory.get_network_slot_id(nbt_slot);
            if network_slot.value() < 0 || network_slot.value() as usize >= self.inventory.base.slots.len() {
                continue;
            }
            self.inventory.base.slots[network_slot.value() as usize] = ItemStack {
                id: ItemId(item.get("id").get_short()),
                count: item.get("Count").get_byte(),
                data: item.get("Damage").get_short(),
            };
        }
    }

    pub fn serialize_to_nbt(&mut self) -> Tag {
        let motion = vec![
            Tag::Double { name: String::new(), double_value: 0.0 },
            Tag::Double { name: String::new(), double_value: 0.0 },
            Tag::Double { name: String::new(), double_value: 0.0 },
        ];

        // Save position and rotation
        let pos = vec![
            Tag::Double { name: String::new(), double_value: self.position.pos.x },
            Tag::Double { name: String::new(), double_value: self.position.pos.y },
            Tag::Double { name: String::new(), double_value: self.position.pos.z },
        ];

        let rotation = vec![
            Tag::Float { name: String::new(), float_value: self.rotation.x },
            Tag::Float { name: String::new(), float_value: self.rotation.y },
        ];

        // Save our current inventory
        let mut inventory_list = Vec::new();
        let mut slot_id: i32 = 0;
        for item in &self.inventory.base.slots {
            if item.id != items::INVALID {
                let mut item_compound = HashMap::new();
                item_compound.insert(
                    "Slot".to_string(),
                    Tag::Byte {
                        name: "Slot".to_string(),
                        byte_value: self.inventory.get_nbt_slot_id(NetworkSlotId(slot_id as i16)).value(),
                    },
                );
                item_compound.insert(
                    "id".to_string(),
                    Tag::Short { name: "id".to_string(), short_value: item.id.value() },
                );
                item_compound.insert(
                    "Count".to_string(),
                    Tag::Byte { name: "Count".to_string(), byte_value: item.count },
                );
                item_compound.insert(
                    "Damage".to_string(),
                    Tag::Short { name: "Damage".to_string(), short_value: item.data },
                );
                inventory_list.push(Tag::Compound { name: String::new(), compound: item_compound });
            }
            slot_id += 1;
        }

        let mut root = HashMap::new();
        root.insert(
            "Motion".to_string(),
            Tag::List { name: "Motion".to_string(), list_type: TAG_DOUBLE, list: motion },
        );
        root.insert(
            "SleepTimer".to_string(),
            Tag::Short { name: "SleepTimer".to_string(), short_value: 0 },
        );
        root.insert("Health".to_string(), Tag::Short { name: "Health".to_string(), short_value: 20 });
        root.insert("Air".to_string(), Tag::Short { name: "Air".to_string(), short_value: 300 });
        root.insert("OnGround".to_string(), Tag::Byte { name: "OnGround".to_string(), byte_value: 0 });
        root.insert(
            "Dimension".to_string(),
            Tag::Int { name: "Dimension".to_string(), int_value: self.dimension as i32 },
        );
        root.insert(
            "Rotation".to_string(),
            Tag::List { name: "Rotation".to_string(), list_type: TAG_FLOAT, list: rotation },
        );
        root.insert(
            "FallDistance".to_string(),
            Tag::Float { name: "FallDistance".to_string(), float_value: 0.0 },
        );
        root.insert("Sleeping".to_string(), Tag::Byte { name: "Sleeping".to_string(), byte_value: 0 });
        root.insert("Pos".to_string(), Tag::List { name: "Pos".to_string(), list_type: TAG_DOUBLE, list: pos });
        root.insert("DeathTime".to_string(), Tag::Short { name: "DeathTime".to_string(), short_value: 0 });
        root.insert("Fire".to_string(), Tag::Short { name: "Fire".to_string(), short_value: -20 });
        root.insert("HurtTime".to_string(), Tag::Short { name: "HurtTime".to_string(), short_value: 0 });
        root.insert(
            "AttackTime".to_string(),
            Tag::Short { name: "AttackTime".to_string(), short_value: 0 },
        );
        root.insert(
            "Inventory".to_string(),
            Tag::List { name: "Inventory".to_string(), list_type: TAG_COMPOUND, list: inventory_list },
        );

        Tag::Compound { name: String::new(), compound: root }
    }
}

impl Drop for PlayerSession {
    fn drop(&mut self) {
        // So our player entity despawns from the world
        if let Some(entity) = &self.entity {
            let mut guard = entity.lock().unwrap();
            guard.base.base.is_dead = true;
            guard.session = None;
        }
        self.entity_tracker = Weak::new();
    }
}
