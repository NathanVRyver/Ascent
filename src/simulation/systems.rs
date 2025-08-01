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
use super::wing_geometry::{create_wing_mesh, WingGeometry};
use super::flapping::FlappingWing;
use super::visualization::TrajectoryTrail;
use crate::physics::weather::WeatherParams;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, 12.0, 15.0).looking_at(Vec3::new(0.0, 3.0, 0.0), Vec3::Y),
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

pub fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(200.0, 200.0))),
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
    
    for i in -20..=20 {
        if i == 0 { continue; }
        let offset = i as f32 * 5.0;
        
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(200.0, 0.02, 0.05))),
            MeshMaterial3d(grid_material.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.01, offset)),
        ));
        
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.05, 0.02, 200.0))),
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
    let body_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.6, 0.4),
        metallic: 0.2,
        perceptual_roughness: 0.6,
        ..default()
    });
    
    let wing_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.9, 0.9, 0.95, 0.9),
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        metallic: 0.1,
        perceptual_roughness: 0.3,
        ..default()
    });
    
    let wing_geometry = WingGeometry::default();
    let wing_mesh = create_wing_mesh(&wing_geometry);
    
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.3, 1.8))),
        MeshMaterial3d(body_material),
        Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
        Flyer { mass: 80.0 },
        Propulsion {
            thrust_power: 500.0,
            thrust_direction: Vec3::new(0.0, 0.5, 1.0).normalize(),
            efficiency: 0.85,
            propeller_diameter: 1.2,
            throttle: 0.0,
        },
        FlightDynamics {
            velocity: Vec3::new(0.0, 0.0, 15.0),
            acceleration: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            forces: Forces::default(),
        },
        FlightData {
            altitude: 10.0,
            airspeed: 15.0,
            vertical_speed: 0.0,
            flight_time: 0.0,
            distance_traveled: 0.0,
        },
        StallIndicator {
            is_stalled: false,
            stall_severity: 0.0,
        },
        TrajectoryTrail::default(),
    )).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(wing_mesh.clone())),
            MeshMaterial3d(wing_material.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.5, -0.3)),
            Wing {
                span: wing_geometry.span,
                chord: (wing_geometry.root_chord + wing_geometry.tip_chord) / 2.0,
                area: wing_geometry.span * (wing_geometry.root_chord + wing_geometry.tip_chord) / 2.0,
                aspect_ratio: wing_geometry.span.powi(2) / (wing_geometry.span * (wing_geometry.root_chord + wing_geometry.tip_chord) / 2.0),
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
        
        if transform.translation.y < 0.5 {
            transform.translation.y = 0.5;
            dynamics.velocity.y = dynamics.velocity.y.max(0.0);
            
            if dynamics.velocity.length() > 15.0 {
                info!("Hard landing! Impact velocity: {:.1} m/s", dynamics.velocity.length());
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
    
    if keyboard.pressed(KeyCode::KeyW) {
        for mut wing in wing_query.iter_mut() {
            wing.angle_of_attack = (wing.angle_of_attack + 0.02).min(0.35);
        }
    }
    if keyboard.pressed(KeyCode::KeyS) {
        for mut wing in wing_query.iter_mut() {
            wing.angle_of_attack = (wing.angle_of_attack - 0.02).max(-0.15);
        }
    }
    
    for (_, _, _, mut propulsion) in query.iter_mut() {
        if keyboard.pressed(KeyCode::KeyT) {
            propulsion.throttle = (propulsion.throttle + 0.02).min(1.0);
        } else {
            propulsion.throttle = (propulsion.throttle - 0.05).max(0.0);
        }
    }
    
    if keyboard.just_pressed(KeyCode::KeyR) {
        params.is_running = false;
        for (mut transform, mut dynamics, mut flight_data, mut propulsion) in query.iter_mut() {
            transform.translation = Vec3::new(0.0, 10.0, 0.0);
            
            dynamics.velocity = Vec3::new(0.0, 0.0, 15.0);
            dynamics.acceleration = Vec3::ZERO;
            dynamics.angular_velocity = Vec3::ZERO;
            dynamics.forces = Forces::default();
            
            flight_data.altitude = 10.0;
            flight_data.airspeed = 15.0;
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