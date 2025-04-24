use bevy::prelude::*;

#[derive(Resource, Reflect, Default)]
pub struct MapSize(pub UVec2);