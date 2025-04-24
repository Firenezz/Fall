use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileStorage;
use common::resources::MapSize;

use crate::SimulationRate;

#[derive(Component, Default, Reflect, Debug, PartialEq, PartialOrd)]
#[reflect(Component)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Temperature {
    pub value: f32,
}

impl Temperature {
    pub fn set_temperature(&mut self, value: f32) {
        self.value = value;
    }

    pub fn get_temperature(&self) -> f32 {
        self.value
    }
}

#[derive(Component, Default, Reflect, Debug, PartialEq, PartialOrd)]
#[reflect(Component)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HeatCell {
    pub temperature: Temperature,
    pub conductivity: ThermalConductivity,
}

#[derive(Component, Default, Reflect, Debug, PartialEq, PartialOrd)]
#[reflect(Component)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ThermalConductivity {
    pub value: f32,
}

/// Calculates heat transfer between two HeatCells over a time step
/// 
/// # Arguments
/// * `cell1` - First heat cell
/// * `cell2` - Second heat cell
/// * `dt` - Time step in seconds
/// * `transfer_coefficient` - How much heat can transfer between cells (0.0 to 1.0)
/// 
/// # Returns
/// Tuple of (new_temp1, new_temp2)
pub fn calculate_heat_transfer(
    cell1: &HeatCell,
    cell2: &HeatCell,
    dt: Duration,
    transfer_coefficient: f32,
) -> (f32, f32) {
    // Calculate the average conductivity between the two cells
    let avg_conductivity = (cell1.conductivity.value + cell2.conductivity.value) * 0.5;
    
    // Calculate temperature difference
    let temp_diff = cell2.temperature.value - cell1.temperature.value;
    
    // Calculate heat transfer based on conductivity, temperature difference, and time
    let heat_transfer = temp_diff * avg_conductivity * dt.as_secs_f32() * transfer_coefficient;
    
    // Return new temperatures
    (
        cell1.temperature.value + heat_transfer,
        cell2.temperature.value - heat_transfer
    )
}

pub struct ThermalPlugin;

impl Plugin for ThermalPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Temperature>()
            .register_type::<HeatCell>()
            .register_type::<ThermalConductivity>()
            .add_systems(Update, thermal_conduction);
    }
}

fn thermal_conduction(
    mut tile_heat_query: Query<(&mut HeatCell, &bevy_ecs_tilemap::tiles::TilePos, &TileStorage)>,
    mut simulation_rate: ResMut<SimulationRate>,
    size: Res<MapSize>,
    time: Res<Time>,
) {
    simulation_rate.rate.tick(time.delta());
    if simulation_rate.rate.just_finished() {
        use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
        let map_size = bevy_ecs_tilemap::map::TilemapSize::from(size.into_inner().0);
        
        // First pass: collect all temperature changes
        let mut temp_accumulators = vec![0.0; (map_size.x * map_size.y) as usize];
        for (heat_cell, tile_pos, tile_storage) in tile_heat_query.iter() {
            let neighbors = Neighbors::get_square_neighboring_positions(tile_pos, &map_size, false)
                .entities(tile_storage);
            
            let mut total_change = 0.0;
            for neighbor in neighbors.iter() {
                if let Ok((neighbor_cell, neighbor_pos, _)) = tile_heat_query.get(*neighbor) {
                    let (new_temp1, new_temp2) = calculate_heat_transfer(heat_cell, neighbor_cell, time.delta(), 0.5);
                    total_change += new_temp1 - heat_cell.temperature.value;
                    // Update neighbor changes
                    temp_accumulators[neighbor_pos.to_index(&map_size)] += new_temp2 - neighbor_cell.temperature.value;
                }
            }
            temp_accumulators[tile_pos.to_index(&map_size)] = total_change;
        }
        
        // Second pass: apply all temperature changes
        for (mut heat_cell, tile_pos, _) in tile_heat_query.iter_mut() {
            let index = tile_pos.to_index(&map_size);
            heat_cell.temperature.value += temp_accumulators[index];
        }
    }
}
