use bevy::prelude::*;

pub struct GroundEffectParams {
    pub altitude: f32,
    pub wing_span: f32,
    pub wing_chord: f32,
}

pub fn calculate_ground_effect_factor(params: &GroundEffectParams) -> f32 {
    let h_over_b = params.altitude / params.wing_span;
    
    if h_over_b > 1.0 {
        1.0
    } else {
        let reduction_factor = (16.0 * h_over_b).powi(2) / (1.0 + (16.0 * h_over_b).powi(2));
        1.0 / (1.0 - reduction_factor)
    }
}

pub fn apply_ground_effect_to_lift(base_lift: Vec3, ground_effect_factor: f32) -> Vec3 {
    base_lift * ground_effect_factor
}

pub fn calculate_ground_effect_drag_reduction(params: &GroundEffectParams) -> f32 {
    let h_over_b = params.altitude / params.wing_span;
    
    if h_over_b > 1.0 {
        1.0
    } else {
        1.0 - 0.48 * (-2.0 * h_over_b).exp()
    }
}