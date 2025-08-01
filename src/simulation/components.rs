use bevy::prelude::*;

#[derive(Component)]
pub struct Flyer {
    pub mass: f32,
}

#[derive(Component)]
pub struct Propulsion {
    pub thrust_power: f32,
    pub thrust_direction: Vec3,
    pub efficiency: f32,
    pub propeller_diameter: f32,
    pub throttle: f32,
}

#[derive(Component)]
pub struct Wing {
    pub span: f32,
    pub chord: f32,
    pub area: f32,
    pub aspect_ratio: f32,
    pub angle_of_attack: f32,
    pub lift_coefficient_base: f32,
    pub drag_coefficient_base: f32,
    pub efficiency_factor: f32,
}

#[derive(Component)]
pub struct FlightDynamics {
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub angular_velocity: Vec3,
    pub forces: Forces,
}

#[derive(Default, Clone)]
pub struct Forces {
    pub lift: Vec3,
    pub drag: Vec3,
    pub weight: Vec3,
    pub thrust: Vec3,
    pub total: Vec3,
}

#[derive(Component)]
pub struct FlightData {
    pub altitude: f32,
    pub airspeed: f32,
    pub vertical_speed: f32,
    pub flight_time: f32,
    pub distance_traveled: f32,
}

#[derive(Component)]
pub struct GroundPlane;

#[derive(Component)]
pub struct Atmosphere {
    pub air_density: f32,
    pub wind_velocity: Vec3,
    pub turbulence_intensity: f32,
    pub temperature: f32,
}

#[derive(Component)]
pub struct StallIndicator {
    pub is_stalled: bool,
    pub stall_severity: f32,
}
