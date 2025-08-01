mod components;
mod systems;
mod resources;
mod ui;
mod visualization;
mod wing_geometry;
mod flapping;

use bevy::prelude::*;
use ui::UIPlugin;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(UIPlugin)
            .init_resource::<resources::SimulationParams>()
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
            ));
    }
}