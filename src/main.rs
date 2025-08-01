mod physics;
mod simulation;

use bevy::prelude::*;
use simulation::SimulationPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimulationPlugin)
        .run();
}