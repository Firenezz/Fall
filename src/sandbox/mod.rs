use bevy::prelude::*;

use common::units::temperature::*;
use simulation::temperature::Temperature;

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, temperature_sandbox);
    }
}

pub fn temperature_sandbox(mut commands: Commands) {
    commands.spawn(Temperature::new(Celsius::new(20.0)));
}