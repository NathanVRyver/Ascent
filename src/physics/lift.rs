pub struct LiftParams {
    pub air_density: f64,
    pub air_speed: f64,
    pub wing_area: f64,
    pub lift_coefficient: f64,
}

pub fn calculate_lift(params: &LiftParams) -> f64 {
    0.5 * params.air_density * params.air_speed.powi(2) * params.wing_area * params.lift_coefficient
}
