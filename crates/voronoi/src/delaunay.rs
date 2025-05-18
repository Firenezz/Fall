use bevy::{log::trace_span};
use std::fmt::Debug;

use crate::geometry::locations::Coord;

#[bevy::log::tracing::instrument]
pub fn triangulate<C: Coord<Inner = f32> + Debug>(points: Vec<C>) -> Vec<[u32; 3]> {
    
    trace_span!("triangulate").enter();

    todo!()
}