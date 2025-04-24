use bevy::prelude::*;

pub mod temperature;

#[derive(Resource)]
pub struct SimulationRate {
    pub rate: Timer,
}

impl Default for SimulationRate {
    fn default() -> Self {
        use std::time::Duration;
        Self { rate: Timer::new(Duration::from_millis(200), TimerMode::Repeating) }
    }
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SimulationRate>()
            .add_plugins(temperature::ThermalPlugin);
    }
}

