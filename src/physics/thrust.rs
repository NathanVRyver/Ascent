use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct ThrustParams {
    pub thrust_power: f32,
    pub thrust_direction: Vec3,
    pub efficiency: f32,
    pub propeller_diameter: f32,
    pub air_density: f32,
    pub velocity: Vec3,
}

pub fn calculate_thrust_force(params: &ThrustParams) -> Vec3 {
    let static_thrust = params.thrust_power * params.efficiency;
    
    let velocity_in_thrust_direction = params.velocity.dot(params.thrust_direction);
    let velocity_factor = 1.0 - (velocity_in_thrust_direction / 50.0).clamp(0.0, 0.8);
    
    params.thrust_direction * static_thrust * velocity_factor
}

pub fn calculate_propeller_efficiency(advance_ratio: f32) -> f32 {
    if advance_ratio < 0.0 || advance_ratio > 2.0 {
        0.0
    } else {
        let peak_ratio = 0.8;
        let width = 0.5;
        let efficiency = (-(advance_ratio - peak_ratio).powi(2) / (2.0 * width.powi(2))).exp();
        efficiency * 0.85
    }
}

pub fn calculate_advance_ratio(velocity: f32, rpm: f32, diameter: f32) -> f32 {
    if rpm <= 0.0 || diameter <= 0.0 {
        0.0
    } else {
        velocity / (rpm / 60.0 * diameter)
    }
}