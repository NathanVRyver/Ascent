use bevy::prelude::*;

#[derive(Component)]
pub struct Flyer {
    pub mass: f32,
}

#[derive(Component)]
pub struct Wing {
//    pub span: f32,         // Wing span in meters
//    pub chord: f32,        // Average chord length in meters
    pub area: f32,         // Total wing area in m²
 //   pub aspect_ratio: f32, // Span² / Area
    pub angle_of_attack: f32, // Angle in radians
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
