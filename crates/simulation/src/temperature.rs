//! Tile heat conduction between neighbors.
//!
//! Numerical model, stability (CFL-style clamp), and parallel apply strategy:
//! see repository root `docs/THERMAL_SIMULATION.md`.

use std::{ops::{Add, Sub, Mul, Div}, time::Duration, ops::{Deref, DerefMut}};

use bevy::prelude::*;
use bevy::utils::Parallel;
use bevy_ecs_tilemap::tiles::TileStorage;
use common::{resources::MapSize, units::temperature::Kelvin};

use bevy_ecs_tilemap::prelude::TilePos;

use crate::SimulationRate;

#[derive(Component, Reflect, Debug, PartialEq, PartialOrd)]
#[reflect(Component)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Temperature {
    // Temperature in Kelvin (K)
    pub value: Kelvin,
}

impl Temperature {
    pub fn new(value: impl Into<Kelvin>) -> Self {
        Self { value: value.into() }
    }

    pub fn set_temperature(&mut self, value: Kelvin) {
        self.value = value;
    }

    pub fn get_temperature(&self) -> Kelvin {
        self.value
    }
}

impl AsRef<Kelvin> for Temperature {
    fn as_ref(&self) -> &Kelvin {
        &self.value
    }
}

impl AsMut<Kelvin> for Temperature {
    fn as_mut(&mut self) -> &mut Kelvin {
        &mut self.value
    }
}

impl Deref for Temperature {
    type Target = Kelvin;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Temperature {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl Add<Kelvin> for &Temperature {
    type Output = Kelvin;
    fn add(self, other: Kelvin) -> Self::Output {
        self.value + other
    }
}

impl Sub<Kelvin> for &Temperature {
    type Output = Kelvin;
    fn sub(self, other: Kelvin) -> Self::Output {
        self.value - other
    }
}

impl Add<Temperature> for &Temperature {
    type Output = Kelvin;
    fn add(self, other: Temperature) -> Self::Output {
        self.value + other.value
    }
}

impl Sub<Temperature> for &Temperature {
    type Output = Kelvin;
    fn sub(self, other: Temperature) -> Self::Output {
        self.value - other.value
    }
}

impl Mul<f32> for &Temperature {
    type Output = Kelvin;
    fn mul(self, other: f32) -> Self::Output {
        self.value * other
    }
}

impl Div<f32> for &Temperature {
    type Output = Kelvin;
    fn div(self, other: f32) -> Self::Output {
        self.value / other
    }
}

impl Default for Temperature {
    fn default() -> Self {
        Self { value: Kelvin::default() }
    }
}

#[derive(Component, Default, Reflect, Debug, PartialEq, PartialOrd)]
#[reflect(Component)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HeatCell {
    pub temperature: Temperature,
    pub conductivity: ThermalConductivity,
}

impl HeatCell {
    pub fn new(temperature: Temperature, conductivity: ThermalConductivity) -> Self {
        Self { temperature, conductivity }
    }

    pub fn with_temperature(mut self, temperature: Temperature) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_conductivity(mut self, conductivity: ThermalConductivity) -> Self {
        self.conductivity = conductivity;
        self
    }
}

#[derive(Component, Reflect, Debug, PartialEq, PartialOrd)]
#[reflect(Component)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ThermalConductivity(pub f32);

impl ThermalConductivity {
    pub fn new(value: f32) -> Self {
        Self(value)
    }
}

impl Default for ThermalConductivity {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Max fraction of the neighbor temperature gap a single edge may close in one step.
/// With square 4-neighbors, keeping this ≤ 0.25 keeps explicit relaxation stable (no overshoot / blow-up).
const MAX_EDGE_RELAXATION: f32 = 0.25;

/// Calculates heat transfer between two HeatCells over a time step
///
/// Uses explicit neighbor relaxation: ΔT₁ = λ(T₂−T₁), ΔT₂ = λ(T₁−T₂) with λ clamped so that
/// large `conductivity * dt` does not violate the CFL-like bound for a 4-connected grid.
///
/// # Arguments
/// * `cell1` - First heat cell
/// * `cell2` - Second heat cell
/// * `dt` - Time step in seconds
/// * `transfer_coefficient` - How much heat can transfer between cells (0.0 to 1.0)
///
/// # Returns
/// Tuple of (delta_temp1, delta_temp2) in Kelvin as raw `f32` for accumulation
//#[tracing::instrument(name = "Calculating heat transfer", skip(cell1, cell2, dt, transfer_coefficient))]
pub fn calculate_heat_transfer(
    cell1: &HeatCell,
    cell2: &HeatCell,
    dt: Duration,
    transfer_coefficient: f32,
) -> (f32, f32) {
    let avg_conductivity = (cell1.conductivity.0 + cell2.conductivity.0) * 0.5;
    let temp_diff = cell2.temperature.value - cell1.temperature.value;

    let lambda = (avg_conductivity * dt.as_secs_f32() * transfer_coefficient).min(MAX_EDGE_RELAXATION);
    let heat_transfer = lambda * temp_diff;

    debug!("Heat transfer: {}", heat_transfer);

    (heat_transfer.get_value(), -heat_transfer.get_value())
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
    mut tile_heat_query: Query<(&ChildOf, &mut HeatCell, &TilePos)>,
    layer_query: Query<&TileStorage>,
    mut simulation_rate: ResMut<SimulationRate>,
    size: Res<MapSize>,
    time: Res<Time>,
    mut thread_changes: Local<Parallel<Vec<(usize, f32)>>>,
) {
    simulation_rate.rate.tick(time.delta());
    let times_finished = simulation_rate.rate.times_finished_this_tick();
    let size = size.0;
    for _ in 0..times_finished {
        use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
        let map_size = bevy_ecs_tilemap::map::TilemapSize::from(size);
        let mut temp_accumulators = vec![0.0; (map_size.x * map_size.y) as usize];

        // First pass: collect temperature changes in parallel using thread-local buffers
        // (avoids Mutex contention; model and stability notes: docs/THERMAL_SIMULATION.md)
        let thread_changes_ref = &*thread_changes;
        tile_heat_query.par_iter().for_each(|(child_of, heat_cell, tile_pos)| {
            let mut local = thread_changes_ref.borrow_local_mut();
            if let Ok(tile_storage) = layer_query.get(child_of.parent()) {
                let current_index = tile_pos.to_index(&map_size) as usize;
                let neighbors = Neighbors::get_square_neighboring_positions(tile_pos, &map_size, false)
                    .entities(tile_storage);


                for neighbor in neighbors.iter() {
                    if let Ok((_child_of_neighbor, neighbor_cell, neighbor_pos)) = tile_heat_query.get(*neighbor) {
                        let neighbor_index = neighbor_pos.to_index(&map_size) as usize;
                        if neighbor_index > current_index {
                            let (delta1, delta2) = calculate_heat_transfer(heat_cell, neighbor_cell, simulation_rate.rate.duration(), 0.5);
                            local.push((current_index, delta1));
                            local.push((neighbor_index, delta2));
                        }
                    }
                }
            }
        });

        let mut all_changes = Vec::new();
        thread_changes.drain_into(&mut all_changes);
        for (index, change) in all_changes {
            temp_accumulators[index] += change;
        }

        // Second pass: apply all temperature changes in parallel
        tile_heat_query.par_iter_mut().for_each(|(_, mut heat_cell, tile_pos)| {
            let index = tile_pos.to_index(&map_size) as usize;
            let next = heat_cell.temperature.value.get_value() + temp_accumulators[index];
            heat_cell.temperature.set_temperature(Kelvin::new(next));
        });
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

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
        if let Some(child_of) = world.get::<ChildOf>(entity) {
            eprintln!("- ChildOf: {:?}", child_of);
        }
        if let Some(tilemap_id) = world.get::<TilemapId>(entity) {
            eprintln!("- TilemapId: {:?}", tilemap_id);
        }

        eprintln!("\nAll components:");
        if let Some(archetype) = world.archetypes().get(world.entity(entity).archetype().id()) {
            for component_id in archetype.components() {
                if let Some(component_info) = world.components().get_info(*component_id) {
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
                    heat_cell.temperature.value = Kelvin::new(100.0); // Center tile is hot
                } else {
                    heat_cell.temperature.value = Kelvin::new(0.0); // Surrounding tiles are cool
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
            temperature: Temperature { value: Kelvin::new(100.0) },
            conductivity: ThermalConductivity::default(),
        };
        let cell2 = HeatCell {
            temperature: Temperature { value: Kelvin::new(0.0) },
            conductivity: ThermalConductivity::default(),
        };
        
        let dt = Duration::from_secs(1);
        let transfer_coefficient = 1.0;

        let (cell1_new_temp, cell2_new_temp) = calculate_heat_transfer(&cell1, &cell2, dt, transfer_coefficient);

        // temp_diff = -100; λ = min(1.0, MAX_EDGE_RELAXATION) = 0.25 → heat_transfer = -25
        assert_eq!(cell1_new_temp, -25.0);
        assert_eq!(cell2_new_temp, 25.0);
    }

    #[test]
    fn test_thermal_conduction() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(SimulationPlugin);
        app.insert_resource(MapSize(UVec2::new(3, 3)));
        // Add a very short simulation rate for testing
        app.insert_resource(SimulationRate { rate: Timer::new(Duration::from_millis(1), TimerMode::Repeating) });
        app.finish();

        // Setup the test map
        app.add_systems(Startup, setup_test_map);
        app.update();

        // Run the thermal conduction system
        app.add_systems(FixedUpdate, thermal_conduction);
        
        // Run for a few frames to let heat transfer occur
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(2));
            app.update();
        }

        // Verify the results
        let mut query = app.world_mut().query::<(&HeatCell, &TilePos)>();
        let tiles: Vec<_> = query.iter(app.world()).collect();
        
        // Check center tile (should have cooled down)
        let center_tile = tiles.iter().find(|(_, pos)| pos.x == 1 && pos.y == 1).unwrap();
        assert!(center_tile.0.temperature.value.get_value() < 100.0, "Center tile should have cooled down");
        
        // Check surrounding tiles (should have warmed up)
        for (heat_cell, pos) in tiles.iter() {
            if pos.x != 1 || pos.y != 1 {
                assert!(heat_cell.temperature.value.get_value() > 0.0, "Surrounding tiles should have warmed up");
            }
        }
    }
}

