mod components;
mod systems;
mod resources;
mod ui;
mod visualization;
mod wing_geometry;
mod flapping;

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
            .add_systems(Startup, (
                systems::setup_camera,
                systems::setup_environment,
                systems::spawn_flyer,
            ))
            .add_systems(Update, (
                systems::update_physics,
                systems::update_flight_dynamics,
                systems::handle_input,
                visualization::visualize_forces,
                visualization::update_trajectory_trail,
                visualization::toggle_visualization_settings,
                flapping::update_flapping_animation,
                flapping::toggle_flapping,
            ));
    }
}