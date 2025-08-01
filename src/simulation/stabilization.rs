use bevy::prelude::*;
use super::components::*;

#[derive(Component)]
pub struct FlightStabilizer {
    pub max_velocity: f32,
    pub max_angular_velocity: f32,
    pub stability_damping: f32,
    pub auto_level_strength: f32,
}

impl Default for FlightStabilizer {
    fn default() -> Self {
        Self {
            max_velocity: 50.0, // m/s
            max_angular_velocity: 2.0, // rad/s
            stability_damping: 0.98,
            auto_level_strength: 0.5,
        }
    }
}

pub fn apply_flight_stabilization(
    mut query: Query<(&mut FlightDynamics, &FlightStabilizer, &mut Transform)>,
    time: Res<Time>,
) {
    for (mut dynamics, stabilizer, mut transform) in query.iter_mut() {
        let dt = time.delta_secs();
        
        // Velocity limiting
        let velocity_magnitude = dynamics.velocity.length();
        if velocity_magnitude > stabilizer.max_velocity {
            dynamics.velocity = dynamics.velocity.normalize() * stabilizer.max_velocity;
        }
        
        // Angular velocity limiting  
        let angular_magnitude = dynamics.angular_velocity.length();
        if angular_magnitude > stabilizer.max_angular_velocity {
            dynamics.angular_velocity = dynamics.angular_velocity.normalize() * stabilizer.max_angular_velocity;
        }
        
        // Apply stability damping
        dynamics.velocity *= stabilizer.stability_damping;
        dynamics.angular_velocity *= stabilizer.stability_damping;
        
        // Auto-leveling - gradually return to level flight
        let current_rotation = transform.rotation;
        let level_rotation = Quat::from_rotation_y(current_rotation.to_euler(EulerRot::YXZ).0);
        
        transform.rotation = current_rotation.slerp(level_rotation, stabilizer.auto_level_strength * dt);
        
        // Prevent excessive diving or climbing
        let forward = transform.forward();
        let pitch = forward.y.asin();
        let max_pitch = 60.0_f32.to_radians();
        
        if pitch.abs() > max_pitch {
            let clamped_pitch = pitch.clamp(-max_pitch, max_pitch);
            let yaw = current_rotation.to_euler(EulerRot::YXZ).0;
            transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, clamped_pitch, 0.0);
        }
    }
}

pub fn add_ground_avoidance(
    mut query: Query<(&mut FlightDynamics, &Transform), With<Flyer>>,
    time: Res<Time>,
) {
    for (mut dynamics, transform) in query.iter_mut() {
        let altitude = transform.translation.y;
        let min_safe_altitude = 2.0;
        
        if altitude < min_safe_altitude && dynamics.velocity.y < 0.0 {
            // Add upward force when too close to ground and descending
            let avoidance_strength = (min_safe_altitude - altitude) / min_safe_altitude;
            let upward_force = Vec3::Y * avoidance_strength * 500.0 * time.delta_secs();
            
            dynamics.velocity += upward_force;
            
            // Reduce downward velocity
            if dynamics.velocity.y < -1.0 {
                dynamics.velocity.y *= 0.8;
            }
        }
    }
}