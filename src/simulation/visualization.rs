use bevy::prelude::*;
use super::components::*;

pub fn visualize_forces(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &FlightDynamics), With<Flyer>>,
) {
    for (transform, dynamics) in query.iter() {
        let pos = transform.translation;
        let scale = 0.01; // Scale factor for force visualization
        
        // Lift (green, pointing up)
        if dynamics.forces.lift.length() > 0.1 {
            gizmos.line(
                pos,
                pos + dynamics.forces.lift * scale,
                Color::srgb(0.0, 1.0, 0.0),
            );
            gizmos.sphere(
                pos + dynamics.forces.lift * scale,
                0.1,
                Color::srgb(0.0, 1.0, 0.0),
            );
        }
        
        // Weight (red, pointing down)
        if dynamics.forces.weight.length() > 0.1 {
            gizmos.line(
                pos,
                pos + dynamics.forces.weight * scale,
                Color::srgb(1.0, 0.0, 0.0),
            );
            gizmos.sphere(
                pos + dynamics.forces.weight * scale,
                0.1,
                Color::srgb(1.0, 0.0, 0.0),
            );
        }
        
        // Drag (yellow, opposing velocity)
        if dynamics.forces.drag.length() > 0.1 {
            gizmos.line(
                pos,
                pos + dynamics.forces.drag * scale,
                Color::srgb(1.0, 1.0, 0.0),
            );
            gizmos.sphere(
                pos + dynamics.forces.drag * scale,
                0.1,
                Color::srgb(1.0, 1.0, 0.0),
            );
        }
        
        // Thrust (blue, forward)
        if dynamics.forces.thrust.length() > 0.1 {
            gizmos.line(
                pos,
                pos + dynamics.forces.thrust * scale,
                Color::srgb(0.0, 0.0, 1.0),
            );
            gizmos.sphere(
                pos + dynamics.forces.thrust * scale,
                0.1,
                Color::srgb(0.0, 0.0, 1.0),
            );
        }
        
        // Total force (white)
        if dynamics.forces.total.length() > 0.1 {
            gizmos.line(
                pos,
                pos + dynamics.forces.total * scale,
                Color::WHITE,
            );
        }
        
        // Velocity vector (cyan)
        if dynamics.velocity.length() > 0.1 {
            gizmos.line(
                pos,
                pos + dynamics.velocity * 0.2,
                Color::srgb(0.0, 1.0, 1.0),
            );
        }
    }
}