mod components;
mod systems;
mod resources;
mod ui;
mod visualization;
mod wing_geometry;
mod flapping;
mod telemetry;
mod camera;
mod human_model;
mod stabilization;

use bevy::prelude::*;
use ui::UIPlugin;
use crate::physics::weather::WeatherParams;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(UIPlugin)
            .init_resource::<resources::SimulationParams>()
            .init_resource::<WeatherParams>()
            .init_resource::<visualization::VisualizationSettings>()
            .init_resource::<telemetry::TelemetrySystem>()
            .add_systems(Startup, (
                systems::setup_camera,
                systems::setup_environment,
                systems::spawn_flyer,
                camera::setup_follow_camera,
            ))
            .add_systems(Update, systems::update_physics)
            .add_systems(Update, systems::update_flight_dynamics)
            .add_systems(Update, systems::handle_input)
            .add_systems(Update, visualization::visualize_forces)
            .add_systems(Update, visualization::update_trajectory_trail)
            .add_systems(Update, visualization::toggle_visualization_settings)
            .add_systems(Update, flapping::update_flapping_animation)
            .add_systems(Update, flapping::toggle_flapping)
            .add_systems(Update, telemetry::record_telemetry)
            .add_systems(Update, telemetry::toggle_telemetry_recording)
            .add_systems(Update, telemetry::export_telemetry_data)
            .add_systems(Update, camera::update_follow_camera)
            .add_systems(Update, camera::reset_camera_on_flyer_reset)
            .add_systems(Update, stabilization::apply_flight_stabilization)
            .add_systems(Update, stabilization::add_ground_avoidance);
            // .add_systems(Update, telemetry::display_telemetry_stats);
    }
}