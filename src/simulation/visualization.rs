use bevy::prelude::*;
use super::components::*;

#[derive(Resource)]
pub struct VisualizationSettings {
    pub show_forces: bool,
    pub show_velocity: bool,
    pub show_trajectory: bool,
    pub show_wind: bool,
    pub force_scale: f32,
    pub show_labels: bool,
}

impl Default for VisualizationSettings {
    fn default() -> Self {
        Self {
            show_forces: true,
            show_velocity: true,
            show_trajectory: false,
            show_wind: false,
            force_scale: 0.01,
            show_labels: true,
        }
    }
}

#[derive(Component)]
pub struct TrajectoryTrail {
    pub points: Vec<Vec3>,
    pub max_points: usize,
}

impl Default for TrajectoryTrail {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            max_points: 500,
        }
    }
}

pub fn visualize_forces(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &FlightDynamics, Option<&TrajectoryTrail>), With<Flyer>>,
    settings: Res<VisualizationSettings>,
    atmosphere_query: Query<&Atmosphere>,
) {
    for (transform, dynamics, trail) in query.iter() {
        let pos = transform.translation;
        
        if settings.show_forces {
            draw_force_vectors(&mut gizmos, pos, dynamics, settings.force_scale, settings.show_labels);
        }
        
        if settings.show_velocity {
            draw_velocity_vector(&mut gizmos, pos, dynamics.velocity);
        }
        
        if settings.show_trajectory {
            if let Some(trail) = trail {
                draw_trajectory_trail(&mut gizmos, trail);
            }
        }
        
        draw_orientation_axes(&mut gizmos, transform);
    }
    
    if settings.show_wind {
        if let Ok(atmosphere) = atmosphere_query.single() {
            draw_wind_indicators(&mut gizmos, atmosphere.wind_velocity);
        }
    }
}

fn draw_force_vectors(
    gizmos: &mut Gizmos,
    pos: Vec3,
    dynamics: &FlightDynamics,
    scale: f32,
    show_labels: bool,
) {
    let forces = [
        (dynamics.forces.lift, Color::srgb(0.0, 1.0, 0.0), "Lift"),
        (dynamics.forces.weight, Color::srgb(1.0, 0.0, 0.0), "Weight"),
        (dynamics.forces.drag, Color::srgb(1.0, 0.5, 0.0), "Drag"),
        (dynamics.forces.thrust, Color::srgb(0.0, 0.5, 1.0), "Thrust"),
    ];
    
    for (force, color, label) in forces {
        if force.length() > 0.1 {
            let end_pos = pos + force * scale;
            
            gizmos.line(pos, end_pos, color);
            
            draw_arrow_head(gizmos, pos, end_pos, color, force.length() * scale * 0.1);
            
            if show_labels {
                let label_pos = end_pos + Vec3::new(0.2, 0.2, 0.0);
                draw_3d_text(gizmos, label_pos, label, color);
            }
        }
    }
    
    if dynamics.forces.total.length() > 0.1 {
        let end_pos = pos + dynamics.forces.total * scale;
        let color = Color::WHITE;
        
        gizmos.line(pos, end_pos, color.with_alpha(0.8));
        draw_arrow_head(gizmos, pos, end_pos, color, dynamics.forces.total.length() * scale * 0.15);
        
        if show_labels {
            draw_3d_text(gizmos, end_pos + Vec3::new(0.3, 0.3, 0.0), "Net Force", color);
        }
    }
}

fn draw_velocity_vector(gizmos: &mut Gizmos, pos: Vec3, velocity: Vec3) {
    if velocity.length() > 0.1 {
        let color = Color::srgb(0.0, 1.0, 1.0);
        let end_pos = pos + velocity * 0.2;
        
        gizmos.line(pos, end_pos, color);
        draw_arrow_head(gizmos, pos, end_pos, color, 0.3);
        
        let speed_text = format!("{:.1} m/s", velocity.length());
        draw_3d_text(gizmos, end_pos + Vec3::new(0.2, -0.2, 0.0), &speed_text, color);
    }
}

fn draw_arrow_head(gizmos: &mut Gizmos, start: Vec3, end: Vec3, color: Color, size: f32) {
    let direction = (end - start).normalize();
    let perpendicular = if direction.abs_diff_eq(Vec3::Y, 0.1) {
        direction.cross(Vec3::X)
    } else {
        direction.cross(Vec3::Y)
    }.normalize();
    
    let arrow_base = end - direction * size;
    let arrow_point1 = arrow_base + perpendicular * size * 0.5;
    let arrow_point2 = arrow_base - perpendicular * size * 0.5;
    
    gizmos.line(end, arrow_point1, color);
    gizmos.line(end, arrow_point2, color);
    gizmos.line(arrow_point1, arrow_point2, color);
}

fn draw_orientation_axes(gizmos: &mut Gizmos, transform: &Transform) {
    let pos = transform.translation;
    let scale = 1.0;
    
    let forward = transform.forward() * scale;
    let right = transform.right() * scale;
    let up = transform.up() * scale;
    
    gizmos.line(pos, pos + forward, Color::srgb(0.0, 0.0, 1.0).with_alpha(0.5));
    gizmos.line(pos, pos + right, Color::srgb(1.0, 0.0, 0.0).with_alpha(0.5));
    gizmos.line(pos, pos + up, Color::srgb(0.0, 1.0, 0.0).with_alpha(0.5));
}

fn draw_trajectory_trail(gizmos: &mut Gizmos, trail: &TrajectoryTrail) {
    if trail.points.len() < 2 {
        return;
    }
    
    for i in 1..trail.points.len() {
        let alpha = i as f32 / trail.points.len() as f32 * 0.5;
        let color = Color::srgb(1.0, 1.0, 0.0).with_alpha(alpha);
        
        gizmos.line(trail.points[i - 1], trail.points[i], color);
    }
}

fn draw_wind_indicators(gizmos: &mut Gizmos, wind_velocity: Vec3) {
    let grid_size = 20;
    let spacing = 5.0;
    let y_levels = [0.0, 5.0, 10.0];
    
    for y in y_levels {
        for x in -grid_size..=grid_size {
            for z in -grid_size..=grid_size {
                if x % 4 == 0 && z % 4 == 0 {
                    let pos = Vec3::new(x as f32 * spacing, y, z as f32 * spacing);
                    let wind_end = pos + wind_velocity * 0.2;
                    
                    let alpha = 0.3 - (pos.length() / 100.0).min(0.25);
                    let color = Color::srgb(0.5, 0.5, 1.0).with_alpha(alpha);
                    
                    gizmos.line(pos, wind_end, color);
                }
            }
        }
    }
}

fn draw_3d_text(gizmos: &mut Gizmos, _pos: Vec3, _text: &str, _color: Color) {
    // Text rendering in 3D space would require additional implementation
    // For now, we'll skip this as it requires more complex text rendering
}

pub fn update_trajectory_trail(
    mut query: Query<(&Transform, &mut TrajectoryTrail), With<Flyer>>,
    time: Res<Time>,
    mut last_update: Local<f32>,
) {
    let current_time = time.elapsed_secs();
    
    if current_time - *last_update < 0.1 {
        return;
    }
    
    *last_update = current_time;
    
    for (transform, mut trail) in query.iter_mut() {
        trail.points.push(transform.translation);
        
        if trail.points.len() > trail.max_points {
            trail.points.remove(0);
        }
    }
}

pub fn toggle_visualization_settings(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<VisualizationSettings>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        settings.show_forces = !settings.show_forces;
    }
    if keyboard.just_pressed(KeyCode::F2) {
        settings.show_velocity = !settings.show_velocity;
    }
    if keyboard.just_pressed(KeyCode::F3) {
        settings.show_trajectory = !settings.show_trajectory;
    }
    if keyboard.just_pressed(KeyCode::F4) {
        settings.show_wind = !settings.show_wind;
    }
    if keyboard.just_pressed(KeyCode::F5) {
        settings.show_labels = !settings.show_labels;
    }
}