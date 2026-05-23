pub mod kelvin;
pub mod celsius;
pub mod fahrenheit;

use bevy::app::Plugin;
// Re-export the units
pub use kelvin::*;
pub use celsius::*;
pub use fahrenheit::*;

type TemperatureValue = f32;

pub struct TemperaturePlugin;

impl Plugin for TemperaturePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .register_type::<Kelvin>()
            .register_type::<Celsius>()
            .register_type::<Fahrenheit>();
    }
}