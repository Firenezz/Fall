use bevy::prelude::UVec2;

pub trait Grid {
    fn get_size(&self) -> UVec2;
    fn get_cell(&self, x: i32, y: i32) -> Box<dyn Cell>;
    fn set_cell(&mut self, x: i32, y: i32, cell: Box<dyn Cell>);
}

pub trait Cell: Send + Sync {}