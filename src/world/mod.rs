pub mod tile;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tile::FallTileBundle;

use crate::{loading::TextureAssets, GameState};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(TilemapPlugin)
            .add_systems(OnEnter(GameState::Playing), build_world)
            .add_systems(OnExit(GameState::Playing), drop_world);
    }
}

#[derive(Component)]
pub struct World {
    size: bevy::math::UVec2,
}

const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 16.0, y: 16.0 };
const CHUNK_SIZE: UVec2 = UVec2 { x: 32, y: 32 };

fn drop_world(mut commands: Commands, tilemap_query: Query<Entity, With<TileStorage>>) {
    for tilemap_entity in tilemap_query.iter() {
        commands.entity(tilemap_entity).despawn_recursive();
    }
}

fn build_world(mut commands: Commands, textures: Res<TextureAssets>) {
    let tilemap_entity = commands.spawn_empty().insert(Name::new("Tilemap")).id();
    let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());

    // Spawn the elements of the tilemap.
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .insert(Name::new("Tile"))
                .id();
            commands.entity(tilemap_entity).add_child(tile_entity);
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let transform = Transform::from_translation(Vec3::new(
        0.0,
        0.0,
        1.0,
    ));
    let texture_handle: Handle<Image> = textures.tile_atlas.clone();
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: TILE_SIZE.into(),
        size: CHUNK_SIZE.into(),
        storage: tile_storage,
        tile_size: TILE_SIZE,
        transform,
        texture: TilemapTexture::Single(texture_handle),
        render_settings: TilemapRenderSettings {
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(FallTileBundle::default());

}