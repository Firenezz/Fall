use bevy::{math::{IVec2, UVec2, Vec2}, prelude::{Component, Reflect}};

/// Integer position on the world grid (simulation, buildings, tilemap alignment).
/// Convert at boundaries to `bevy_ecs_tilemap::tiles::TilePos` when needed.
#[derive(Reflect, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CellPos(pub IVec2);

#[derive(Reflect, Default, Clone, Copy, PartialEq, Debug)]
pub struct WorldPos(pub Vec2);

#[derive(Reflect, Clone, Copy, PartialEq, Debug)]
#[non_exhaustive]
pub enum Coordinate {
    World(WorldPos),
    Cell(CellPos),
}

impl Coordinate {
    pub fn new_world(x: f32, y: f32) -> Self {
        Self::World(WorldPos(Vec2::new(x, y)))
    }

    pub fn new_cell(x: i32, y: i32) -> Self {
        Self::Cell(CellPos(IVec2::new(x, y)))
    }

    pub fn get_vec2(&self) -> Vec2 {
        match self {
            Self::World(pos) => pos.0,
            Self::Cell(pos) => Vec2::new(pos.0.x as f32, pos.0.y as f32),
        }
    }

    pub fn get_ivec2(&self) -> IVec2 {
        match self {
            Self::World(pos) => IVec2::new(pos.0.x as i32, pos.0.y as i32),
            Self::Cell(pos) => pos.0,
        }
    }

    pub fn get_uvec2(&self) -> UVec2 {
        match self {
            Self::World(pos) => UVec2::new(pos.0.x as u32, pos.0.y as u32),
            Self::Cell(pos) => UVec2::new(pos.0.x as u32, pos.0.y as u32),
        }
    }

    pub fn as_world_pos(&self) -> WorldPos {
        match self {
            Self::World(pos) => *pos,
            Self::Cell(pos) => WorldPos(Vec2::new(pos.0.x as f32, pos.0.y as f32)),
        }
    }
}

/// Rotation of an occupied grid cell in 90° steps.
#[derive(Component, Reflect, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Rotation {
    #[default]
    None = 0,
    Clockwise90,
    Clockwise180,
    Clockwise270,
}

impl Rotation {
    pub fn from_u8(rotation: u8) -> Self {
        let rotation = rotation % 4;
        Self::new(rotation)
    }

    pub fn new(rotation: u8) -> Self {
        match rotation {
            0 => Self::None,
            1 => Self::Clockwise90,
            2 => Self::Clockwise180,
            3 => Self::Clockwise270,
            _ => panic!("Invalid rotation: {}", rotation),
        }
    }
    pub fn get_rotated(&self, to_rotate: CellPos) -> CellPos {
        match self {
            Self::None => to_rotate,
            Self::Clockwise90 => CellPos(IVec2::new(1, 0)),
            Self::Clockwise180 => CellPos(IVec2::new(0, 1)),
            Self::Clockwise270 => CellPos(IVec2::new(-1, 0)),
        }
    }

    pub fn get_next_rotation(&self, amount: u8) -> Self {
        match self {
            Self::None => Self::new(amount),
            Self::Clockwise90 => Self::new(amount + 1),
            Self::Clockwise180 => Self::new(amount + 2),
            Self::Clockwise270 => Self::new(amount + 3),
        }
    }
}

#[derive(Component, Reflect, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EntityID(pub u64);

impl EntityID {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Component, Reflect, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct OccupiedCell {
    pub position: CellPos,
}

impl OccupiedCell {
    pub fn new(position: CellPos) -> Self {
        Self { position }
    }
}
