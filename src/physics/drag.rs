use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct DragParams {
    pub air_density: f32,
    pub velocity: Vec3,
    pub wing_area: f32,
    pub drag_coefficient: f32,
    pub aspect_ratio: f32,
    pub efficiency_factor: f32,
}

pub fn calculate_parasitic_drag(params: &DragParams) -> Vec3 {
    let velocity_magnitude = params.velocity.length();
    if velocity_magnitude < 0.001 {
        return Vec3::ZERO;
    }
    
    let drag_magnitude = 0.5 * params.air_density * velocity_magnitude.powi(2) * params.wing_area * params.drag_coefficient;
    
    -params.velocity.normalize() * drag_magnitude
}

pub fn calculate_induced_drag(params: &DragParams, lift_coefficient: f32) -> Vec3 {
    let velocity_magnitude = params.velocity.length();
    if velocity_magnitude < 0.001 {
        return Vec3::ZERO;
    }
    
    let induced_drag_coefficient = lift_coefficient.powi(2) / (std::f32::consts::PI * params.aspect_ratio * params.efficiency_factor);
    
    let drag_magnitude = 0.5 * params.air_density * velocity_magnitude.powi(2) * params.wing_area * induced_drag_coefficient;
    
    -params.velocity.normalize() * drag_magnitude
}

pub fn calculate_total_drag(params: &DragParams, lift_coefficient: f32) -> Vec3 {
    calculate_parasitic_drag(params) + calculate_induced_drag(params, lift_coefficient)
}

pub fn calculate_drag_coefficient(angle_of_attack: f32, base_cd: f32) -> f32 {
    base_cd + 0.02 * angle_of_attack.abs()
}