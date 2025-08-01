
#[derive(Debug, Clone, Copy)]
pub struct StallParams {
    pub angle_of_attack: f32,
    pub critical_angle: f32,
    pub post_stall_drop: f32,
    pub stall_progression_rate: f32,
}

impl Default for StallParams {
    fn default() -> Self {
        Self {
            angle_of_attack: 0.0,
            critical_angle: 15.0_f32.to_radians(),
            post_stall_drop: 0.5,
            stall_progression_rate: 2.0,
        }
    }
}

pub fn calculate_stall_factor(params: &StallParams) -> f32 {
    if params.angle_of_attack.abs() <= params.critical_angle {
        1.0
    } else {
        let over_critical = (params.angle_of_attack.abs() - params.critical_angle).max(0.0);
        let stall_severity = (over_critical * params.stall_progression_rate).min(1.0);
        
        1.0 - stall_severity * (1.0 - params.post_stall_drop)
    }
}

pub fn calculate_lift_coefficient_with_stall(
    base_cl: f32,
    angle_of_attack: f32,
    stall_params: &StallParams,
) -> f32 {
    let linear_cl = base_cl * angle_of_attack.sin();
    let stall_factor = calculate_stall_factor(&StallParams {
        angle_of_attack,
        ..stall_params.clone()
    });
    
    linear_cl * stall_factor
}

pub fn calculate_drag_coefficient_stalled(
    base_cd: f32,
    angle_of_attack: f32,
    stall_params: &StallParams,
) -> f32 {
    if angle_of_attack.abs() <= stall_params.critical_angle {
        base_cd
    } else {
        let stall_severity = ((angle_of_attack.abs() - stall_params.critical_angle) / stall_params.critical_angle).min(1.0);
        base_cd * (1.0 + 3.0 * stall_severity)
    }
}