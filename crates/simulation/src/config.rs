use bevy::{ecs::resource::Resource, time::Timer};



#[derive(Resource)]
pub struct SimulationRate {
    pub rate: Timer,
    pub accumulator: f32,
    pub time_scale: f32,
}