use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use super::components::*;
use super::resources::*;
use super::flapping::FlappingWing;
use crate::physics::weather::WeatherParams;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(EguiPlugin)
            .init_resource::<UIState>()
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (
                update_ui_text,
                handle_parameter_input,
                render_egui_ui,
            ));
    }
}

#[derive(Resource, Default)]
pub struct UIState {
    pub show_parameters: bool,
    pub show_telemetry: bool,
    pub show_weather: bool,
}

#[derive(Component)]
struct FlightDataText;

#[derive(Component)]
struct ForceDataText;

#[derive(Component)]
struct InstructionsText;

fn setup_ui(mut commands: Commands) {
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
    
    commands.spawn((
        Text::new(
            "Controls:\n\
            Space: Start/Pause\n\
            W/S: Wing angle\n\
            T: Thrust\n\
            F: Toggle Flapping\n\
            R: Reset\n\
            Tab: Show Parameters\n\
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
    mut ui_state: ResMut<UIState>,
    time: Res<Time>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        ui_state.show_parameters = !ui_state.show_parameters;
    }
    
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        let rotation_speed = 2.0 * time.delta_secs();
        let move_speed = 10.0 * time.delta_secs();
        
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

fn render_egui_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UIState>,
    mut wing_query: Query<&mut Wing>,
    mut flyer_query: Query<&mut Flyer>,
    mut propulsion_query: Query<&mut Propulsion>,
    mut flapping_query: Query<&mut FlappingWing>,
    mut atmosphere_query: Query<&mut Atmosphere>,
    mut params: ResMut<SimulationParams>,
    mut weather_params: ResMut<WeatherParams>,
) {
    if !ui_state.show_parameters {
        return;
    }
    
    egui::Window::new("Flight Parameters")
        .default_pos([400.0, 50.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Aircraft Configuration");
            
            ui.collapsing("Wing Parameters", |ui| {
                if let Ok(mut wing) = wing_query.single_mut() {
                    ui.add(egui::Slider::new(&mut wing.span, 5.0..=20.0).text("Wing Span (m)"));
                    ui.add(egui::Slider::new(&mut wing.chord, 0.5..=3.0).text("Wing Chord (m)"));
                    ui.add(egui::Slider::new(&mut wing.lift_coefficient_base, 0.5..=2.5).text("Lift Coefficient"));
                    ui.add(egui::Slider::new(&mut wing.drag_coefficient_base, 0.01..=0.1).text("Drag Coefficient"));
                    ui.add(egui::Slider::new(&mut wing.efficiency_factor, 0.5..=1.0).text("Efficiency Factor"));
                    
                    wing.area = wing.span * wing.chord;
                    wing.aspect_ratio = wing.span * wing.span / wing.area;
                    
                    ui.label(format!("Wing Area: {:.2} m²", wing.area));
                    ui.label(format!("Aspect Ratio: {:.2}", wing.aspect_ratio));
                }
            });
            
            ui.collapsing("Mass & Propulsion", |ui| {
                if let Ok(mut flyer) = flyer_query.single_mut() {
                    ui.add(egui::Slider::new(&mut flyer.mass, 40.0..=150.0).text("Total Mass (kg)"));
                }
                
                if let Ok(mut propulsion) = propulsion_query.single_mut() {
                    ui.add(egui::Slider::new(&mut propulsion.thrust_power, 0.0..=1000.0).text("Thrust Power (N)"));
                    ui.add(egui::Slider::new(&mut propulsion.throttle, 0.0..=1.0).text("Throttle"));
                    ui.add(egui::Slider::new(&mut propulsion.efficiency, 0.5..=0.95).text("Propulsion Efficiency"));
                }
            });
            
            ui.collapsing("Flapping Wing", |ui| {
                if let Ok(mut flapping) = flapping_query.single_mut() {
                    ui.checkbox(&mut flapping.is_active, "Enable Flapping");
                    ui.add(egui::Slider::new(&mut flapping.frequency, 0.5..=5.0).text("Flap Frequency (Hz)"));
                    ui.add(egui::Slider::new(&mut flapping.amplitude, 0.0..=90.0).text("Flap Amplitude (°)"));
                    ui.add(egui::Slider::new(&mut flapping.power_stroke_ratio, 0.2..=0.5).text("Power Stroke Ratio"));
                    ui.add(egui::Slider::new(&mut flapping.twist_amplitude, 0.0..=45.0).text("Wing Twist (°)"));
                }
            });
            
            ui.separator();
            
            ui.checkbox(&mut ui_state.show_weather, "Weather Controls");
            
            if ui_state.show_weather {
                ui.collapsing("Weather Parameters", |ui| {
                    ui.add(egui::Slider::new(&mut weather_params.base_wind.x, -20.0..=20.0).text("Wind X (m/s)"));
                    ui.add(egui::Slider::new(&mut weather_params.base_wind.z, -20.0..=20.0).text("Wind Z (m/s)"));
                    ui.add(egui::Slider::new(&mut weather_params.turbulence_intensity, 0.0..=1.0).text("Turbulence"));
                    ui.add(egui::Slider::new(&mut weather_params.temperature, -20.0..=40.0).text("Temperature (°C)"));
                    ui.add(egui::Slider::new(&mut weather_params.pressure, 90000.0..=105000.0).text("Pressure (Pa)"));
                    ui.add(egui::Slider::new(&mut weather_params.humidity, 0.0..=1.0).text("Humidity"));
                    
                    if let Ok(atmosphere) = atmosphere_query.single() {
                        ui.label(format!("Air Density: {:.3} kg/m³", atmosphere.air_density));
                    }
                });
            }
            
            ui.separator();
            
            ui.checkbox(&mut ui_state.show_telemetry, "Show Telemetry");
            
            ui.horizontal(|ui| {
                if ui.button("Reset Defaults").clicked() {
                    *params = SimulationParams::default();
                    *weather_params = WeatherParams::default();
                }
                
                if ui.button("Close").clicked() {
                    ui_state.show_parameters = false;
                }
            });
        });
    
    if ui_state.show_telemetry {
        egui::Window::new("Flight Telemetry")
            .default_pos([800.0, 50.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Real-time Data");
                ui.label("Detailed telemetry data will be displayed here");
            });
    }
}