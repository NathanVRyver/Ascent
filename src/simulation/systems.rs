use bevy::prelude::*;
use crate::physics::{
    lift::{calculate_lift_force, LiftParams},
    drag::{calculate_total_drag, DragParams},
    ground_effect::{calculate_ground_effect_factor, apply_ground_effect_to_lift, GroundEffectParams},
    stall::{calculate_lift_coefficient_with_stall, calculate_drag_coefficient_stalled, StallParams},
    thrust::{calculate_thrust_force, ThrustParams},
    weather::{calculate_air_density, calculate_wind_with_turbulence},
};
use super::components::*;
use super::resources::*;
use super::flapping::FlappingWing;
use super::visualization::TrajectoryTrail;
use super::human_model::{create_human_flyer_bundle, create_realistic_wings};
use super::stabilization::FlightStabilizer;
use crate::physics::weather::WeatherParams;

pub fn setup_camera(_commands: Commands) {
    // Camera will be set up by the follow camera system
}

pub fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500.0, 500.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.35, 0.2),
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO),
        GroundPlane,
    ));
    
    let grid_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.4, 0.4),
        unlit: true,
        ..default()
    });
    
    for i in -50..=50 {
        if i == 0 { continue; }
        let offset = i as f32 * 10.0;
        
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(500.0, 0.02, 0.1))),
            MeshMaterial3d(grid_material.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.01, offset)),
        ));
        
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.1, 0.02, 500.0))),
            MeshMaterial3d(grid_material.clone()),
            Transform::from_translation(Vec3::new(offset, 0.01, 0.0)),
        ));
    }
    
    commands.spawn((
        Atmosphere {
            air_density: 1.225,
            wind_velocity: Vec3::new(5.0, 0.0, 2.0),
            turbulence_intensity: 0.1,
            temperature: 15.0,
        },
    ));
}

pub fn spawn_flyer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create human model
    let (torso_mesh, torso_material, body_parts) = create_human_flyer_bundle(&mut meshes, &mut materials);
    let (wing_mesh, wing_material) = create_realistic_wings(&mut meshes, &mut materials);
    
    commands.spawn((
        torso_mesh,
        torso_material,
        Transform::from_translation(Vec3::new(0.0, 5.0, 0.0)),
        Flyer { mass: 80.0 },
        Propulsion {
            thrust_power: 500.0,
            thrust_direction: Vec3::new(0.0, 0.5, 1.0).normalize(),
            efficiency: 0.85,
            propeller_diameter: 1.2,
            throttle: 0.0,
        },
        FlightDynamics {
            velocity: Vec3::new(0.0, 0.0, 0.0),
            acceleration: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            forces: Forces::default(),
        },
        FlightData {
            altitude: 5.0,
            airspeed: 0.0,
            vertical_speed: 0.0,
            flight_time: 0.0,
            distance_traveled: 0.0,
        },
        StallIndicator {
            is_stalled: false,
            stall_severity: 0.0,
        },
        TrajectoryTrail::default(),
        FlightStabilizer::default(),
    )).with_children(|parent| {
        // Add body parts
        for (mesh, material, transform) in body_parts {
            parent.spawn((mesh, material, transform));
        }
        
        // Add wings
        parent.spawn((
            wing_mesh.clone(),
            wing_material.clone(),
            Transform::from_translation(Vec3::new(-2.5, 0.5, 0.0)),
            Wing {
                span: 5.0,
                chord: 1.0,
                area: 5.0,
                aspect_ratio: 5.0,
                angle_of_attack: 0.1,
                lift_coefficient_base: 1.2,
                drag_coefficient_base: 0.03,
                efficiency_factor: 0.85,
            },
            FlappingWing::default(),
        ));
        
        parent.spawn((
            wing_mesh,
            wing_material,
            Transform::from_translation(Vec3::new(2.5, 0.5, 0.0)),
            Wing {
                span: 5.0,
                chord: 1.0,
                area: 5.0,
                aspect_ratio: 5.0,
                angle_of_attack: 0.1,
                lift_coefficient_base: 1.2,
                drag_coefficient_base: 0.03,
                efficiency_factor: 0.85,
            },
            FlappingWing::default(),
        ));
    });
}

pub fn update_physics(
    time: Res<Time>,
    params: Res<SimulationParams>,
    weather_params: Res<WeatherParams>,
    mut atmosphere_query: Query<&mut Atmosphere>,
    mut query: Query<(
        &Flyer,
        &mut FlightDynamics,
        &Transform,
        &Children,
        &Propulsion,
        Option<&mut StallIndicator>,
    )>,
    mut wing_query: Query<(&Wing, &FlappingWing)>,
) {
    if !params.is_running {
        return;
    }
    
    if let Ok(mut atmosphere) = atmosphere_query.single_mut() {
        atmosphere.air_density = calculate_air_density(
            weather_params.temperature,
            weather_params.pressure,
            weather_params.humidity,
        );
        atmosphere.wind_velocity = calculate_wind_with_turbulence(
            &weather_params,
            Vec3::ZERO,
            time.elapsed_secs(),
        );
    }
    
    let atmosphere = atmosphere_query.single().unwrap();
    
    for (flyer, mut dynamics, transform, children, propulsion, mut stall_indicator) in query.iter_mut() {
        let weight = Vec3::new(0.0, -flyer.mass * params.gravity, 0.0);
        dynamics.forces.weight = weight;
        
        let mut total_lift = Vec3::ZERO;
        let mut total_drag = Vec3::ZERO;
        let mut _total_wing_area = 0.0;
        
        for child in children.iter() {
            if let Ok((wing, flapping)) = wing_query.get_mut(child) {
                _total_wing_area += wing.area;
                
                let airspeed_vector = dynamics.velocity - atmosphere.wind_velocity;
                
                let stall_params = StallParams {
                    angle_of_attack: wing.angle_of_attack,
                    critical_angle: 15.0_f32.to_radians(),
                    post_stall_drop: 0.5,
                    stall_progression_rate: 2.0,
                };
                
                let lift_coefficient = calculate_lift_coefficient_with_stall(
                    wing.lift_coefficient_base,
                    wing.angle_of_attack,
                    &stall_params,
                );
                
                let lift_params = LiftParams {
                    air_density: atmosphere.air_density,
                    velocity: airspeed_vector,
                    wing_area: wing.area,
                    wing_span: wing.span,
                    wing_chord: wing.chord,
                    angle_of_attack: wing.angle_of_attack,
                };
                
                let mut lift_force = calculate_lift_force(&lift_params, lift_coefficient);
                
                let ground_effect_params = GroundEffectParams {
                    altitude: transform.translation.y,
                    wing_span: wing.span,
                    wing_chord: wing.chord,
                };
                
                let ground_effect_factor = calculate_ground_effect_factor(&ground_effect_params);
                lift_force = apply_ground_effect_to_lift(lift_force, ground_effect_factor);
                
                total_lift += lift_force;
                
                let drag_coefficient = calculate_drag_coefficient_stalled(
                    wing.drag_coefficient_base,
                    wing.angle_of_attack,
                    &stall_params,
                );
                
                let drag_params = DragParams {
                    air_density: atmosphere.air_density,
                    velocity: airspeed_vector,
                    wing_area: wing.area,
                    drag_coefficient,
                    aspect_ratio: wing.aspect_ratio,
                    efficiency_factor: wing.efficiency_factor,
                };
                
                let drag_force = calculate_total_drag(&drag_params, lift_coefficient);
                total_drag += drag_force;
                
                if flapping.is_active {
                    let flapping_thrust = super::flapping::calculate_flapping_thrust(
                        flapping,
                        wing.area,
                        atmosphere.air_density,
                        time.elapsed_secs(),
                    );
                    dynamics.forces.thrust += flapping_thrust;
                }
                
                if let Some(ref mut stall) = stall_indicator {
                    stall.is_stalled = wing.angle_of_attack.abs() > stall_params.critical_angle;
                    stall.stall_severity = if stall.is_stalled {
                        ((wing.angle_of_attack.abs() - stall_params.critical_angle) / stall_params.critical_angle).min(1.0)
                    } else {
                        0.0
                    };
                }
            }
        }
        
        dynamics.forces.lift = total_lift;
        dynamics.forces.drag = total_drag;
        
        let thrust_params = ThrustParams {
            thrust_power: propulsion.thrust_power * propulsion.throttle,
            thrust_direction: propulsion.thrust_direction,
            efficiency: propulsion.efficiency,
            propeller_diameter: propulsion.propeller_diameter,
            air_density: atmosphere.air_density,
            velocity: dynamics.velocity,
        };
        
        let base_thrust = calculate_thrust_force(&thrust_params);
        dynamics.forces.thrust = dynamics.forces.thrust + base_thrust;
        
        dynamics.forces.total = dynamics.forces.weight + dynamics.forces.lift + dynamics.forces.drag + dynamics.forces.thrust;
        
        dynamics.acceleration = dynamics.forces.total / flyer.mass;
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
        let acceleration = dynamics.acceleration;
        dynamics.velocity += acceleration * dt;
        
        let displacement = dynamics.velocity * dt;
        transform.translation += displacement;
        
        // Ground collision with better landing mechanics
        let ground_level = 1.0; // Account for human height
        if transform.translation.y <= ground_level {
            transform.translation.y = ground_level;
            
            let impact_velocity = dynamics.velocity.length();
            
            if impact_velocity > 20.0 {
                info!("CRASH! Impact velocity: {:.1} m/s", impact_velocity);
                // Hard crash - stop all movement
                dynamics.velocity = Vec3::ZERO;
                dynamics.acceleration = Vec3::ZERO;
            } else if impact_velocity > 8.0 {
                info!("Hard landing! Impact velocity: {:.1} m/s", impact_velocity);
                // Hard landing - reduce all velocity
                dynamics.velocity *= 0.3;
                dynamics.velocity.y = 0.0;
            } else if impact_velocity > 3.0 {
                info!("Landing. Impact velocity: {:.1} m/s", impact_velocity);
                // Normal landing - soft stop
                dynamics.velocity *= 0.8;
                dynamics.velocity.y = 0.0;
            } else {
                // Gentle touchdown
                dynamics.velocity.y = 0.0;
                dynamics.velocity *= 0.95;
            }
        }
        
        flight_data.altitude = transform.translation.y;
        flight_data.airspeed = dynamics.velocity.length();
        flight_data.vertical_speed = dynamics.velocity.y;
        flight_data.flight_time += dt;
        flight_data.distance_traveled += displacement.length();
    }
}

pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut params: ResMut<SimulationParams>,
    mut wing_query: Query<&mut Wing>,
    mut query: Query<(
        &mut Transform,
        &mut FlightDynamics,
        &mut FlightData,
        &mut Propulsion,
    ), With<Flyer>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        params.is_running = !params.is_running;
    }
    
    let dt = time.delta_secs();
    let wing_control_speed = 1.0; // radians per second
    
    if keyboard.pressed(KeyCode::KeyW) {
        for mut wing in wing_query.iter_mut() {
            wing.angle_of_attack = (wing.angle_of_attack + wing_control_speed * dt).min(0.35);
        }
    }
    if keyboard.pressed(KeyCode::KeyS) {
        for mut wing in wing_query.iter_mut() {
            wing.angle_of_attack = (wing.angle_of_attack - wing_control_speed * dt).max(-0.15);
        }
    }
    
    let throttle_speed = 2.0; // per second
    
    for (_, _, _, mut propulsion) in query.iter_mut() {
        if keyboard.pressed(KeyCode::KeyT) {
            propulsion.throttle = (propulsion.throttle + throttle_speed * dt).min(1.0);
        } else {
            propulsion.throttle = (propulsion.throttle - throttle_speed * dt * 0.5).max(0.0);
        }
    }
    
    if keyboard.just_pressed(KeyCode::KeyR) {
        params.is_running = false;
        for (mut transform, mut dynamics, mut flight_data, mut propulsion) in query.iter_mut() {
            transform.translation = Vec3::new(0.0, 5.0, 0.0);
            
            dynamics.velocity = Vec3::new(0.0, 0.0, 0.0);
            dynamics.acceleration = Vec3::ZERO;
            dynamics.angular_velocity = Vec3::ZERO;
            dynamics.forces = Forces::default();
            
            flight_data.altitude = 5.0;
            flight_data.airspeed = 0.0;
            flight_data.vertical_speed = 0.0;
            flight_data.flight_time = 0.0;
            flight_data.distance_traveled = 0.0;
            
            propulsion.throttle = 0.0;
        }
        
        for mut wing in wing_query.iter_mut() {
            wing.angle_of_attack = 0.1;
        }
    }
}