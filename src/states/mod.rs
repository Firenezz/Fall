#![allow(dead_code)]
// CHORE: Remove this once we have a proper way to handle states

use bevy::prelude::*;

pub mod generation;

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, _app: &mut App) {
        
    }
}
