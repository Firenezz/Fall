pub mod tile;
pub mod layer;

use bevy::{prelude::*, platform::collections::HashSet};
use bevy_ecs_tilemap::prelude::*;
use layer::Layer;
use rand::{rngs::ThreadRng, Rng};

use crate::{loading::TextureAssets, GameState};
use crate::states::generation::GenerationState;
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SolidTiles>()
            .init_resource::<GenerationSeed>()
            .init_non_send_resource::<SeededRng<ThreadRng>>()
            .add_plugins(TilemapPlugin)
            .add_plugins(layer::LayerPlugin)
            .add_systems(OnEnter(GenerationState::Initializing), build_world)
            .add_systems(OnExit(GameState::Playing), drop_world);
    }
}

#[derive(Resource)]
pub struct SolidTiles (pub HashSet<(i8, i8)>);

#[derive(Resource)]
pub struct SeededRng<R: Rng + ?Sized>(R);

#[derive(Resource)]
pub struct GenerationSeed(u32);

impl Default for SolidTiles {
    fn default() -> Self {
        Self(HashSet::new())
    }
}

impl Default for GenerationSeed {
    fn default() -> Self {
        let seed = rand::rng().random();
        Self(seed)
    }
}

impl Default for SeededRng<ThreadRng> {
    fn default() -> Self {
        Self(rand::rng())
    }
}

/// The world is a collection of layers.
/// 
#[derive(Component, Reflect)]
pub struct Grid {
    size: bevy::math::UVec2,
    /// The layers of the world.    
    /// 
    /// The first layer is the base layer, and the layers are rendered in order of their id.
    /// 
    /// The layer id corresponds to the layer index in the tilemap and the z-index in the 3D scene. 
    layers: Vec<Layer>,
}

impl Grid {
    pub fn get_layer(&self, id: u32) -> Option<&Layer> {
        self.layers.iter().find(|layer| layer.id == id)
    }

    pub fn get_layer_mut(&mut self, id: u32) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|layer| layer.id == id)
    }

    pub fn get_layers(&self) -> &[Layer] {
        &self.layers
    }

    pub fn get_layers_mut(&mut self) -> &mut [Layer] {
        &mut self.layers
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self { size: CHUNK_SIZE.into(), layers: vec![] }
    }
}

const CHUNK_SIZE: UVec2 = UVec2 { x: 32, y: 32 };

fn drop_world(mut commands: Commands, tilemap_query: Query<Entity, With<TileStorage>>) {
    for tilemap_entity in tilemap_query.iter() {
        commands.entity(tilemap_entity).despawn();
    }
}

fn build_world(mut commands: Commands, mut next_state: ResMut<NextState<GenerationState>>) {
    commands.spawn_empty()
        .insert(Name::new("World"))
        .insert(Grid::default())
        .insert((
            GlobalTransform::default(),
            InheritedVisibility::default(),
            Visibility::default(),
        ));
    next_state.set(GenerationState::Generating);
}

