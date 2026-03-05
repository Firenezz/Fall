

pub trait Grid {
    fn get_size(&self) -> UVec2;
    fn get_cell(&self, x: i32, y: i32) -> Cell;
    fn set_cell(&mut self, x: i32, y: i32, cell: impl Into<Cell>);
}

pub trait Cell {
    
}