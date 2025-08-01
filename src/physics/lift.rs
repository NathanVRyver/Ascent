use bevy::prelude::*;

pub struct LiftParams {
    pub air_density: f32,
    pub velocity: Vec3,
    pub wing_area: f32,
    pub wing_span: f32,
    pub wing_chord: f32,
    pub angle_of_attack: f32,
}

pub fn calculate_lift_coefficient(angle_of_attack: f32, base_cl: f32) -> f32 {
    base_cl * angle_of_attack.sin()
}

pub fn calculate_lift_force(params: &LiftParams, lift_coefficient: f32) -> Vec3 {
    let velocity_magnitude = params.velocity.length();
    if velocity_magnitude < 0.001 {
        return Vec3::ZERO;
    }
    
    let lift_magnitude = 0.5 * params.air_density * velocity_magnitude.powi(2) * params.wing_area * lift_coefficient;
    
    let velocity_normalized = params.velocity.normalize();
    let world_up = Vec3::Y;
    let lift_direction = velocity_normalized.cross(world_up.cross(velocity_normalized)).normalize();
    
    lift_direction * lift_magnitude
}

pub fn calculate_aspect_ratio(wing_span: f32, wing_area: f32) -> f32 {
    wing_span.powi(2) / wing_area
}
