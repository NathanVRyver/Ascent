use bevy::prelude::*;
use super::components::*;

#[derive(Component)]
pub struct POVCamera {
    pub head_offset: Vec3,
    pub look_sensitivity: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for POVCamera {
    fn default() -> Self {
        Self {
            head_offset: Vec3::new(0.0, 0.6, 0.1), // Position at head level, slightly forward
            look_sensitivity: 2.0,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

pub fn setup_pov_camera(mut commands: Commands) {
    // POV camera will be positioned relative to the flyer
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0.0, 5.6, 0.1)) // Start at head level of flyer
            .looking_at(Vec3::new(0.0, 5.6, 1.0), Vec3::Y),
        POVCamera::default(),
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

pub fn update_pov_camera(
    time: Res<Time>,
    flyer_query: Query<&Transform, (With<Flyer>, Without<POVCamera>)>,
    mut camera_query: Query<(&mut Transform, &mut POVCamera), (With<Camera3d>, Without<Flyer>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let Ok(flyer_transform) = flyer_query.single() else { return; };
    let Ok((mut camera_transform, mut pov_camera)) = camera_query.single_mut() else { return; };
    
    let rotation_speed = pov_camera.look_sensitivity * time.delta_secs();
    
    // Handle look controls
    if keyboard.pressed(KeyCode::ArrowLeft) {
        pov_camera.yaw += rotation_speed;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        pov_camera.yaw -= rotation_speed;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        pov_camera.pitch = (pov_camera.pitch + rotation_speed).min(1.5); // Limit pitch
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        pov_camera.pitch = (pov_camera.pitch - rotation_speed).max(-1.5); // Limit pitch
    }
    
    // Position camera at the flyer's head
    let head_position = flyer_transform.translation + 
        flyer_transform.rotation * pov_camera.head_offset;
    
    camera_transform.translation = head_position;
    
    // Apply pitch and yaw rotations relative to the flyer's orientation
    let base_rotation = flyer_transform.rotation;
    let pitch_rotation = Quat::from_rotation_x(pov_camera.pitch);
    let yaw_rotation = Quat::from_rotation_y(pov_camera.yaw);
    
    camera_transform.rotation = base_rotation * yaw_rotation * pitch_rotation;
}

pub fn reset_pov_camera_on_flyer_reset(
    flyer_query: Query<&Transform, (With<Flyer>, Changed<Transform>)>,
    mut camera_query: Query<(&mut Transform, &mut POVCamera), (With<POVCamera>, Without<Flyer>)>,
) {
    if let (Ok(flyer_transform), Ok((mut camera_transform, mut pov_camera))) = (flyer_query.single(), camera_query.single_mut()) {
        // Check if flyer was reset (back to starting position)
        if flyer_transform.translation.distance(Vec3::new(0.0, 5.0, 0.0)) < 1.0 {
            // Reset POV camera to head position
            let head_position = flyer_transform.translation + 
                flyer_transform.rotation * pov_camera.head_offset;
            camera_transform.translation = head_position;
            camera_transform.rotation = flyer_transform.rotation;
            
            // Reset pitch and yaw
            pov_camera.pitch = 0.0;
            pov_camera.yaw = 0.0;
        }
    }
}