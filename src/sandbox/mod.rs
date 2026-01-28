use bevy::prelude::*;

use common::units::temperature::*;
use simulation::temperature::Temperature;

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

pub fn temperature_sandbox(mut commands: Commands) {
    commands.spawn(Temperature { value: 20.0 });
}