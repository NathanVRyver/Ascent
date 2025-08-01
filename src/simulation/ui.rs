use bevy::prelude::*;
use super::components::*;
use super::resources::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (
                update_ui_text,
                handle_parameter_input,
            ));
    }
}

#[derive(Component)]
struct FlightDataText;

#[derive(Component)]
struct ForceDataText;

#[derive(Component)]
struct InstructionsText;

fn setup_ui(mut commands: Commands) {
    // Flight data display
    commands.spawn((
        Text::new("Flight Data\n-----------\n"),
        TextColor(Color::WHITE),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        },
        FlightDataText,
    ));
    
    // Force data display
    commands.spawn((
        Text::new("Forces\n------\n"),
        TextColor(Color::WHITE),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(150.0),
            ..default()
        },
        ForceDataText,
    ));
    
    // Instructions
    commands.spawn((
        Text::new(
            "Controls:\n\
            Space: Start/Pause\n\
            W/S: Wing angle\n\
            T: Thrust\n\
            R: Reset\n\
            Arrow Keys: Camera"
        ),
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        },
        InstructionsText,
    ));
}

fn update_ui_text(
    mut flight_text_query: Query<&mut Text, (With<FlightDataText>, Without<ForceDataText>)>,
    mut force_text_query: Query<&mut Text, (With<ForceDataText>, Without<FlightDataText>)>,
    flight_data_query: Query<&FlightData>,
    dynamics_query: Query<&FlightDynamics>,
    params: Res<SimulationParams>,
) {
    // Update flight data text
    if let Ok(mut text) = flight_text_query.single_mut() {
        if let Ok(flight_data) = flight_data_query.single() {
            **text = format!(
                "Flight Data\n\
                -----------\n\
                Altitude: {:.1} m\n\
                Airspeed: {:.1} m/s\n\
                Vertical: {:.1} m/s\n\
                Time: {:.1} s\n\
                Distance: {:.1} m\n\
                Status: {}",
                flight_data.altitude,
                flight_data.airspeed,
                flight_data.vertical_speed,
                flight_data.flight_time,
                flight_data.distance_traveled,
                if params.is_running { "Running" } else { "Paused" }
            );
        }
    }
    
    // Update force data text
    if let Ok(mut text) = force_text_query.single_mut() {
        if let Ok(dynamics) = dynamics_query.single() {
            **text = format!(
                "Forces (N)\n\
                ----------\n\
                Lift: {:.1}\n\
                Drag: {:.1}\n\
                Weight: {:.1}\n\
                Thrust: {:.1}\n\
                Total: {:.1}",
                dynamics.forces.lift.length(),
                dynamics.forces.drag.length(),
                dynamics.forces.weight.length(),
                dynamics.forces.thrust.length(),
                dynamics.forces.total.length()
            );
        }
    }
}

fn handle_parameter_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    // Camera controls
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        let rotation_speed = 2.0 * time.delta_secs();
        let move_speed = 10.0 * time.delta_secs();
        
        // Rotate camera around target
        if keyboard.pressed(KeyCode::ArrowLeft) {
            camera_transform.rotate_around(
                Vec3::new(0.0, 2.0, 0.0),
                Quat::from_rotation_y(rotation_speed),
            );
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            camera_transform.rotate_around(
                Vec3::new(0.0, 2.0, 0.0),
                Quat::from_rotation_y(-rotation_speed),
            );
        }
        
        // Zoom
        if keyboard.pressed(KeyCode::ArrowUp) {
            let direction = (Vec3::new(0.0, 2.0, 0.0) - camera_transform.translation).normalize();
            camera_transform.translation += direction * move_speed;
        }
        if keyboard.pressed(KeyCode::ArrowDown) {
            let direction = (Vec3::new(0.0, 2.0, 0.0) - camera_transform.translation).normalize();
            camera_transform.translation -= direction * move_speed;
        }
    }
}