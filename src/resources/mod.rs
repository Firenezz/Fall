use bevy::prelude::Resource;


#[derive(Resource)]
pub struct Tileset {
    pub tiles: Vec<Tile>,
}

#[derive(Resource)]
pub struct Tile {
    pub id: u32,
}

#[derive(Resource)]
pub struct ElementConfigs {
    pub elements: Vec<ElementConfig>,
}

#[derive(Resource)]
pub struct ElementConfig { 
    pub id: u32,
    pub name: String,
    pub symbol: String,
    pub density: f32,
    pub specific_heat: f32,
}

impl Default for Tileset {
    fn default() -> Self {
        Self { tiles: vec![] }
    }
}


