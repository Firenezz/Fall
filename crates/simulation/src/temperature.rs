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

#[derive(Component, Reflect, Debug, PartialEq, PartialOrd)]
#[reflect(Component)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ThermalConductivity {
    pub value: f32,
}

impl Default for ThermalConductivity {
    fn default() -> Self {
        Self { value: 1.0 }
    }
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
#[tracing::instrument(name = "Calculating heat transfer", skip(cell1, cell2, dt, transfer_coefficient))]
pub fn calculate_heat_transfer(
    cell1: &HeatCell,
    cell2: &HeatCell,
    dt: Duration,
    transfer_coefficient: f32,
) -> (f32, f32) {
    // Calculate the average conductivity between the two cells
    let avg_conductivity = (cell1.conductivity.value + cell2.conductivity.value) * 0.5;
    
    // Calculate temperature difference (ΔT)
    let temp_diff = cell2.temperature.value - cell1.temperature.value;
    
    // Calculate heat transfer using simplified Fourier's Law: q = -k * ΔT
    // Then multiply by time and divide by 2 since heat is shared between two cells
    // The negative sign ensures heat flows from hot to cold
    let heat_transfer = avg_conductivity * temp_diff * dt.as_secs_f32() * transfer_coefficient * 0.5;

    info!("Heat transfer: {}", heat_transfer);
    info!("Cell 1 temperature: {}", cell1.temperature.value);
    info!("Cell 2 temperature: {}", cell2.temperature.value);
    info!("Avg conductivity: {}", avg_conductivity);
    info!("Transfer coefficient: {}", transfer_coefficient);
    info!("Time: {}", dt.as_secs_f32());
    
    // Return new temperatures
    // Heat flows from hot to cold, so:
    // Hot cell loses heat (negative)
    // Cold cell gains heat (positive)
    (
        heat_transfer,
        -heat_transfer
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
    mut tile_heat_query: Query<(&mut HeatCell, &bevy_ecs_tilemap::tiles::TilePos, &Parent)>,
    layer_query: Query<&TileStorage>,
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
        
        for (heat_cell, tile_pos, parent) in tile_heat_query.iter() {
            if let Ok(tile_storage) = layer_query.get(parent.get()) {
                let neighbors = Neighbors::get_square_neighboring_positions(tile_pos, &map_size, false)
                    .entities(tile_storage);
                
                for neighbor in neighbors.iter() {
                    if let Ok((neighbor_cell, neighbor_pos, _)) = tile_heat_query.get(*neighbor) {
                        let (new_temp1, new_temp2) = calculate_heat_transfer(heat_cell, neighbor_cell, time.delta(), 0.5);
                        temp_accumulators[tile_pos.to_index(&map_size)] += new_temp1;
                        // Update neighbor changes
                        temp_accumulators[neighbor_pos.to_index(&map_size)] += new_temp2;
                    }
                }
            }
        }
        
        // Second pass: apply all temperature changes
        for (mut heat_cell, tile_pos, _) in tile_heat_query.iter_mut() {
            let index = tile_pos.to_index(&map_size);
            heat_cell.temperature.value += temp_accumulators[index];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimulationPlugin;
    use bevy_ecs_tilemap::prelude::*;
    use bevy::ecs::world::World;

    // Helper function to debug entity components
    fn debug_entity_components(world: &World, entity: Entity) {
        eprintln!("\nDebugging entity {:?} components:", entity);
        
        // Check for specific components we expect
        if let Some(pos) = world.get::<TilePos>(entity) {
            eprintln!("- TilePos: {:?}", pos);
        }
        if let Some(heat_cell) = world.get::<HeatCell>(entity) {
            eprintln!("- HeatCell: {:?}", heat_cell);
        }
        if let Some(parent) = world.get::<Parent>(entity) {
            eprintln!("- Parent: {:?}", parent);
        }
        if let Some(tilemap_id) = world.get::<TilemapId>(entity) {
            eprintln!("- TilemapId: {:?}", tilemap_id);
        }

        eprintln!("\nAll components:");
        if let Some(archetype) = world.archetypes().get(world.entity(entity).archetype().id()) {
            for component_id in archetype.components() {
                if let Some(component_info) = world.components().get_info(component_id) {
                    eprintln!("- {:?}", component_info.name());
                }
            }
        }
    }

    fn debug_neighbors(world: &World, tile_storage: &TileStorage, center_pos: &TilePos, map_size: &TilemapSize) {
        eprintln!("\nDebugging neighbors of tile at {:?}:", center_pos);
        
        let neighbors = helpers::square_grid::neighbors::Neighbors::get_square_neighboring_positions(
            center_pos,
            map_size,
            false
        );
        
        eprintln!("Neighbor positions: {:?}", neighbors);
        
        for neighbor_pos in neighbors.iter() {
            if let Some(neighbor_entity) = tile_storage.get(neighbor_pos) {
                eprintln!("\nNeighbor at position {:?}:", neighbor_pos);
                debug_entity_components(world, neighbor_entity);
            } else {
                eprintln!("No neighbor found at position {:?}", neighbor_pos);
            }
        }
    }

    fn setup_test_map(mut commands: Commands) {
        // Create a 3x3 tilemap
        let map_size = TilemapSize { x: 3, y: 3 };
        let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
        let grid_size = TilemapGridSize { x: 48.0, y: 48.0 };
        
        // Create the tilemap entity
        let tilemap_entity = commands.spawn_empty().id();
        let mut tile_storage = TileStorage::empty(map_size);
        
        // Spawn tiles
        for x in 0..map_size.x {
            for y in 0..map_size.y {
                let tile_pos = TilePos { x, y };
                let mut heat_cell = HeatCell::default();
                
                // Set center tile to hot, others to cool
                if x == 1 && y == 1 {
                    heat_cell.temperature.value = 100.0; // Center tile is hot
                } else {
                    heat_cell.temperature.value = 20.0; // Surrounding tiles are cool
                }
                
                // Spawn tile as a child of the tilemap
                commands.entity(tilemap_entity)
                    .with_children(|parent| {
                        tile_storage.set(&tile_pos, parent.spawn((
                            TileBundle {
                                position: tile_pos,
                                tilemap_id: TilemapId(tilemap_entity),
                                ..Default::default()
                            },
                            heat_cell,
                        )).id());
                    });
            }
        }
        
        // Add tilemap components
        commands.entity(tilemap_entity).insert(TilemapBundle {
            grid_size,
            tile_size,
            storage: tile_storage,
            size: map_size,
            ..Default::default()
        });
    }
    
    #[test]
    fn test_calculate_heat_transfer() {
        let cell1 = HeatCell {
            temperature: Temperature { value: 100.0 },
            conductivity: ThermalConductivity { value: 1.0 },
        };
        let cell2 = HeatCell {
            temperature: Temperature { value: 0.0 },
            conductivity: ThermalConductivity { value: 1.0 },
        };
        
        let dt = Duration::from_secs(1);
        let transfer_coefficient = 1.0;

        let (cell1_new_temp, cell2_new_temp) = calculate_heat_transfer(&cell1, &cell2, dt, transfer_coefficient);

        // With temp_diff = -100, conductivity = 1.0, dt = 1.0, transfer_coefficient = 1.0
        // heat_transfer = 1.0 * -100 * 1.0 * 1.0 * 0.5 = -50
        // So cell1 (hot) should lose 50°C and cell2 (cold) should gain 50°C
        assert_eq!(cell1_new_temp, -50.0);
        assert_eq!(cell2_new_temp, 50.0);
    }

    #[test]
    fn test_thermal_conduction() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(SimulationPlugin);
        app.insert_resource(MapSize(UVec2::new(3, 3)));
        // Add a very short simulation rate for testing
        app.insert_resource(SimulationRate { rate: Timer::new(Duration::from_millis(1), TimerMode::Repeating) });

        // Setup the test map
        app.add_systems(Startup, setup_test_map);
        app.update();

        // Run the thermal conduction system
        app.add_systems(Update, thermal_conduction);
        
        // Run for a few frames to let heat transfer occur
        for _ in 0..5 {
            app.update();
        }

        // Verify the results
        let mut query = app.world_mut().query::<(&HeatCell, &TilePos)>();
        let tiles: Vec<_> = query.iter(app.world()).collect();
        
        // Check center tile (should have cooled down)
        let center_tile = tiles.iter().find(|(_, pos)| pos.x == 1 && pos.y == 1).unwrap();
        assert!(center_tile.0.temperature.value < 100.0, "Center tile should have cooled down");
        
        // Check surrounding tiles (should have warmed up)
        for (heat_cell, pos) in tiles.iter() {
            if pos.x != 1 || pos.y != 1 {
                assert!(heat_cell.temperature.value > 20.0, "Surrounding tiles should have warmed up");
            }
        }
    }
}

