use bevy::prelude::*;
use super::components::*;

#[derive(Component)]
pub struct FollowCamera {
    pub target_offset: Vec3,
    pub follow_speed: f32,
    pub look_ahead: f32,
    pub height_offset: f32,
    pub distance: f32,
}

impl Default for FollowCamera {
    fn default() -> Self {
        Self {
            target_offset: Vec3::ZERO,
            follow_speed: 2.0,
            look_ahead: 5.0,
            height_offset: 8.0,
            distance: 15.0,
        }
    }
}

pub fn setup_follow_camera(mut commands: Commands) {
    // Setup camera at a default position - it will find and follow the flyer later
    let camera_position = Vec3::new(-15.0, 8.0, 0.0);
    
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(camera_position)
            .looking_at(Vec3::new(0.0, 5.0, 0.0), Vec3::Y),
        FollowCamera::default(),
    ));
    
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.2, 0.0)),
    ));
}

pub fn update_follow_camera(
    time: Res<Time>,
    flyer_query: Query<(&Transform, &FlightDynamics), (With<Flyer>, Without<FollowCamera>)>,
    mut camera_query: Query<(&mut Transform, &mut FollowCamera), (With<Camera3d>, Without<Flyer>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let Ok((flyer_transform, flyer_dynamics)) = flyer_query.get_single() else { return; };
    let Ok((mut camera_transform, mut follow_camera)) = camera_query.get_single_mut() else { return; };
    
    // Manual camera controls override
    let mut manual_control = false;
    let rotation_speed = 2.0 * time.delta_secs();
    let zoom_speed = 20.0 * time.delta_secs();
    
    if keyboard.pressed(KeyCode::ArrowLeft) {
        camera_transform.rotate_around(
            flyer_transform.translation,
            Quat::from_rotation_y(rotation_speed),
        );
        manual_control = true;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        camera_transform.rotate_around(
            flyer_transform.translation,
            Quat::from_rotation_y(-rotation_speed),
        );
        manual_control = true;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        follow_camera.distance = (follow_camera.distance - zoom_speed).max(5.0);
        manual_control = true;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        follow_camera.distance = (follow_camera.distance + zoom_speed).min(50.0);
        manual_control = true;
    }
    
    // Auto-follow when not manually controlling
    if !manual_control {
        let target_position = flyer_transform.translation;
        let velocity_prediction = flyer_dynamics.velocity * follow_camera.look_ahead;
        let predicted_position = target_position + velocity_prediction;
        
        // Calculate ideal camera position
        let camera_offset = Vec3::new(
            -follow_camera.distance * 0.7,
            follow_camera.height_offset,
            follow_camera.distance * 0.3,
        );
        
        let ideal_camera_pos = predicted_position + camera_offset;
        
        // Smooth interpolation
        let current_pos = camera_transform.translation;
        let new_pos = current_pos.lerp(ideal_camera_pos, follow_camera.follow_speed * time.delta_secs());
        
        camera_transform.translation = new_pos;
        camera_transform.look_at(predicted_position, Vec3::Y);
    }
    
    // Always keep camera looking at the flyer when manually controlling
    if manual_control {
        let to_target = (flyer_transform.translation - camera_transform.translation).normalize();
        camera_transform.translation = flyer_transform.translation - to_target * follow_camera.distance;
        camera_transform.look_at(flyer_transform.translation, Vec3::Y);
    }
}

pub fn reset_camera_on_flyer_reset(
    flyer_query: Query<&Transform, (With<Flyer>, Changed<Transform>)>,
    mut camera_query: Query<&mut Transform, (With<FollowCamera>, Without<Flyer>)>,
) {
    if let (Ok(flyer_transform), Ok(mut camera_transform)) = (flyer_query.get_single(), camera_query.get_single_mut()) {
        // Check if flyer was reset (back to starting position)
        if flyer_transform.translation.distance(Vec3::new(0.0, 5.0, 0.0)) < 1.0 {
            let camera_position = flyer_transform.translation + Vec3::new(-15.0, 8.0, 0.0);
            camera_transform.translation = camera_position;
            camera_transform.look_at(flyer_transform.translation, Vec3::Y);
        }
    }
}