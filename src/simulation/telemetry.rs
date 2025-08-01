use bevy::prelude::*;
use std::fs::File;
use std::io::Write;
use std::collections::VecDeque;
use super::components::*;

#[derive(Resource)]
pub struct TelemetrySystem {
    pub enabled: bool,
    pub recording: bool,
    pub data_points: VecDeque<TelemetryDataPoint>,
    pub max_data_points: usize,
    pub recording_interval: f32,
    pub last_record_time: f32,
    pub export_path: String,
}

impl Default for TelemetrySystem {
    fn default() -> Self {
        Self {
            enabled: true,
            recording: false,
            data_points: VecDeque::new(),
            max_data_points: 10000,
            recording_interval: 0.1,
            last_record_time: 0.0,
            export_path: "flight_telemetry.csv".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TelemetryDataPoint {
    pub timestamp: f32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub altitude: f32,
    pub airspeed: f32,
    pub vertical_speed: f32,
    pub angle_of_attack: f32,
    pub lift_force: f32,
    pub drag_force: f32,
    pub thrust_force: f32,
    pub net_force: f32,
    pub stall_status: bool,
    pub flapping_active: bool,
    pub wind_speed: f32,
    pub air_density: f32,
}

pub fn record_telemetry(
    mut telemetry: ResMut<TelemetrySystem>,
    time: Res<Time>,
    flyer_query: Query<(&Transform, &FlightDynamics, &FlightData), With<Flyer>>,
    wing_query: Query<&Wing>,
    stall_query: Query<&StallIndicator>,
    flapping_query: Query<&super::flapping::FlappingWing>,
    atmosphere_query: Query<&Atmosphere>,
) {
    if !telemetry.enabled || !telemetry.recording {
        return;
    }
    
    let current_time = time.elapsed_secs();
    
    if current_time - telemetry.last_record_time < telemetry.recording_interval {
        return;
    }
    
    telemetry.last_record_time = current_time;
    
    if let Ok((transform, dynamics, flight_data)) = flyer_query.single() {
        let wing = wing_query.single().ok();
        let stall = stall_query.single().ok();
        let flapping = flapping_query.single().ok();
        let atmosphere = atmosphere_query.single().ok();
        
        let data_point = TelemetryDataPoint {
            timestamp: current_time,
            position: transform.translation,
            velocity: dynamics.velocity,
            acceleration: dynamics.acceleration,
            altitude: flight_data.altitude,
            airspeed: flight_data.airspeed,
            vertical_speed: flight_data.vertical_speed,
            angle_of_attack: wing.map(|w| w.angle_of_attack).unwrap_or(0.0),
            lift_force: dynamics.forces.lift.length(),
            drag_force: dynamics.forces.drag.length(),
            thrust_force: dynamics.forces.thrust.length(),
            net_force: dynamics.forces.total.length(),
            stall_status: stall.map(|s| s.is_stalled).unwrap_or(false),
            flapping_active: flapping.map(|f| f.is_active).unwrap_or(false),
            wind_speed: atmosphere.map(|a| a.wind_velocity.length()).unwrap_or(0.0),
            air_density: atmosphere.map(|a| a.air_density).unwrap_or(1.225),
        };
        
        telemetry.data_points.push_back(data_point);
        
        if telemetry.data_points.len() > telemetry.max_data_points {
            telemetry.data_points.pop_front();
        }
    }
}

pub fn toggle_telemetry_recording(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut telemetry: ResMut<TelemetrySystem>,
) {
    if keyboard.just_pressed(KeyCode::KeyL) {
        telemetry.recording = !telemetry.recording;
        info!("Telemetry recording: {}", if telemetry.recording { "Started" } else { "Stopped" });
    }
}

pub fn export_telemetry_data(
    keyboard: Res<ButtonInput<KeyCode>>,
    telemetry: Res<TelemetrySystem>,
) {
    if keyboard.just_pressed(KeyCode::KeyE) && keyboard.pressed(KeyCode::ShiftLeft) {
        if let Err(e) = export_to_csv(&telemetry) {
            error!("Failed to export telemetry: {}", e);
        } else {
            info!("Telemetry exported to: {}", telemetry.export_path);
        }
    }
}

fn export_to_csv(telemetry: &TelemetrySystem) -> std::io::Result<()> {
    let mut file = File::create(&telemetry.export_path)?;
    
    writeln!(file, "timestamp,position_x,position_y,position_z,velocity_x,velocity_y,velocity_z,acceleration_x,acceleration_y,acceleration_z,altitude,airspeed,vertical_speed,angle_of_attack,lift_force,drag_force,thrust_force,net_force,stall_status,flapping_active,wind_speed,air_density")?;
    
    for data_point in &telemetry.data_points {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            data_point.timestamp,
            data_point.position.x,
            data_point.position.y,
            data_point.position.z,
            data_point.velocity.x,
            data_point.velocity.y,
            data_point.velocity.z,
            data_point.acceleration.x,
            data_point.acceleration.y,
            data_point.acceleration.z,
            data_point.altitude,
            data_point.airspeed,
            data_point.vertical_speed,
            data_point.angle_of_attack.to_degrees(),
            data_point.lift_force,
            data_point.drag_force,
            data_point.thrust_force,
            data_point.net_force,
            data_point.stall_status,
            data_point.flapping_active,
            data_point.wind_speed,
            data_point.air_density
        )?;
    }
    
    Ok(())
}

pub fn display_telemetry_stats(
    telemetry: Res<TelemetrySystem>,
    mut contexts: bevy_egui::EguiContexts,
    ui_state: Res<super::ui::UIState>,
) {
    if !ui_state.show_telemetry || telemetry.data_points.is_empty() {
        return;
    }
    
    bevy_egui::egui::Window::new("Flight Telemetry")
        .default_pos([800.0, 50.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Real-time Data");
            
            ui.label(format!("Recording: {}", if telemetry.recording { "Active" } else { "Stopped" }));
            ui.label(format!("Data points: {}", telemetry.data_points.len()));
            
            if let Some(latest) = telemetry.data_points.back() {
                ui.separator();
                ui.label("Latest Values:");
                
                ui.horizontal(|ui| {
                    ui.label(format!("Altitude: {:.1} m", latest.altitude));
                    ui.label(format!("Speed: {:.1} m/s", latest.airspeed));
                });
                
                ui.horizontal(|ui| {
                    ui.label(format!("V-Speed: {:.1} m/s", latest.vertical_speed));
                    ui.label(format!("AoA: {:.1}°", latest.angle_of_attack.to_degrees()));
                });
                
                ui.separator();
                ui.label("Forces:");
                
                ui.horizontal(|ui| {
                    ui.label(format!("Lift: {:.1} N", latest.lift_force));
                    ui.label(format!("Drag: {:.1} N", latest.drag_force));
                });
                
                ui.horizontal(|ui| {
                    ui.label(format!("Thrust: {:.1} N", latest.thrust_force));
                    ui.label(format!("Net: {:.1} N", latest.net_force));
                });
                
                ui.separator();
                
                let max_altitude = telemetry.data_points.iter()
                    .map(|d| d.altitude)
                    .fold(0.0, f32::max);
                let max_speed = telemetry.data_points.iter()
                    .map(|d| d.airspeed)
                    .fold(0.0, f32::max);
                
                ui.label(format!("Max Altitude: {:.1} m", max_altitude));
                ui.label(format!("Max Speed: {:.1} m/s", max_speed));
            }
            
            ui.separator();
            ui.label("Controls:");
            ui.label("L - Toggle Recording");
            ui.label("Shift+E - Export to CSV");
        });
}