use bevy::prelude::*;

#[derive(Resource)]
pub struct SimulationParams {
    pub gravity: f32,
    pub air_density: f32,
    pub wind_velocity: Vec3,
    pub simulation_speed: f32,
    pub is_running: bool,
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            gravity: 9.81,
            air_density: 1.225,  // kg/m³ at sea level
            wind_velocity: Vec3::ZERO,
            simulation_speed: 1.0,
            is_running: false,
        }
    }
}