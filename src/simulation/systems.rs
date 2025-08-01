use bevy::prelude::*;
use crate::physics::lift::{calculate_lift, LiftParams};
use super::components::*;
use super::resources::*;

pub fn setup_camera(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
    ));
    
    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
    ));
}

pub fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO),
        GroundPlane,
    ));
    
    // Grid lines for reference
    let grid_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),
        unlit: true,
        ..default()
    });
    
    for i in -10..=10 {
        if i == 0 { continue; }
        let offset = i as f32 * 5.0;
        
        // X-axis lines
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(100.0, 0.01, 0.1))),
            MeshMaterial3d(grid_material.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.01, offset)),
        ));
        
        // Z-axis lines
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.1, 0.01, 100.0))),
            MeshMaterial3d(grid_material.clone()),
            Transform::from_translation(Vec3::new(offset, 0.01, 0.0)),
        ));
    }
}

pub fn spawn_flyer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let flyer_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.2),
        ..default()
    });
    
    let wing_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.2, 0.8, 0.8),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    
    // Spawn flyer entity (human body)
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.3, 1.8))),
        MeshMaterial3d(flyer_material),
        Transform::from_translation(Vec3::new(0.0, 5.0, 0.0)),
        Flyer { mass: 80.0 },
        FlightDynamics {
            velocity: Vec3::new(0.0, 0.0, 10.0), // Start with some forward velocity
            acceleration: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            forces: Forces::default(),
        },
        FlightData {
            altitude: 5.0,
            airspeed: 10.0,
            vertical_speed: 0.0,
            flight_time: 0.0,
            distance_traveled: 0.0,
        },
    )).with_children(|parent| {
        // Spawn wings as children
        let wing_span = 6.0;  // 6 meter wingspan
        let wing_chord = 1.0; // 1 meter average chord
        let wing_area = wing_span * wing_chord * 0.8; // Simplified calculation
        
        // Left wing
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(wing_span / 2.0, 0.1, wing_chord))),
            MeshMaterial3d(wing_material.clone()),
            Transform::from_translation(Vec3::new(-wing_span / 4.0, 0.0, 0.0)),
            Wing {
                area: wing_area / 2.0,
                angle_of_attack: 0.1, // ~5.7 degrees
            },
        ));
        
        // Right wing
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(wing_span / 2.0, 0.1, wing_chord))),
            MeshMaterial3d(wing_material),
            Transform::from_translation(Vec3::new(wing_span / 4.0, 0.0, 0.0)),
            Wing {
                area: wing_area / 2.0,
                angle_of_attack: 0.1,
            },
        ));
    });
}

pub fn update_physics(
    _time: Res<Time>,
    params: Res<SimulationParams>,
    mut query: Query<(&Flyer, &mut FlightDynamics, &Transform, &Children)>,
    wing_query: Query<&Wing>,
) {
    if !params.is_running {
        return;
    }
    
    for (flyer, mut dynamics, _transform, children) in query.iter_mut() {
        // Calculate weight
        let weight = Vec3::new(0.0, -flyer.mass * params.gravity, 0.0);
        dynamics.forces.weight = weight;
        
        // Calculate lift for all wings
        let mut total_lift = Vec3::ZERO;
        let mut total_wing_area = 0.0;
        
        for child in children.iter() {
            if let Ok(wing) = wing_query.get(child) {
                total_wing_area += wing.area;
                
                // Calculate airspeed (velocity relative to air)
                let airspeed = (dynamics.velocity - params.wind_velocity).length();
                
                // Calculate lift coefficient (simplified model)
                let lift_coefficient = calculate_lift_coefficient(wing.angle_of_attack);
                
                let lift_params = LiftParams {
                    air_density: params.air_density as f64,
                    air_speed: airspeed as f64,
                    wing_area: wing.area as f64,
                    lift_coefficient: lift_coefficient as f64,
                };
                
                let lift_magnitude = calculate_lift(&lift_params) as f32;
                
                // Lift is perpendicular to velocity direction
                let lift_direction = if dynamics.velocity.length() > 0.1 {
                    // Simplified: lift is mostly upward with slight forward component based on velocity
                    Vec3::Y
                } else {
                    Vec3::Y
                };
                
                total_lift += lift_direction * lift_magnitude;
            }
        }
        
        dynamics.forces.lift = total_lift;
        
        // Calculate drag (simplified)
        let drag_coefficient = 0.3;
        let drag_magnitude = 0.5 * params.air_density * dynamics.velocity.length_squared() * total_wing_area * drag_coefficient;
        dynamics.forces.drag = if dynamics.velocity.length() > 0.1 {
            -dynamics.velocity.normalize() * drag_magnitude
        } else {
            Vec3::ZERO
        };
        
        // Sum all forces
        dynamics.forces.total = dynamics.forces.weight + dynamics.forces.lift + dynamics.forces.drag + dynamics.forces.thrust;
        
        // Update acceleration
        dynamics.acceleration = dynamics.forces.total / flyer.mass;
        
        // Debug output for troubleshooting
        if params.is_running && total_lift.length() > 0.1 {
            println!("Lift: {:.1}N, Weight: {:.1}N, Net: {:.1}N", 
                total_lift.length(), 
                weight.length(), 
                (total_lift + weight).length());
        }
    }
}

pub fn update_flight_dynamics(
    time: Res<Time>,
    params: Res<SimulationParams>,
    mut query: Query<(&mut Transform, &mut FlightDynamics, &mut FlightData)>,
) {
    if !params.is_running {
        return;
    }
    
    let dt = time.delta_secs() * params.simulation_speed;
    
    for (mut transform, mut dynamics, mut flight_data) in query.iter_mut() {
        // Update velocity
        let acceleration = dynamics.acceleration;
        dynamics.velocity += acceleration * dt;
        
        // Update position
        let displacement = dynamics.velocity * dt;
        transform.translation += displacement;
        
        // Prevent going below ground
        if transform.translation.y < 0.5 {
            transform.translation.y = 0.5;
            dynamics.velocity.y = 0.0;
            
            // Check for crash conditions
            if dynamics.velocity.length() > 10.0 {
                println!("Crash! Impact velocity: {:.2} m/s", dynamics.velocity.length());
            }
        }
        
        // Update flight data
        flight_data.altitude = transform.translation.y;
        flight_data.airspeed = dynamics.velocity.length();
        flight_data.vertical_speed = dynamics.velocity.y;
        flight_data.flight_time += dt;
        flight_data.distance_traveled += displacement.length();
        
        // Debug output
        if transform.translation.y < 0.0 || transform.translation.y > 1000.0 {
            println!("WARNING: Object position out of bounds: {:?}", transform.translation);
            println!("Velocity: {:?}, Acceleration: {:?}", dynamics.velocity, dynamics.acceleration);
        }
    }
}

pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut params: ResMut<SimulationParams>,
    mut wing_query: Query<&mut Wing>,
    mut query: Query<(&mut Transform, &mut FlightDynamics, &mut FlightData), With<Flyer>>,
) {
    // Start/stop simulation
    if keyboard.just_pressed(KeyCode::Space) {
        params.is_running = !params.is_running;
        println!("Simulation {}", if params.is_running { "started" } else { "paused" });
    }
    
    // Adjust wing angle of attack
    if keyboard.pressed(KeyCode::KeyW) {
        for mut wing in wing_query.iter_mut() {
            wing.angle_of_attack = (wing.angle_of_attack + 0.01).min(0.3);
            println!("Wing angle: {:.1}°", wing.angle_of_attack.to_degrees());
        }
    }
    if keyboard.pressed(KeyCode::KeyS) {
        for mut wing in wing_query.iter_mut() {
            wing.angle_of_attack = (wing.angle_of_attack - 0.01).max(-0.1);
            println!("Wing angle: {:.1}°", wing.angle_of_attack.to_degrees());
        }
    }
    
    // Add thrust
    if keyboard.pressed(KeyCode::KeyT) {
        for (_, mut dynamics, _) in query.iter_mut() {
            dynamics.forces.thrust = Vec3::new(0.0, 100.0, 50.0); // Upward and forward thrust
            println!("Thrust applied: {:?}", dynamics.forces.thrust);
        }
    } else {
        for (_, mut dynamics, _) in query.iter_mut() {
            dynamics.forces.thrust = Vec3::ZERO;
        }
    }
    
    // Reset simulation
    if keyboard.just_pressed(KeyCode::KeyR) {
        params.is_running = false;
        for (mut transform, mut dynamics, mut flight_data) in query.iter_mut() {
            // Reset position
            transform.translation = Vec3::new(0.0, 5.0, 0.0);
            
            // Reset dynamics
            dynamics.velocity = Vec3::ZERO;
            dynamics.acceleration = Vec3::ZERO;
            dynamics.angular_velocity = Vec3::ZERO;
            dynamics.forces = Forces::default();
            
            // Reset flight data
            flight_data.altitude = 5.0;
            flight_data.airspeed = 0.0;
            flight_data.vertical_speed = 0.0;
            flight_data.flight_time = 0.0;
            flight_data.distance_traveled = 0.0;
        }
        println!("Simulation reset");
    }
}

fn calculate_lift_coefficient(angle_of_attack: f32) -> f32 {
    // Simplified lift coefficient model
    // CL = 2π * α for small angles
    // With stall consideration
    let alpha_deg = angle_of_attack.to_degrees();
    
    if alpha_deg < -5.0 {
        0.0
    } else if alpha_deg < 15.0 {
        2.0 * std::f32::consts::PI * angle_of_attack
    } else if alpha_deg < 20.0 {
        // Stall region
        1.2 - (alpha_deg - 15.0) * 0.1
    } else {
        0.5 // Post-stall
    }
}