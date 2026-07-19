/*
 * Copyright (c) 2026, mudkipdev <github.com/mudkipdev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use crate::base_structs::Block;
use crate::enums::blocks::{
    BLOCK_BED, BLOCK_BUTTON_STONE, BLOCK_DISPENSER, BLOCK_DOOR_IRON, BLOCK_DOOR_WOOD, BLOCK_DOUBLE_SLAB,
    BLOCK_FURNACE, BLOCK_FURNACE_LIT, BLOCK_LADDER, BLOCK_LAVA_FLOWING, BLOCK_LAVA_STILL, BLOCK_LEAVES, BLOCK_LEVER,
    BLOCK_LOG, BLOCK_PISTON, BLOCK_PISTON_HEAD, BLOCK_PISTON_STICKY, BLOCK_PUMPKIN, BLOCK_PUMPKIN_LIT, BLOCK_RAIL,
    BLOCK_RAIL_DETECTOR, BLOCK_RAIL_POWERED, BLOCK_REDSTONE_REPEATER_OFF, BLOCK_REDSTONE_REPEATER_ON,
    BLOCK_REDSTONE_TORCH_OFF, BLOCK_REDSTONE_TORCH_ON, BLOCK_SAPLING, BLOCK_SIGN, BLOCK_SIGN_WALL, BLOCK_SLAB,
    BLOCK_SNOW_LAYER, BLOCK_STAIRS_COBBLESTONE, BLOCK_STAIRS_WOOD, BLOCK_TALLGRASS, BLOCK_TORCH, BLOCK_TRAPDOOR,
    BLOCK_WATER_FLOWING, BLOCK_WATER_STILL, BLOCK_WOOL, BlockType,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WoodType {
    Oak = 0,
    Spruce = 1,
    Birch = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WoolColor {
    White = 0,
    Orange = 1,
    Magenta = 2,
    LightBlue = 3,
    Yellow = 4,
    Lime = 5,
    Pink = 6,
    Gray = 7,
    LightGray = 8,
    Cyan = 9,
    Purple = 10,
    Blue = 11,
    Brown = 12,
    Green = 13,
    Red = 14,
    Black = 15,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SlabType {
    Stone = 0,
    Sandstone = 1,
    Wood = 2,
    Cobblestone = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TallGrassType {
    DeadBush = 0,
    TallGrass = 1,
    Fern = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TorchAttachment {
    WestWall = 1,
    EastWall = 2,
    NorthWall = 3,
    SouthWall = 4,
    Floor = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WallFacing {
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LeverMount {
    WestWall = 1,
    EastWall = 2,
    NorthWall = 3,
    SouthWall = 4,
    FloorEastWest = 5,
    FloorNorthSouth = 6,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ButtonMount {
    WestWall = 1,
    EastWall = 2,
    NorthWall = 3,
    SouthWall = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RailShape {
    FlatNorthSouth = 0,
    FlatEastWest = 1,
    AscendingEast = 2,
    AscendingWest = 3,
    AscendingNorth = 4,
    AscendingSouth = 5,
    CurveNorthEast = 6,
    CurveSouthEast = 7,
    CurveSouthWest = 8,
    CurveNorthWest = 9,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PistonFacing {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum StairsDirection {
    East = 0,
    West = 1,
    South = 2,
    North = 3,
}

#[derive(Clone, Copy, Debug)]
pub struct StairsBuilder {
    id: BlockType,
    direction: StairsDirection,
}

impl StairsBuilder {
    pub const fn new(id: BlockType) -> Self {
        Self { id, direction: StairsDirection::East }
    }

    pub fn facing(self, direction: StairsDirection) -> Self {
        let mut b = self;
        b.direction = direction;
        b
    }

    pub fn as_block(&self) -> Block {
        Block { r#type: self.id, data: self.direction as u8 }
    }
}

impl From<StairsBuilder> for Block {
    fn from(b: StairsBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FluidBuilder {
    id: BlockType,
    level: u8,
    falling: bool,
}

impl FluidBuilder {
    pub const fn new(id: BlockType) -> Self {
        Self { id, level: 0, falling: false }
    }

    pub fn level(self, level: u8) -> Self {
        let mut b = self;
        b.level = level.min(7);
        b
    }

    pub fn falling(self, falling: bool) -> Self {
        let mut b = self;
        b.falling = falling;
        b
    }

    pub fn as_block(&self) -> Block {
        let mut data = self.level & 0x7;
        if self.falling {
            data |= 0x8;
        }
        Block { r#type: self.id, data }
    }
}

impl From<FluidBuilder> for Block {
    fn from(b: FluidBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SaplingBuilder {
    id: BlockType,
    r#type: WoodType,
    ready_to_grow: bool,
}

impl SaplingBuilder {
    pub const fn new() -> Self {
        Self { id: BLOCK_SAPLING, r#type: WoodType::Oak, ready_to_grow: false }
    }

    pub fn r#type(self, r#type: WoodType) -> Self {
        let mut b = self;
        b.r#type = r#type;
        b
    }

    pub fn ready_to_grow(self, ready_to_grow: bool) -> Self {
        let mut b = self;
        b.ready_to_grow = ready_to_grow;
        b
    }

    pub fn as_block(&self) -> Block {
        let mut data = self.r#type as u8;
        if self.ready_to_grow {
            data |= 0x8;
        }
        Block { r#type: self.id, data }
    }
}

impl From<SaplingBuilder> for Block {
    fn from(b: SaplingBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LogBuilder {
    id: BlockType,
    r#type: WoodType,
}

impl LogBuilder {
    pub const fn new() -> Self {
        Self { id: BLOCK_LOG, r#type: WoodType::Oak }
    }

    pub fn r#type(self, r#type: WoodType) -> Self {
        let mut b = self;
        b.r#type = r#type;
        b
    }

    pub fn as_block(&self) -> Block {
        Block { r#type: self.id, data: self.r#type as u8 }
    }
}

impl From<LogBuilder> for Block {
    fn from(b: LogBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LeavesBuilder {
    id: BlockType,
    r#type: WoodType,
    decaying: bool,
}

impl LeavesBuilder {
    pub const fn new() -> Self {
        Self { id: BLOCK_LEAVES, r#type: WoodType::Oak, decaying: false }
    }

    pub fn r#type(self, r#type: WoodType) -> Self {
        let mut b = self;
        b.r#type = r#type;
        b
    }

    pub fn decaying(self, decaying: bool) -> Self {
        let mut b = self;
        b.decaying = decaying;
        b
    }

    pub fn as_block(&self) -> Block {
        let mut data = self.r#type as u8;
        if self.decaying {
            data |= 0x8;
        }
        Block { r#type: self.id, data }
    }
}

impl From<LeavesBuilder> for Block {
    fn from(b: LeavesBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TallGrassBuilder {
    id: BlockType,
    r#type: TallGrassType,
}

impl TallGrassBuilder {
    pub const fn new() -> Self {
        Self { id: BLOCK_TALLGRASS, r#type: TallGrassType::TallGrass }
    }

    pub fn r#type(self, r#type: TallGrassType) -> Self {
        let mut b = self;
        b.r#type = r#type;
        b
    }

    pub fn as_block(&self) -> Block {
        Block { r#type: self.id, data: self.r#type as u8 }
    }
}

impl From<TallGrassBuilder> for Block {
    fn from(b: TallGrassBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WoolBuilder {
    id: BlockType,
    color: WoolColor,
}

impl WoolBuilder {
    pub const fn new() -> Self {
        Self { id: BLOCK_WOOL, color: WoolColor::White }
    }

    pub fn color(self, color: WoolColor) -> Self {
        let mut b = self;
        b.color = color;
        b
    }

    pub fn as_block(&self) -> Block {
        Block { r#type: self.id, data: self.color as u8 }
    }
}

impl From<WoolBuilder> for Block {
    fn from(b: WoolBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SlabBuilder {
    id: BlockType,
    r#type: SlabType,
}

impl SlabBuilder {
    pub const fn new(id: BlockType) -> Self {
        Self { id, r#type: SlabType::Stone }
    }

    pub fn r#type(self, r#type: SlabType) -> Self {
        let mut b = self;
        b.r#type = r#type;
        b
    }

    pub fn as_block(&self) -> Block {
        Block { r#type: self.id, data: self.r#type as u8 }
    }
}

impl From<SlabBuilder> for Block {
    fn from(b: SlabBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TorchBuilder {
    id: BlockType,
    attachment: TorchAttachment,
}

impl TorchBuilder {
    pub const fn new(id: BlockType) -> Self {
        Self { id, attachment: TorchAttachment::Floor }
    }

    pub fn attachment(self, attachment: TorchAttachment) -> Self {
        let mut b = self;
        b.attachment = attachment;
        b
    }

    pub fn as_block(&self) -> Block {
        Block { r#type: self.id, data: self.attachment as u8 }
    }
}

impl From<TorchBuilder> for Block {
    fn from(b: TorchBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WallFacingBuilder {
    id: BlockType,
    facing: WallFacing,
}

impl WallFacingBuilder {
    pub const fn new(id: BlockType) -> Self {
        Self { id, facing: WallFacing::North }
    }

    pub fn facing(self, facing: WallFacing) -> Self {
        let mut b = self;
        b.facing = facing;
        b
    }

    pub fn as_block(&self) -> Block {
        Block { r#type: self.id, data: self.facing as u8 }
    }
}

impl From<WallFacingBuilder> for Block {
    fn from(b: WallFacingBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BedBuilder {
    id: BlockType,
    direction: Direction,
    occupied: bool,
    head: bool,
}

impl BedBuilder {
    pub const fn new() -> Self {
        Self { id: BLOCK_BED, direction: Direction::South, occupied: false, head: false }
    }

    pub fn facing(self, direction: Direction) -> Self {
        let mut b = self;
        b.direction = direction;
        b
    }

    pub fn occupied(self, occupied: bool) -> Self {
        let mut b = self;
        b.occupied = occupied;
        b
    }

    pub fn head(self, head: bool) -> Self {
        let mut b = self;
        b.head = head;
        b
    }

    pub fn as_block(&self) -> Block {
        let mut data = match self.direction {
            Direction::South => 0,
            Direction::West => 1,
            Direction::North => 2,
            Direction::East => 3,
        };
        if self.occupied {
            data |= 0x4;
        }
        if self.head {
            data |= 0x8;
        }
        Block { r#type: self.id, data }
    }
}

impl From<BedBuilder> for Block {
    fn from(b: BedBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PoweredRailBuilder {
    id: BlockType,
    shape: RailShape,
    powered: bool,
}

impl PoweredRailBuilder {
    pub const fn new(id: BlockType) -> Self {
        Self { id, shape: RailShape::FlatNorthSouth, powered: false }
    }

    pub fn shape(self, shape: RailShape) -> Self {
        let mut b = self;
        b.shape = shape;
        b
    }

    pub fn powered(self, powered: bool) -> Self {
        let mut b = self;
        b.powered = powered;
        b
    }

    pub fn as_block(&self) -> Block {
        let mut data = (self.shape as u8) & 0x7;
        if self.powered {
            data |= 0x8;
        }
        Block { r#type: self.id, data }
    }
}

impl From<PoweredRailBuilder> for Block {
    fn from(b: PoweredRailBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PistonBuilder {
    id: BlockType,
    facing: PistonFacing,
    extended: bool,
}

impl PistonBuilder {
    pub const fn new(id: BlockType) -> Self {
        Self { id, facing: PistonFacing::Up, extended: false }
    }

    pub fn facing(self, facing: PistonFacing) -> Self {
        let mut b = self;
        b.facing = facing;
        b
    }

    pub fn extended(self, extended: bool) -> Self {
        let mut b = self;
        b.extended = extended;
        b
    }

    pub fn as_block(&self) -> Block {
        let mut data = self.facing as u8;
        if self.extended {
            data |= 0x8;
        }
        Block { r#type: self.id, data }
    }
}

impl From<PistonBuilder> for Block {
    fn from(b: PistonBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PistonHeadBuilder {
    id: BlockType,
    facing: PistonFacing,
    sticky: bool,
}

impl PistonHeadBuilder {
    pub const fn new() -> Self {
        Self { id: BLOCK_PISTON_HEAD, facing: PistonFacing::Up, sticky: false }
    }

    pub fn facing(self, facing: PistonFacing) -> Self {
        let mut b = self;
        b.facing = facing;
        b
    }

    pub fn sticky(self, sticky: bool) -> Self {
        let mut b = self;
        b.sticky = sticky;
        b
    }

    pub fn as_block(&self) -> Block {
        let mut data = self.facing as u8;
        if self.sticky {
            data |= 0x8;
        }
        Block { r#type: self.id, data }
    }
}

impl From<PistonHeadBuilder> for Block {
    fn from(b: PistonHeadBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DoorBuilder {
    id: BlockType,
    rotation: u8,
    open: bool,
    upper_half: bool,
}

impl DoorBuilder {
    pub const fn new(id: BlockType) -> Self {
        Self { id, rotation: 0, open: false, upper_half: false }
    }

    pub fn rotation(self, rotation: u8) -> Self {
        let mut b = self;
        b.rotation = rotation & 0x3;
        b
    }

    pub fn open(self, open: bool) -> Self {
        let mut b = self;
        b.open = open;
        b
    }

    pub fn upper_half(self, upper_half: bool) -> Self {
        let mut b = self;
        b.upper_half = upper_half;
        b
    }

    pub fn as_block(&self) -> Block {
        let mut data = self.rotation & 0x3;
        if self.open {
            data |= 0x4;
        }
        if self.upper_half {
            data |= 0x8;
        }
        Block { r#type: self.id, data }
    }
}

impl From<DoorBuilder> for Block {
    fn from(b: DoorBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SignBuilder {
    id: BlockType,
    rotation: u8,
}

impl SignBuilder {
    pub const fn new() -> Self {
        Self { id: BLOCK_SIGN, rotation: 0 }
    }

    pub fn rotation(self, rotation: u8) -> Self {
        let mut b = self;
        b.rotation = rotation & 0xF;
        b
    }

    pub fn as_block(&self) -> Block {
        Block { r#type: self.id, data: self.rotation & 0xF }
    }
}

impl From<SignBuilder> for Block {
    fn from(b: SignBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RailBuilder {
    id: BlockType,
    shape: RailShape,
}

impl RailBuilder {
    pub const fn new() -> Self {
        Self { id: BLOCK_RAIL, shape: RailShape::FlatNorthSouth }
    }

    pub fn shape(self, shape: RailShape) -> Self {
        let mut b = self;
        b.shape = shape;
        b
    }

    pub fn as_block(&self) -> Block {
        Block { r#type: self.id, data: self.shape as u8 }
    }
}

impl From<RailBuilder> for Block {
    fn from(b: RailBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LeverBuilder {
    id: BlockType,
    mount: LeverMount,
    on: bool,
}

impl LeverBuilder {
    pub const fn new() -> Self {
        Self { id: BLOCK_LEVER, mount: LeverMount::FloorEastWest, on: false }
    }

    pub fn mount(self, mount: LeverMount) -> Self {
        let mut b = self;
        b.mount = mount;
        b
    }

    pub fn on(self, on: bool) -> Self {
        let mut b = self;
        b.on = on;
        b
    }

    pub fn as_block(&self) -> Block {
        let mut data = self.mount as u8;
        if self.on {
            data |= 0x8;
        }
        Block { r#type: self.id, data }
    }
}

impl From<LeverBuilder> for Block {
    fn from(b: LeverBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ButtonBuilder {
    id: BlockType,
    mount: ButtonMount,
    pressed: bool,
}

impl ButtonBuilder {
    pub const fn new() -> Self {
        Self { id: BLOCK_BUTTON_STONE, mount: ButtonMount::WestWall, pressed: false }
    }

    pub fn mount(self, mount: ButtonMount) -> Self {
        let mut b = self;
        b.mount = mount;
        b
    }

    pub fn pressed(self, pressed: bool) -> Self {
        let mut b = self;
        b.pressed = pressed;
        b
    }

    pub fn as_block(&self) -> Block {
        let mut data = self.mount as u8;
        if self.pressed {
            data |= 0x8;
        }
        Block { r#type: self.id, data }
    }
}

impl From<ButtonBuilder> for Block {
    fn from(b: ButtonBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SnowLayerBuilder {
    id: BlockType,
    height: u8,
}

impl SnowLayerBuilder {
    pub const fn new() -> Self {
        Self { id: BLOCK_SNOW_LAYER, height: 0 }
    }

    pub fn height(self, height: u8) -> Self {
        let mut b = self;
        b.height = height.min(7);
        b
    }

    pub fn as_block(&self) -> Block {
        Block { r#type: self.id, data: self.height & 0x7 }
    }
}

impl From<SnowLayerBuilder> for Block {
    fn from(b: SnowLayerBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TrapdoorBuilder {
    id: BlockType,
    direction: Direction,
    open: bool,
}

impl TrapdoorBuilder {
    pub const fn new() -> Self {
        Self { id: BLOCK_TRAPDOOR, direction: Direction::South, open: false }
    }

    pub fn facing(self, direction: Direction) -> Self {
        let mut b = self;
        b.direction = direction;
        b
    }

    pub fn open(self, open: bool) -> Self {
        let mut b = self;
        b.open = open;
        b
    }

    pub fn as_block(&self) -> Block {
        let mut data = match self.direction {
            Direction::South => 0,
            Direction::North => 1,
            Direction::East => 2,
            Direction::West => 3,
        };
        if self.open {
            data |= 0x4;
        }
        Block { r#type: self.id, data }
    }
}

impl From<TrapdoorBuilder> for Block {
    fn from(b: TrapdoorBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PumpkinBuilder {
    id: BlockType,
    direction: Direction,
}

impl PumpkinBuilder {
    pub const fn new(id: BlockType) -> Self {
        Self { id, direction: Direction::South }
    }

    pub fn facing(self, direction: Direction) -> Self {
        let mut b = self;
        b.direction = direction;
        b
    }

    pub fn as_block(&self) -> Block {
        let data = match self.direction {
            Direction::South => 0,
            Direction::West => 1,
            Direction::North => 2,
            Direction::East => 3,
        };
        Block { r#type: self.id, data }
    }
}

impl From<PumpkinBuilder> for Block {
    fn from(b: PumpkinBuilder) -> Self {
        b.as_block()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RepeaterBuilder {
    id: BlockType,
    direction: Direction,
    delay: u8,
}

impl RepeaterBuilder {
    pub const fn new(id: BlockType) -> Self {
        Self { id, direction: Direction::North, delay: 0 }
    }

    pub fn facing(self, direction: Direction) -> Self {
        let mut b = self;
        b.direction = direction;
        b
    }

    pub fn delay(self, delay: u8) -> Self {
        let mut b = self;
        b.delay = delay.min(3);
        b
    }

    pub fn delay_ticks(&self) -> i32 {
        (self.delay as i32 + 1) * 2
    }

    pub fn as_block(&self) -> Block {
        let mut data = match self.direction {
            Direction::North => 0,
            Direction::East => 1,
            Direction::South => 2,
            Direction::West => 3,
        };
        data |= (self.delay & 0x3) << 2;
        Block { r#type: self.id, data }
    }
}

impl From<RepeaterBuilder> for Block {
    fn from(b: RepeaterBuilder) -> Self {
        b.as_block()
    }
}

pub const FLOWING_WATER: FluidBuilder = FluidBuilder::new(BLOCK_WATER_FLOWING);
pub const WATER: FluidBuilder = FluidBuilder::new(BLOCK_WATER_STILL);
pub const FLOWING_LAVA: FluidBuilder = FluidBuilder::new(BLOCK_LAVA_FLOWING);
pub const LAVA: FluidBuilder = FluidBuilder::new(BLOCK_LAVA_STILL);
pub const SAPLING: SaplingBuilder = SaplingBuilder::new();
pub const LOG: LogBuilder = LogBuilder::new();
pub const LEAVES: LeavesBuilder = LeavesBuilder::new();
pub const DISPENSER: WallFacingBuilder = WallFacingBuilder::new(BLOCK_DISPENSER);
pub const BED: BedBuilder = BedBuilder::new();
pub const POWERED_RAIL: PoweredRailBuilder = PoweredRailBuilder::new(BLOCK_RAIL_POWERED);
pub const DETECTOR_RAIL: PoweredRailBuilder = PoweredRailBuilder::new(BLOCK_RAIL_DETECTOR);
pub const STICKY_PISTON: PistonBuilder = PistonBuilder::new(BLOCK_PISTON_STICKY);
pub const TALL_GRASS: TallGrassBuilder = TallGrassBuilder::new();
pub const PISTON: PistonBuilder = PistonBuilder::new(BLOCK_PISTON);
pub const PISTON_HEAD: PistonHeadBuilder = PistonHeadBuilder::new();
pub const WOOL: WoolBuilder = WoolBuilder::new();
pub const DOUBLE_SLAB: SlabBuilder = SlabBuilder::new(BLOCK_DOUBLE_SLAB);
pub const SLAB: SlabBuilder = SlabBuilder::new(BLOCK_SLAB);
pub const TORCH: TorchBuilder = TorchBuilder::new(BLOCK_TORCH);
pub const WOODEN_STAIRS: StairsBuilder = StairsBuilder::new(BLOCK_STAIRS_WOOD);
pub const FURNACE: WallFacingBuilder = WallFacingBuilder::new(BLOCK_FURNACE);
pub const LIT_FURNACE: WallFacingBuilder = WallFacingBuilder::new(BLOCK_FURNACE_LIT);
pub const SIGN: SignBuilder = SignBuilder::new();
pub const WOODEN_DOOR: DoorBuilder = DoorBuilder::new(BLOCK_DOOR_WOOD);
pub const LADDER: WallFacingBuilder = WallFacingBuilder::new(BLOCK_LADDER);
pub const RAIL: RailBuilder = RailBuilder::new();
pub const COBBLESTONE_STAIRS: StairsBuilder = StairsBuilder::new(BLOCK_STAIRS_COBBLESTONE);
pub const WALL_SIGN: WallFacingBuilder = WallFacingBuilder::new(BLOCK_SIGN_WALL);
pub const LEVER: LeverBuilder = LeverBuilder::new();
pub const IRON_DOOR: DoorBuilder = DoorBuilder::new(BLOCK_DOOR_IRON);
pub const REDSTONE_TORCH: TorchBuilder = TorchBuilder::new(BLOCK_REDSTONE_TORCH_OFF);
pub const LIT_REDSTONE_TORCH: TorchBuilder = TorchBuilder::new(BLOCK_REDSTONE_TORCH_ON);
pub const STONE_BUTTON: ButtonBuilder = ButtonBuilder::new();
pub const SNOW_LAYER: SnowLayerBuilder = SnowLayerBuilder::new();
pub const PUMPKIN: PumpkinBuilder = PumpkinBuilder::new(BLOCK_PUMPKIN);
pub const JACK_O_LANTERN: PumpkinBuilder = PumpkinBuilder::new(BLOCK_PUMPKIN_LIT);
pub const REDSTONE_REPEATER: RepeaterBuilder = RepeaterBuilder::new(BLOCK_REDSTONE_REPEATER_OFF);
pub const LIT_REDSTONE_REPEATER: RepeaterBuilder = RepeaterBuilder::new(BLOCK_REDSTONE_REPEATER_ON);
pub const TRAPDOOR: TrapdoorBuilder = TrapdoorBuilder::new();
