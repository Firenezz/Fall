//! Tile heat conduction between neighbors.
//!
//! Numerical model and stability (CFL-style clamp): see `docs/THERMAL_SIMULATION.md`.
//! Conduction uses row-major scratch buffers (gather → stencil → scatter); [`HeatCell`] stays on tiles.

use std::{ops::{Add, Sub, Mul, Div}, time::Duration, ops::{Deref, DerefMut}};

use bevy::{math::ops::sqrt, prelude::*};
use bevy_ecs_tilemap::{map::TilemapSize, tiles::TileStorage};
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
/// With square 4-neighbors, sum of incident edge λ should stay ≲ 1; see `docs/THERMAL_SIMULATION.md`.
const MAX_EDGE_RELAXATION: f32 = 0.15;

/// Passed from `thermal_conduction` into edge relaxation (see `docs/THERMAL_SIMULATION.md`).
const TRANSFER_COEFFICIENT: f32 = 0.5;

/// Reused row-major workspace; [`HeatCell`] on tiles remains authoritative between steps.
#[derive(Resource, Default)]
struct ThermalScratch {
    temperatures: Vec<f32>,
    conductivities: Vec<f32>,
    deltas: Vec<f32>,
    active: Vec<bool>,
}

impl ThermalScratch {
    fn ensure_size(&mut self, size: TilemapSize) {
        let n = size.count() as usize;
        self.temperatures.resize(n, 0.0);
        self.conductivities.resize(n, 1.0);
        self.deltas.resize(n, 0.0);
        self.active.resize(n, false);
    }
}

/// Scalar edge relaxation: ΔT₁ = λ(T₂−T₁); uses geometric mean for k_eff (see `docs/THERMAL_SIMULATION.md`).
fn edge_delta(t1: f32, t2: f32, k1: f32, k2: f32, dt_secs: f32, transfer_coefficient: f32) -> f32 {
    if (t2 - t1).abs() < 1.0 {
        return 0.0;
    }
    let lambda = (sqrt(k1 * k2) * dt_secs * transfer_coefficient).min(MAX_EDGE_RELAXATION);
    lambda * (t2 - t1)
}

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
    if temp_diff > Kelvin::new(20.0) {
        warn!("Temperature difference is too large: {:?}", temp_diff);
    }

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
            .init_resource::<ThermalScratch>()
            .add_systems(Update, thermal_conduction);
    }
}

fn thermal_conduction(
    mut heat_cells: Query<&mut HeatCell>,
    layer_query: Query<&TileStorage>,
    mut scratch: ResMut<ThermalScratch>,
    mut simulation_rate: ResMut<SimulationRate>,
    size: Res<MapSize>,
    time: Res<Time>,
) {
    simulation_rate.rate.tick(time.delta());
    let times_finished = simulation_rate.rate.times_finished_this_tick();
    let map_size = TilemapSize::from(size.0);
    scratch.ensure_size(map_size);
    let dt_secs = simulation_rate.rate.duration().as_secs_f32();

    for _ in 0..times_finished {
        for tile_storage in layer_query.iter() {
            conduction_substep(
                tile_storage,
                &map_size,
                &mut scratch,
                &mut heat_cells,
                dt_secs,
            );
        }
    }
}

/// Gather → stencil (Jacobi on scratch) → scatter for one layer.
fn conduction_substep(
    tile_storage: &TileStorage,
    map_size: &TilemapSize,
    scratch: &mut ThermalScratch,
    heat_cells: &mut Query<&mut HeatCell>,
    dt_secs: f32,
) {
    let w = map_size.x;
    let h = map_size.y;

    scratch.active.fill(false);

    for y in 0..h {
        for x in 0..w {
            let pos = TilePos::new(x, y);
            let i = pos.to_index(map_size);
            if let Some(entity) = tile_storage.get(&pos) {
                if let Ok(cell) = heat_cells.get(entity) {
                    scratch.temperatures[i] = cell.temperature.value.get_value();
                    scratch.conductivities[i] = cell.conductivity.0;
                    scratch.active[i] = true;
                }
            }
        }
    }

    scratch.deltas.fill(0.0);

    for y in 0..h {
        for x in 0..w {
            let i = TilePos::new(x, y).to_index(map_size);
            if !scratch.active[i] {
                continue;
            }
            let t = scratch.temperatures[i];
            let k = scratch.conductivities[i];

            std::iter::once(
                match (x as u32, y as u32) {
                    (0, 0) => {
                        [
                            None,
                            Some(TilePos::new(0 + 1, 0)),
                            None,
                            Some(TilePos::new(0, 0 + 1)),
                        ]
                    }
                    (0, y) => {
                        [
                            Some(TilePos::new(0 + 1, y)),
                            None,
                            Some(TilePos::new(0, y + 1)),
                            Some(TilePos::new(0, y - 1)),
                        ]
                    }
                    (x, 0) => {
                        [
                            Some(TilePos::new(x - 1, 0)),
                            Some(TilePos::new(x + 1, 0)),
                            None,
                            Some(TilePos::new(x, 0 + 1)),
                        ]
                    }
                    (x, y) => {
                        [
                            Some(TilePos::new(x - 1, y)),
                            Some(TilePos::new(x + 1, y)),
                            Some(TilePos::new(x, y - 1)),
                            Some(TilePos::new(x, y + 1)),
                        ]
                    }
                }
            ).flatten().filter_map(|pos| pos).filter(|pos| pos.within_map_bounds(map_size)).for_each(|pos| {
                let j = pos.to_index(map_size);
                if scratch.active[j] {
                    scratch.deltas[i] += edge_delta(
                        t,
                        scratch.temperatures[j],
                        k,
                        scratch.conductivities[j],
                        dt_secs,
                        TRANSFER_COEFFICIENT
                    );
                }
            });
        }
    }

    for i in 0..scratch.temperatures.len() {
        if scratch.active[i] {
            scratch.temperatures[i] += scratch.deltas[i];
        }
    }

    for y in 0..h {
        for x in 0..w {
            let pos = TilePos::new(x, y);
            let i = pos.to_index(map_size);
            if !scratch.active[i] {
                continue;
            }
            if let Some(entity) = tile_storage.get(&pos) {
                if let Ok(mut cell) = heat_cells.get_mut(entity) {
                    cell.temperature
                        .set_temperature(Kelvin::new(scratch.temperatures[i]));
                }
            }
        }
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

