use bevy::prelude::*;

#[derive(Component)]
pub struct FlappingWing {
    pub frequency: f32,
    pub amplitude: f32,
    pub phase_offset: f32,
    pub power_stroke_ratio: f32,
    pub twist_amplitude: f32,
    pub is_active: bool,
}

impl Default for FlappingWing {
    fn default() -> Self {
        Self {
            frequency: 2.0,
            amplitude: 45.0_f32.to_radians(),
            phase_offset: 0.0,
            power_stroke_ratio: 0.3,
            twist_amplitude: 20.0_f32.to_radians(),
            is_active: false,
        }
    }
}

#[derive(Component)]
pub struct WingJoint {
    pub joint_type: WingJointType,
    pub rest_angle: f32,
    pub current_angle: f32,
    pub max_angle: f32,
    pub min_angle: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WingJointType {
    Shoulder,
    Elbow,
    Wrist,
}

pub fn update_flapping_animation(
    time: Res<Time>,
    mut query: Query<(&FlappingWing, &mut Transform, &Children)>,
    mut joint_query: Query<(&WingJoint, &mut Transform), Without<FlappingWing>>,
) {
    for (flapping, mut transform, children) in query.iter_mut() {
        if !flapping.is_active {
            continue;
        }
        
        let elapsed = time.elapsed_secs();
        let phase = elapsed * flapping.frequency * 2.0 * std::f32::consts::PI + flapping.phase_offset;
        
        let normalized_phase = (phase % (2.0 * std::f32::consts::PI)) / (2.0 * std::f32::consts::PI);
        let is_power_stroke = normalized_phase < flapping.power_stroke_ratio;
        
        let stroke_progress = if is_power_stroke {
            normalized_phase / flapping.power_stroke_ratio
        } else {
            (normalized_phase - flapping.power_stroke_ratio) / (1.0 - flapping.power_stroke_ratio)
        };
        
        let base_angle = if is_power_stroke {
            stroke_progress * 2.0 - 1.0
        } else {
            1.0 - stroke_progress * 2.0
        };
        
        let flap_angle = base_angle * flapping.amplitude;
        let twist_angle = base_angle.abs() * flapping.twist_amplitude * if is_power_stroke { 1.0 } else { -0.5 };
        
        transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            flap_angle,
            0.0,
            twist_angle,
        );
        
        for child in children.iter() {
            if let Ok((joint, mut joint_transform)) = joint_query.get_mut(child) {
                match joint.joint_type {
                    WingJointType::Shoulder => {
                        joint_transform.rotation = Quat::from_rotation_x(flap_angle * 0.3);
                    }
                    WingJointType::Elbow => {
                        let elbow_angle = if is_power_stroke {
                            -stroke_progress * 0.5
                        } else {
                            -0.5 + stroke_progress * 0.5
                        };
                        joint_transform.rotation = Quat::from_rotation_x(elbow_angle);
                    }
                    WingJointType::Wrist => {
                        let wrist_angle = twist_angle * 0.5;
                        joint_transform.rotation = Quat::from_rotation_z(wrist_angle);
                    }
                }
            }
        }
    }
}

pub fn calculate_flapping_thrust(
    flapping: &FlappingWing,
    wing_area: f32,
    air_density: f32,
    time: f32,
) -> Vec3 {
    if !flapping.is_active {
        return Vec3::ZERO;
    }
    
    let phase = time * flapping.frequency * 2.0 * std::f32::consts::PI + flapping.phase_offset;
    let velocity = flapping.amplitude * flapping.frequency * 2.0 * std::f32::consts::PI * phase.cos();
    
    let normalized_phase = (phase % (2.0 * std::f32::consts::PI)) / (2.0 * std::f32::consts::PI);
    let is_power_stroke = normalized_phase < flapping.power_stroke_ratio;
    
    if is_power_stroke {
        let thrust_coefficient = 0.8;
        let thrust_magnitude = 0.5 * air_density * velocity.abs().powi(2) * wing_area * thrust_coefficient;
        Vec3::new(thrust_magnitude * 0.7, thrust_magnitude * 0.3, 0.0)
    } else {
        Vec3::ZERO
    }
}

pub fn toggle_flapping(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut FlappingWing>,
) {
    if keyboard.just_pressed(KeyCode::KeyF) {
        for mut flapping in query.iter_mut() {
            flapping.is_active = !flapping.is_active;
        }
    }
}