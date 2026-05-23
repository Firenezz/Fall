#![allow(dead_code)]
// CHORE: Remove this once we have a proper way to handle systems

use bevy::app::Plugin;

pub struct TemperatureSystemPlugin;

impl Plugin for TemperatureSystemPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        todo!()
    }
}