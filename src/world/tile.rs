use bevy::reflect::Reflect;

use bevy::prelude::*;


#[derive(Default, Component, Reflect, Clone, Copy, Debug)]
pub struct TileTemperature(pub f32);

//#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Component, Reflect, Clone, Copy, Debug)]
pub struct TileMass(pub f32);

#[derive(Bundle, Default, Reflect, Clone, Copy, Debug)]
pub struct FallTileBundle {
    pub tile_mass: TileMass,
    pub tile_temperature: TileTemperature,
}