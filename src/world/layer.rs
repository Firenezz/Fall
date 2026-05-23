use bevy::prelude::*;
use tracing;
use bevy_ecs_tilemap::{map::TilemapSize, tiles::{TileBundle, TileColor, TilePos, TileStorage}};
use common::{resources::MapSize, units::temperature::{Celsius, Kelvin}};
use crate::{loading::TextureAssets, states::generation::GenerationState};
use simulation::temperature::{HeatCell, Temperature, ThermalConductivity};

pub struct LayerPlugin;

impl Plugin for LayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(MapSize(UVec2::new(100, 100)))
            .add_systems(
                OnEnter(GenerationState::Generating),
                (
                    (/*build_background_layer,*/ build_solid_layer),
                    populate_layer_heat_cells,
                )
                    .chain(),
            )
            .add_systems(Update, update_tile_color_based_on_temperature);
    }
}

#[derive(Component, Reflect)]
pub struct Layer {
    pub id: u32,
    pub tile_storage: TileStorage,
    pub layer_type: LayerType,
}

impl Default for Layer {
    fn default() -> Self {
        Self { id: 0, tile_storage: TileStorage::default(), layer_type: LayerType::Empty }
    }
}


#[derive(Bundle)]
#[derive(Default)]
pub struct LayerBundle {
    layer: Layer,
    tilemap_storage: TileStorage,
}


#[allow(dead_code)]
pub enum Texture {
    String(String),
    Handle(Handle<Image>),
}

// Init methods

#[derive(Default)]
pub struct LayerBuilder {
    name: Option<String>,
    layer_type: Option<LayerType>,
    size: Option<TilemapSize>,
    transform: Option<Transform>,
    texture: Option<Texture>,
    tile_function: Option<Box<dyn Fn(TilePos, Entity) -> TileBundle>>,
}

#[allow(dead_code)]
impl LayerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_type(mut self, layer_type: LayerType) -> Self {
        self.layer_type = Some(layer_type);
        self
    }

    pub fn with_size(mut self, size: TilemapSize) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self
    }

    pub fn with_texture(mut self, texture: Texture) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn with_tile_function(mut self, tile_function: impl Fn(TilePos, Entity) -> TileBundle + 'static) -> Self {
        self.tile_function = Some(Box::new(tile_function));
        self
    }

    pub fn build(self, commands: &mut Commands, resources: &Res<TextureAssets>) -> Entity {

        info!("Building layer");

        let layer_entity = commands.spawn(LayerBundle::default()).id();
        let texture = match self.texture {
            Some(Texture::String(_name)) => {
                resources.tile_atlas.clone()
            }
            Some(Texture::Handle(handle)) => handle,
            None => resources.tile_atlas.clone()
        };
        
        if let Some(name) = self.name {
            commands.entity(layer_entity).insert(Name::new(name));
        }

        if let Some(transform) = self.transform {
            commands.entity(layer_entity).insert(transform);
        }

        if let Some(size) = self.size {
            use bevy_ecs_tilemap::prelude::*;
            let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
            let grid_size = TilemapGridSize { x: tile_size.x, y: tile_size.y };

            info!("LayerBuilder::build - size: {:?}, tile_size: {:?}, grid_size: {:?}", size, tile_size, grid_size);

            let mut tile_storage = TileStorage::empty(size);
            helpers::filling::fill_tilemap(TileTextureIndex(5), size, TilemapId(layer_entity), commands, &mut tile_storage);

            let tilemap_size = TilemapSize { x: size.x, y: size.y };

            if let Some(tile_function) = self.tile_function {
                commands.entity(layer_entity).with_children(|parent| {
                    for x in 0..size.x {
                        for y in 0..size.y {
                            let tile_pos = TilePos { x, y };
                            let tile_bundle = tile_function(tile_pos, layer_entity);
                            let tile_entity = parent.spawn(tile_bundle).id();
                            tile_storage.set(&tile_pos, tile_entity);
                        }
                    }
                });
            }

            commands.entity(layer_entity).insert(TilemapBundle {
                grid_size,
                size: tilemap_size,
                tile_size,
                storage: tile_storage,
                texture: TilemapTexture::Single(texture),
                ..Default::default()
            });
        }

        layer_entity
    }
}

#[allow(dead_code)]
#[tracing::instrument(name = "Building solid layer", skip(commands, size, grid_query, resources))]
fn build_background_layer(mut commands: Commands, size: Res<MapSize>, mut grid_query: Query<Entity, With<super::Grid>>, resources: Res<TextureAssets>) -> Result<(), BevyError> {

    use tracing::info;

    info!("Building background layer");

    let map_size = TilemapSize { x: size.0.x, y: size.0.y };
    let grid_entity = grid_query.single_mut()?;

    let layer_entity = 
        LayerBuilder::new()
            .with_name("Background Layer")
            .with_type(LayerType::Background)
            .with_size(map_size)
            .build(&mut commands, &resources);

    commands.entity(grid_entity)
        .add_child(layer_entity);

    Ok(())
}

#[tracing::instrument(name = "Building solid layer", skip(commands, size, grid_query, resources))]
fn build_solid_layer(mut commands: Commands, size: Res<MapSize>, mut grid_query: Query<Entity, With<super::Grid>>, resources: Res<TextureAssets>) -> Result<(), BevyError> {

    use tracing::info;

    info!("Building solid layer");
    info!("MapSize resource: {:?}", size.0);

    let map_size = TilemapSize { x: size.0.x, y: size.0.y };
    info!("TilemapSize created: {:?}", map_size);
    
    let grid_entity = grid_query.single_mut()?;

    let layer_entity = 
        LayerBuilder::new()
            .with_name("Solid Layer")
            .with_type(LayerType::Solid)
            .with_size(map_size)
            .build(&mut commands, &resources);

    commands.entity(grid_entity)
        .add_child(layer_entity);

    Ok(())
}

#[tracing::instrument(name = "Populating heat cells", skip(commands, layer_query))]
fn populate_layer_heat_cells(
    mut commands: Commands,
    layer_query: Query<(&Layer, &TileStorage, &TilemapSize), Added<TileStorage>>,
) {
    for (_layer, tile_storage, size) in &layer_query {

        for x in 0..size.x {
            for y in 0..size.y {
                let tile_pos = TilePos { x, y };
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    match tile_pos {
                        TilePos { x, y } if (x + 10) % 20 == 0 || (y + 10) % 20 == 0 => {
                            commands.entity(tile_entity).insert(HeatCell { temperature: Temperature::new(Celsius::new(5000.0)), conductivity: ThermalConductivity::new(60.0) });
                        }
                        _ => {
                            commands.entity(tile_entity).insert(HeatCell::default().with_temperature(Temperature::new(Celsius::new(0.0))).with_conductivity(ThermalConductivity::new(2.0)));
                        }
                    }
                }
            }
        }
    }
}

fn update_tile_color_based_on_temperature(mut tile_query: Query<(&mut TileColor, &HeatCell)>) {
    for (mut tile_color, temperature) in &mut tile_query {
        tile_color.0 = temperature_to_color(temperature.temperature.value);
    }
}

fn temperature_to_color(temperature: Kelvin) -> Color {
    // Keep 20°C (293.15K) as a visual midpoint (cyan-green).
    let cold_k = 250.0;
    let mid_k = 293.15;
    let hot_k = 1200.0;
    let k = temperature.get_value().clamp(cold_k, hot_k);

    // Anchor colors:
    // cold -> blue, midpoint -> cyan-green, hot -> red
    let cold = (0.08, 0.25, 0.95);
    let mid = (0.00, 0.95, 0.65);
    let hot = (1.00, 0.05, 0.00);

    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let (r, g, b) = if k <= mid_k {
        let t = (k - cold_k) / (mid_k - cold_k);
        (
            lerp(cold.0, mid.0, t),
            lerp(cold.1, mid.1, t),
            lerp(cold.2, mid.2, t),
        )
    } else {
        let t = (k - mid_k) / (hot_k - mid_k);
        (
            lerp(mid.0, hot.0, t),
            lerp(mid.1, hot.1, t),
            lerp(mid.2, hot.2, t),
        )
    };

    Color::srgb(r, g, b)
}

#[derive(Component, Default, Reflect, Debug)]
pub enum LayerType {
    Background = -1,
    #[default]
    Empty = 0,
    Gas = 1,
    GasPipe = 2,
    Liquid = 3,
    LiquidPipe = 4,
    Npc = 5,
    Solid = 6, // Walls, floors, etc.

}

