use bevy::{prelude::*, utils::tracing::{self, Instrument}};
use bevy_ecs_tilemap::{map::{TilemapId, TilemapSize}, tiles::{TileBundle, TilePos, TileStorage}, TilemapBundle};
use common::resources::MapSize;
use crate::states::generation::GenerationState;

pub struct LayerPlugin;

impl Plugin for LayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(MapSize(UVec2::new(2, 2)))
            .add_systems(OnEnter(GenerationState::Generating), ((build_background_layer, build_solid_layer), next_generation_step).chain());
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
pub struct LayerBundle {
    layer: Layer,
    tilemap_storage: TileStorage,
}

impl Default for LayerBundle {
    fn default() -> Self {
        Self { layer: Layer::default(), tilemap_storage: TileStorage::default() }
    }
}

// Init methods

#[derive(Default)]
pub struct LayerBuilder {
    name: Option<String>,
    layer_type: Option<LayerType>,
    size: Option<TilemapSize>,
    transform: Option<Transform>,
}

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

    pub fn build(self, commands: &mut Commands) -> Entity {

        info!("Building layer");

        let layer_entity = commands.spawn(LayerBundle::default()).id();
        
        if let Some(name) = self.name {
            commands.entity(layer_entity).insert(Name::new(name));
        }

        if let Some(transform) = self.transform {
            commands.entity(layer_entity).insert(transform);
        }

        if let Some(size) = self.size {
            use bevy_ecs_tilemap::prelude::*;
            let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
            let grid_size = TilemapGridSize { x: size.x as f32 * tile_size.x, y: size.y as f32 * tile_size.y };

            let mut tile_storage = TileStorage::empty(size.into());
            fill_layer(
                TilemapId(layer_entity),
                size,
                commands,
                &mut tile_storage,
            );

            commands.entity(layer_entity).insert(TilemapBundle {
                grid_size,
                tile_size,
                storage: tile_storage,
                ..Default::default()
            });
        }

        layer_entity
    }
}

#[tracing::instrument(name = "Building solid layer", skip(commands, size, grid_query))]
fn build_background_layer(mut commands: Commands, size: Res<MapSize>, mut grid_query: Query<Entity, With<super::Grid>>) {

    use tracing::info;

    info!("Building background layer");

    let map_size = TilemapSize { x: size.0.x, y: size.0.y };
    let grid_entity = grid_query.single_mut();

    let layer_entity = 
        LayerBuilder::new()
            .with_name("Background Layer")
            .with_type(LayerType::Background)
            .with_size(map_size)
            .build(&mut commands);

    commands.entity(grid_entity)
        .add_child(layer_entity);
}

#[tracing::instrument(name = "Building solid layer", skip(commands, size, grid_query))]
fn build_solid_layer(mut commands: Commands, size: Res<MapSize>, mut grid_query: Query<Entity, With<super::Grid>>) {

    use tracing::info;

    info!("Building solid layer");

    let grid_entity = grid_query.single_mut();

    let layer_entity = 
        LayerBuilder::new()
            .with_name("Solid Layer")
            .with_type(LayerType::Solid)
            .with_size(TilemapSize::from(size.into_inner().0))
            .build(&mut commands);

    commands.entity(grid_entity)
        .add_child(layer_entity);
}

#[tracing::instrument(name = "Filling layer", skip(commands, tile_storage))]
fn fill_layer(
    tilemap_id: TilemapId,
    size: TilemapSize,
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
) {

    use simulation::temperature::*;

    commands.entity(tilemap_id.0).instrument(info_span!("Generating children")).inner_mut().with_children(|parent| {
        for x in 0..size.x {
            for y in 0..size.y {
                let tile_pos = TilePos { x, y };
                let tile_entity = parent.spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id,
                    ..Default::default()
                })
                .insert(HeatCell::default())
                .id();
                tile_storage.set(&tile_pos, tile_entity);
            }
        }
        info!("Tile storage: {:?}", tile_storage);
    });
}

fn next_generation_step(mut commands: Commands, mut next_state: ResMut<NextState<GenerationState>>) {
    next_state.set(GenerationState::Done);
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
    NPC = 5,
    Solid = 6, // Walls, floors, etc.

}

