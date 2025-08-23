use macroquad::prelude::*;
use egui_macroquad::egui::{self, Color32, RichText, Stroke, Vec2 as EguiVec2};
use egui_macroquad;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
struct FlightParams {
    pilot_mass: f32,
    pilot_power_sustained: f32,
    pilot_power_burst: f32,
    
    wing_count: u32,
    wing_span: f32,
    wing_chord: f32,
    wing_thickness_ratio: f32,
    
    wing_material: WingMaterial,
    spar_material: SparMaterial,
    wing_safety_factor: f32,
    
    motor_power: f32,
    motor_mass: f32,
    battery_capacity: f32,
    motor_efficiency: f32,
    
    airfoil_cl_max: f32,
    airfoil_cd_min: f32,
    oswald_efficiency: f32,
    
    forward_speed: f32,
    flapping_frequency: f32,
    flapping_amplitude: f32,
    air_density: f32,
    wind_speed: f32,
}

#[derive(Clone, Debug, PartialEq)]
enum WingMaterial {
    Fabric,
    Carbon,
    Wood,
    Aluminum,
}

#[derive(Clone, Debug, PartialEq)]
enum SparMaterial {
    Carbon,
    Aluminum,
    Wood,
    Steel,
}

#[derive(Clone, Debug, PartialEq)]
enum FlightPreset {
    Default,
    UltralightGlider,
    PoweredTakeoff,
    SustainedFlight,
    MaxEfficiency,
    MinimalWeight,
    RacingConfig,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum UITab {
    Configuration,
    Analysis,
    Physics,
    Optimization,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FlightPhase {
    OnGround,
    Takeoff,
    InFlight,
    Landing,
}

impl FlightParams {
    fn wing_area(&self) -> f32 {
        self.wing_count as f32 * self.wing_span * self.wing_chord
    }
    
    fn aspect_ratio(&self) -> f32 {
        // For multiple wings, effective aspect ratio accounts for all wings
        let single_wing_ar = self.wing_span / self.wing_chord;
        if self.wing_count == 4 {
            // Four wings have lower effective AR due to interference
            single_wing_ar * 0.7
        } else {
            single_wing_ar
        }
    }
    
    fn from_preset(preset: FlightPreset) -> Self {
        match preset {
            FlightPreset::Default => Self::default(),
            
            FlightPreset::UltralightGlider => Self {
                pilot_mass: 55.0,  // Very light pilot
                pilot_power_sustained: 400.0,  // Elite cyclist level
                pilot_power_burst: 1200.0,
                wing_count: 2,
                wing_span: 12.0,  // Much larger wings like Gossamer Albatross
                wing_chord: 1.8,
                wing_thickness_ratio: 0.08,
                wing_material: WingMaterial::Fabric,
                spar_material: SparMaterial::Carbon,
                wing_safety_factor: 1.5,
                motor_power: 0.0,
                motor_mass: 0.0,
                battery_capacity: 0.0,
                motor_efficiency: 0.0,
                airfoil_cl_max: 2.2,  // High-lift airfoil
                airfoil_cd_min: 0.004,  // Very low drag
                oswald_efficiency: 0.92,  // Excellent efficiency
                forward_speed: 8.0,  // Slow but efficient
                flapping_frequency: 0.0,
                flapping_amplitude: 0.0,
                air_density: 1.225,
                wind_speed: 0.0,
            },
            
            FlightPreset::PoweredTakeoff => Self {
                pilot_mass: 70.0,
                pilot_power_sustained: 350.0,
                pilot_power_burst: 1000.0,
                wing_count: 2,
                wing_span: 10.0,  // Large wings for takeoff
                wing_chord: 2.0,
                wing_thickness_ratio: 0.10,
                wing_material: WingMaterial::Carbon,
                spar_material: SparMaterial::Carbon,
                wing_safety_factor: 2.0,
                motor_power: 8000.0,  // Strong motor for takeoff
                motor_mass: 15.0,
                battery_capacity: 2000.0,
                motor_efficiency: 0.90,
                airfoil_cl_max: 2.0,
                airfoil_cd_min: 0.005,
                oswald_efficiency: 0.88,
                forward_speed: 12.0,
                flapping_frequency: 0.0,
                flapping_amplitude: 0.0,
                air_density: 1.225,
                wind_speed: 0.0,
            },
            
            FlightPreset::SustainedFlight => Self {
                pilot_mass: 50.0,  // Very light
                pilot_power_sustained: 500.0,  // World-class endurance
                pilot_power_burst: 1500.0,
                wing_count: 2,
                wing_span: 15.0,  // Massive wings
                wing_chord: 1.5,
                wing_thickness_ratio: 0.06,  // Very thin for efficiency
                wing_material: WingMaterial::Carbon,
                spar_material: SparMaterial::Carbon,
                wing_safety_factor: 1.3,
                motor_power: 3000.0,  // Significant motor assistance
                motor_mass: 8.0,
                battery_capacity: 3000.0,
                motor_efficiency: 0.95,
                airfoil_cl_max: 2.5,  // Very high-lift airfoil
                airfoil_cd_min: 0.003,  // Extremely low drag
                oswald_efficiency: 0.95,
                forward_speed: 6.5,  // Very slow for efficiency
                flapping_frequency: 0.0,
                flapping_amplitude: 0.0,
                air_density: 1.225,
                wind_speed: 0.0,
            },
            
            FlightPreset::MaxEfficiency => Self {
                pilot_mass: 55.0,
                pilot_power_sustained: 350.0,
                pilot_power_burst: 1000.0,
                wing_count: 2,
                wing_span: 8.0,
                wing_chord: 0.9,
                wing_thickness_ratio: 0.08,
                wing_material: WingMaterial::Carbon,
                spar_material: SparMaterial::Carbon,
                wing_safety_factor: 1.6,
                motor_power: 1000.0,
                motor_mass: 4.0,
                battery_capacity: 600.0,
                motor_efficiency: 0.92,
                airfoil_cl_max: 2.0,
                airfoil_cd_min: 0.004,
                oswald_efficiency: 0.90,
                forward_speed: 9.0,
                flapping_frequency: 0.8,
                flapping_amplitude: 15.0,
                air_density: 1.225,
                wind_speed: 4.0,
            },
            
            FlightPreset::MinimalWeight => Self {
                pilot_mass: 50.0,
                pilot_power_sustained: 400.0,
                pilot_power_burst: 1200.0,
                wing_count: 2,
                wing_span: 6.5,
                wing_chord: 1.1,
                wing_thickness_ratio: 0.07,
                wing_material: WingMaterial::Fabric,
                spar_material: SparMaterial::Carbon,
                wing_safety_factor: 1.4,
                motor_power: 800.0,
                motor_mass: 3.0,
                battery_capacity: 400.0,
                motor_efficiency: 0.85,
                airfoil_cl_max: 1.85,
                airfoil_cd_min: 0.0055,
                oswald_efficiency: 0.86,
                forward_speed: 8.5,
                flapping_frequency: 2.0,
                flapping_amplitude: 25.0,
                air_density: 1.225,
                wind_speed: 5.0,
            },
            
            FlightPreset::RacingConfig => Self {
                pilot_mass: 70.0,
                pilot_power_sustained: 450.0,
                pilot_power_burst: 1500.0,
                wing_count: 2,
                wing_span: 5.0,
                wing_chord: 1.8,
                wing_thickness_ratio: 0.14,
                wing_material: WingMaterial::Aluminum,
                spar_material: SparMaterial::Aluminum,
                wing_safety_factor: 2.5,
                motor_power: 5000.0,
                motor_mass: 15.0,
                battery_capacity: 1000.0,
                motor_efficiency: 0.82,
                airfoil_cl_max: 1.5,
                airfoil_cd_min: 0.009,
                oswald_efficiency: 0.75,
                forward_speed: 20.0,
                flapping_frequency: 0.0,
                flapping_amplitude: 0.0,
                air_density: 1.225,
                wind_speed: -2.0,
            },
        }
    }
}

impl Default for FlightParams {
    fn default() -> Self {
        Self {
            pilot_mass: 80.0,
            pilot_power_sustained: 200.0,
            pilot_power_burst: 600.0,
            
            wing_count: 4,
            wing_span: 3.0,
            wing_chord: 1.5,
            wing_thickness_ratio: 0.12,
            
            wing_material: WingMaterial::Fabric,
            spar_material: SparMaterial::Carbon,
            wing_safety_factor: 2.0,
            
            motor_power: 2000.0,
            motor_mass: 8.0,
            battery_capacity: 500.0,
            motor_efficiency: 0.85,
            
            airfoil_cl_max: 1.6,
            airfoil_cd_min: 0.008,
            oswald_efficiency: 0.8,
            
            forward_speed: 12.0,
            flapping_frequency: 2.0,
            flapping_amplitude: 25.0,
            air_density: 1.225,
            wind_speed: 0.0,
        }
    }
}

#[derive(Clone)]
struct StructuralAnalysis {
    wing_mass: f32,
    spar_mass: f32,
    total_structural_mass: f32,
    max_load_factor: f32,
    wing_deflection: f32,
    critical_flutter_speed: f32,
    structural_feasible: bool,
}

#[derive(Clone)]
struct FlightAnalysis {
    // Basic properties (independent of flight state)
    total_mass: f32,
    weight_force: f32,
    wing_area: f32,
    wing_loading: f32,
    stall_speed: f32,
    
    // Flight-dependent values (only valid when airborne)
    flight_phase: FlightPhase,
    effective_airspeed: f32,
    dynamic_pressure: f32,
    current_lift_coefficient: f32,
    current_drag_coefficient: f32,
    
    // Forces (calculated based on flight conditions)
    lift_force: f32,
    drag_force: f32,
    
    // Power requirements
    power_to_overcome_drag: f32,
    power_for_flapping: f32,
    power_for_climb: f32,
    total_power_required: f32,
    power_loading: f32,
    
    // Flight capabilities
    can_takeoff: bool,
    can_sustain_level_flight: bool,
    can_climb: f32,
    
    motor_flight_time: f32,
    takeoff_distance: f32,
    
    structural: StructuralAnalysis,
    
    reynolds_number: f32,
    flutter_margin: f32,
}

struct SimulationState {
    params: FlightParams,
    analysis: FlightAnalysis,
    history: HistoryData,
    optimization_running: bool,
    optimization_result: Option<FlightParams>,
    camera_rotation: f32,
    time: f32,
    selected_preset: FlightPreset,
    active_tab: UITab,
}

struct HistoryData {
    power_history: VecDeque<f32>,
    lift_history: VecDeque<f32>,
    speed_history: VecDeque<f32>,
    drag_history: VecDeque<f32>,
    time_stamps: VecDeque<f32>,
}

impl HistoryData {
    fn new() -> Self {
        Self {
            power_history: VecDeque::with_capacity(100),
            lift_history: VecDeque::with_capacity(100),
            speed_history: VecDeque::with_capacity(100),
            drag_history: VecDeque::with_capacity(100),
            time_stamps: VecDeque::with_capacity(100),
        }
    }
    
    fn update(&mut self, analysis: &FlightAnalysis, time: f32) {
        if self.power_history.len() >= 100 {
            self.power_history.pop_front();
            self.lift_history.pop_front();
            self.speed_history.pop_front();
            self.drag_history.pop_front();
            self.time_stamps.pop_front();
        }
        
        self.power_history.push_back(analysis.total_power_required);
        self.lift_history.push_back(analysis.lift_force / analysis.weight_force);
        self.speed_history.push_back(analysis.effective_airspeed);
        self.drag_history.push_back(analysis.drag_force);
        self.time_stamps.push_back(time);
    }
}

fn calculate_structural_properties(params: &FlightParams) -> StructuralAnalysis {
    let wing_area_single = params.wing_span * params.wing_chord;
    
    let (wing_density, wing_youngs_modulus) = match params.wing_material {
        WingMaterial::Fabric => (200.0, 1_000_000.0),  // kg/m³, Pa
        WingMaterial::Carbon => (1600.0, 150_000_000_000.0),  // kg/m³, Pa  
        WingMaterial::Wood => (600.0, 10_000_000_000.0),  // kg/m³, Pa
        WingMaterial::Aluminum => (2700.0, 70_000_000_000.0),  // kg/m³, Pa
    };
    
    let (spar_density, spar_youngs_modulus) = match params.spar_material {
        SparMaterial::Carbon => (1600.0, 150_000_000_000.0),  // kg/m³, Pa
        SparMaterial::Aluminum => (2700.0, 70_000_000_000.0),  // kg/m³, Pa
        SparMaterial::Wood => (500.0, 10_000_000_000.0),  // kg/m³, Pa
        SparMaterial::Steel => (7850.0, 200_000_000_000.0),  // kg/m³, Pa
    };
    
    let effective_thickness = params.wing_chord * params.wing_thickness_ratio;
    let wing_skin_mass = wing_area_single * wing_density * effective_thickness * 0.01;  // kg, corrected thickness
    
    let spar_height = effective_thickness * 0.8;
    let spar_width = 0.02;  // 2cm spar width
    let spar_volume = params.wing_span * spar_height * spar_width;  // m³
    let spar_mass = spar_volume * spar_density;  // kg, fixed unit consistency
    let wing_mass = wing_skin_mass + spar_mass + 1.5;  // +1.5kg for ribs, hardware
    
    let total_structural_mass = wing_mass * params.wing_count as f32 + params.motor_mass;
    
    let dynamic_pressure = 0.5 * params.air_density * params.forward_speed.powi(2);
    let max_lift_per_wing = params.airfoil_cl_max * dynamic_pressure * wing_area_single;
    let total_weight = (params.pilot_mass + total_structural_mass) * 9.81;
    let max_load_factor = (max_lift_per_wing * params.wing_count as f32) / total_weight;
    
    let moment_of_inertia = (spar_height.powi(3) * 0.02) / 12.0;
    let distributed_load = max_lift_per_wing / params.wing_span;
    
    let effective_modulus = wing_youngs_modulus * 0.1 + spar_youngs_modulus * 0.9;  // Spar carries most load
    let wing_deflection = (distributed_load * params.wing_span.powi(4)) / (8.0 * effective_modulus * moment_of_inertia);
    
    // Credible flutter speed based on wing stiffness and mass distribution
    let flutter_parameter = (effective_modulus * moment_of_inertia) / (spar_density * wing_area_single * params.wing_span.powi(4));
    let critical_flutter_speed = flutter_parameter.sqrt() * 20.0;  // Empirical scaling
    
    let structural_feasible = max_load_factor >= params.wing_safety_factor && 
                            wing_deflection < params.wing_span * 0.1 &&
                            critical_flutter_speed > params.forward_speed * 1.5;
    
    StructuralAnalysis {
        wing_mass,
        spar_mass,
        total_structural_mass,
        max_load_factor,
        wing_deflection,
        critical_flutter_speed,
        structural_feasible,
    }
}

fn calculate_comprehensive_flight_analysis(params: &FlightParams) -> FlightAnalysis {
    let structural = calculate_structural_properties(params);
    
    // Basic properties that don't depend on flight state
    let total_mass = params.pilot_mass + structural.total_structural_mass;
    let weight_force = total_mass * 9.81;
    let wing_area = params.wing_area();
    let wing_loading = weight_force / wing_area;
    
    // Calculate stall speed (minimum speed needed for lift = weight)
    let stall_speed = (2.0 * weight_force / (params.air_density * wing_area * params.airfoil_cl_max)).sqrt();
    
    // Determine flight phase based on speed and power
    let effective_airspeed = (params.forward_speed - params.wind_speed).max(0.0);
    let available_power = params.pilot_power_burst + params.motor_power * params.motor_efficiency;
    
    let flight_phase = if effective_airspeed < 1.0 {
        FlightPhase::OnGround
    } else if effective_airspeed < stall_speed * 1.1 && available_power > 1000.0 {
        FlightPhase::Takeoff
    } else if effective_airspeed >= stall_speed {
        FlightPhase::InFlight
    } else {
        FlightPhase::OnGround
    };
    
    // Calculate flight-dependent values
    let (dynamic_pressure, current_lift_coefficient, current_drag_coefficient, lift_force, drag_force) = 
    match flight_phase {
        FlightPhase::OnGround => (0.0, 0.0, 0.0, 0.0, 0.0),
        
        FlightPhase::Takeoff => {
            let q = 0.5 * params.air_density * effective_airspeed.powi(2);
            
            // During takeoff, use maximum lift coefficient with flapping boost
            let flapping_boost = if params.flapping_frequency > 0.1 {
                let reduced_frequency = params.flapping_frequency * params.wing_span / effective_airspeed.max(0.1);
                1.0 + (reduced_frequency * 0.3 * (params.flapping_amplitude / 45.0)).min(0.8)
            } else {
                1.0
            };
            
            let cl = params.airfoil_cl_max * flapping_boost;
            let lift = cl * q * wing_area;
            
            // Induced drag with multiple wing penalty
            let base_induced_drag = cl.powi(2) / (std::f32::consts::PI * params.aspect_ratio() * params.oswald_efficiency);
            let multi_wing_penalty = if params.wing_count == 4 {
                1.3  // 30% penalty for wing interference
            } else {
                1.0
            };
            let induced_drag_coeff = base_induced_drag * multi_wing_penalty;
            let cd = params.airfoil_cd_min + induced_drag_coeff;
            let drag = cd * q * wing_area;
            
            (q, cl, cd, lift, drag)
        },
        
        FlightPhase::InFlight => {
            let q = 0.5 * params.air_density * effective_airspeed.powi(2);
            
            // In flight, lift coefficient adjusts to maintain level flight (L = W)
            let flapping_boost = if params.flapping_frequency > 0.1 {
                let reduced_frequency = params.flapping_frequency * params.wing_span / effective_airspeed.max(0.1);
                1.0 + (reduced_frequency * 0.3 * (params.flapping_amplitude / 45.0)).min(0.8)
            } else {
                1.0
            };
            
            // Required CL for level flight: L = W
            let required_cl = weight_force / (q * wing_area * flapping_boost);
            let max_available_cl = params.airfoil_cl_max * flapping_boost;
            let cl = required_cl.min(max_available_cl);
            // If we can't generate enough lift, we're in a dive/descending flight
            let lift = cl * q * wing_area * flapping_boost;
            
            // Induced drag with multiple wing penalty
            let base_induced_drag = cl.powi(2) / (std::f32::consts::PI * params.aspect_ratio() * params.oswald_efficiency);
            let multi_wing_penalty = if params.wing_count == 4 {
                1.3  // 30% penalty for wing interference
            } else {
                1.0
            };
            let induced_drag_coeff = base_induced_drag * multi_wing_penalty;
            let cd = params.airfoil_cd_min + induced_drag_coeff;
            let drag = cd * q * wing_area;
            
            (q, cl, cd, lift, drag)
        },
        
        FlightPhase::Landing => (0.0, 0.0, 0.0, 0.0, 0.0), // Not implemented
    };
    
    // Power calculations
    let power_to_overcome_drag = if flight_phase != FlightPhase::OnGround {
        drag_force * effective_airspeed
    } else {
        0.0
    };
    
    let power_for_flapping = if params.flapping_frequency > 0.1 && flight_phase != FlightPhase::OnGround {
        // More realistic flapping power based on wing inertia and aerodynamic work
        let wing_tip_velocity = params.flapping_frequency * params.wing_span * params.flapping_amplitude.to_radians();
        let wing_inertia = structural.wing_mass * params.wing_span.powi(2) / 3.0;  // Simple beam approximation
        let inertial_power = wing_inertia * (2.0 * std::f32::consts::PI * params.flapping_frequency).powi(2) * wing_tip_velocity;
        inertial_power * params.wing_count as f32 * 0.1  // Efficiency factor
    } else {
        0.0
    };
    
    let power_for_climb = if flight_phase == FlightPhase::InFlight {
        // Power for climb = Weight × Climb_Rate
        // Climb rate determined by excess lift converted to vertical velocity
        let excess_lift = (lift_force - weight_force).max(0.0);
        let climb_rate = excess_lift / weight_force * effective_airspeed * 0.1;  // Small angle approximation
        weight_force * climb_rate
    } else {
        0.0
    };
    
    let total_power_required = power_to_overcome_drag + power_for_flapping + power_for_climb;
    let power_loading = if total_power_required > 0.0 { 
        total_power_required / weight_force 
    } else { 
        0.0 
    };
    
    // Flight capabilities
    let can_takeoff = available_power > total_power_required * 1.3 && 
                     structural.structural_feasible &&
                     effective_airspeed > stall_speed * 0.8;
    
    // Sustained flight includes motor assistance
    let sustained_power_available = params.pilot_power_sustained + 
        (params.motor_power * params.motor_efficiency);
    let can_sustain_level_flight = sustained_power_available > total_power_required && 
                                  effective_airspeed > stall_speed &&
                                  structural.structural_feasible;
    
    // Climb rate calculation with proper power balance
    let excess_power = sustained_power_available - total_power_required;
    let can_climb = if excess_power > 0.0 { 
        excess_power / weight_force  // Climb rate in m/s
    } else { 
        -1.0  // Descent rate if insufficient power
    };
    
    // Battery endurance based on actual motor power draw, not rated power
    let actual_motor_power_draw = if flight_phase != FlightPhase::OnGround {
        (params.motor_power * params.motor_efficiency).min(total_power_required - params.pilot_power_sustained.max(0.0))
    } else {
        0.0
    };
    let motor_flight_time = if actual_motor_power_draw > 0.0 {
        (params.battery_capacity / (actual_motor_power_draw / 1000.0)) * 60.0
    } else {
        f32::INFINITY
    };
    
    // Proper takeoff distance calculation with ground roll
    let takeoff_distance = if flight_phase == FlightPhase::Takeoff {
        let lift_off_speed = stall_speed * 1.2;  // Need 20% margin above stall
        // Ground roll dynamics: thrust must overcome rolling resistance + drag
        let ground_drag = 0.02 * weight_force;  // Rolling resistance
        let net_thrust = available_power / effective_airspeed.max(1.0) - ground_drag;
        let acceleration = net_thrust / total_mass;
        if acceleration > 0.1 {
            lift_off_speed.powi(2) / (2.0 * acceleration)
        } else {
            f32::INFINITY
        }
    } else {
        f32::INFINITY
    };
    
    let chord_length = params.wing_chord;
    let reynolds_number = if effective_airspeed > 0.0 { 
        effective_airspeed * chord_length / 1.5e-5 
    } else { 
        0.0 
    };
    let flutter_margin = structural.critical_flutter_speed / effective_airspeed.max(1.0);
    
    FlightAnalysis {
        total_mass,
        weight_force,
        wing_area,
        wing_loading,
        stall_speed,
        flight_phase,
        effective_airspeed,
        dynamic_pressure,
        current_lift_coefficient,
        current_drag_coefficient,
        lift_force,
        drag_force,
        power_to_overcome_drag,
        power_for_flapping,
        power_for_climb,
        total_power_required,
        power_loading,
        can_takeoff,
        can_sustain_level_flight,
        can_climb,
        motor_flight_time,
        takeoff_distance,
        structural,
        reynolds_number,
        flutter_margin,
    }
}

fn draw_main_visualization(state: &SimulationState) {
    let main_area_x = 400.0;
    let main_area_y = 0.0;
    let main_area_width = screen_width() - main_area_x;
    let main_area_height = screen_height();
    
    draw_rectangle(main_area_x, main_area_y, main_area_width, main_area_height, 
        Color::from_rgba(240, 245, 250, 255));
    
    let center_x = main_area_x + main_area_width / 2.0;
    let center_y = main_area_height / 2.0;
    
    // Ground indicator
    let ground_y = main_area_height - 80.0;
    draw_line(main_area_x, ground_y, main_area_x + main_area_width, ground_y, 
        3.0, Color::from_rgba(100, 80, 60, 255));
    draw_text("GROUND", main_area_x + 20.0, ground_y - 10.0, 16.0, Color::from_rgba(100, 80, 60, 255));
    
    // Keep aircraft visible at consistent height
    let visual_y = match state.analysis.flight_phase {
        FlightPhase::OnGround => center_y + 150.0,
        FlightPhase::Takeoff => center_y + 100.0,
        FlightPhase::InFlight => center_y,
        FlightPhase::Landing => center_y + 120.0,
    };
    
    let wing_scale = 35.0;
    let rotation = state.camera_rotation;
    
    let pilot_color = match state.analysis.flight_phase {
        FlightPhase::OnGround => Color::from_rgba(100, 100, 100, 180),
        FlightPhase::Takeoff => Color::from_rgba(200, 200, 50, 180),
        FlightPhase::InFlight => Color::from_rgba(50, 200, 50, 180),
        FlightPhase::Landing => Color::from_rgba(200, 150, 50, 180),
    };
    
    draw_circle(center_x, visual_y, 25.0, pilot_color);
    draw_text("PILOT", center_x - 20.0, visual_y + 5.0, 16.0, WHITE);
    
    let flap_angle = if state.analysis.flight_phase != FlightPhase::OnGround {
        (state.time * state.params.flapping_frequency * 2.0 * std::f32::consts::PI).sin() 
        * state.params.flapping_amplitude.to_radians()
    } else {
        0.0
    };
    
    for i in 0..state.params.wing_count {
        let angle = rotation + (i as f32 * 2.0 * std::f32::consts::PI / state.params.wing_count as f32);
        let wing_x = center_x + angle.cos() * 40.0;
        let wing_y = visual_y + angle.sin() * 20.0;
        
        let wing_end_x = wing_x + angle.cos() * state.params.wing_span * wing_scale;
        let wing_end_y = wing_y + angle.sin() * state.params.wing_span * wing_scale * 0.3 
            + flap_angle.sin() * 30.0;
        
        let wing_color = if state.analysis.structural.structural_feasible {
            Color::from_rgba(100, 150, 200, 150)
        } else {
            Color::from_rgba(200, 100, 100, 150)
        };
        
        draw_line(wing_x, wing_y, wing_end_x, wing_end_y, 
            state.params.wing_chord * 12.0, wing_color);
        
        draw_line(wing_x, wing_y, wing_end_x, wing_end_y, 3.0, 
            Color::from_rgba(50, 50, 100, 255));
    }
    
    // Force visualization (only when airborne)
    if state.analysis.flight_phase != FlightPhase::OnGround {
        let force_scale = 0.08;
        
        // Lift arrow (green, pointing up)
        let lift_height = state.analysis.lift_force * force_scale;
        draw_line(center_x, visual_y - 40.0, center_x, visual_y - 40.0 - lift_height, 
            4.0, Color::from_rgba(50, 200, 50, 255));
        draw_triangle(
            vec2(center_x, visual_y - 45.0 - lift_height),
            vec2(center_x - 8.0, visual_y - 40.0 - lift_height),
            vec2(center_x + 8.0, visual_y - 40.0 - lift_height),
            Color::from_rgba(50, 200, 50, 255)
        );
        
        // Drag arrow (blue, pointing left)
        let drag_width = state.analysis.drag_force * force_scale * 2.0;
        draw_line(center_x - 60.0, visual_y, center_x - 60.0 - drag_width, visual_y, 
            4.0, Color::from_rgba(50, 50, 200, 255));
        draw_triangle(
            vec2(center_x - 65.0 - drag_width, visual_y),
            vec2(center_x - 60.0 - drag_width, visual_y - 8.0),
            vec2(center_x - 60.0 - drag_width, visual_y + 8.0),
            Color::from_rgba(50, 50, 200, 255)
        );
        
        draw_text(&format!("Lift: {:.0}N", state.analysis.lift_force), 
            center_x + 30.0, visual_y - 60.0, 16.0, Color::from_rgba(50, 200, 50, 255));
        draw_text(&format!("Drag: {:.0}N", state.analysis.drag_force), 
            center_x - 180.0, visual_y - 15.0, 16.0, Color::from_rgba(50, 50, 200, 255));
    }
    
    // Weight arrow (always present, pointing down)
    let force_scale = 0.08;
    let weight_height = state.analysis.weight_force * force_scale;
    draw_line(center_x, visual_y + 40.0, center_x, visual_y + 40.0 + weight_height, 
        4.0, Color::from_rgba(200, 50, 50, 255));
    draw_triangle(
        vec2(center_x, visual_y + 45.0 + weight_height),
        vec2(center_x - 8.0, visual_y + 40.0 + weight_height),
        vec2(center_x + 8.0, visual_y + 40.0 + weight_height),
        Color::from_rgba(200, 50, 50, 255)
    );
    draw_text(&format!("Weight: {:.0}N", state.analysis.weight_force), 
        center_x + 30.0, visual_y + 60.0, 16.0, Color::from_rgba(200, 50, 50, 255));
    
    // Flight phase indicator
    let phase_text = match state.analysis.flight_phase {
        FlightPhase::OnGround => "🔧 ON GROUND",
        FlightPhase::Takeoff => "🛫 TAKEOFF",
        FlightPhase::InFlight => "✈️ IN FLIGHT",
        FlightPhase::Landing => "🛬 LANDING",
    };
    
    let phase_color = match state.analysis.flight_phase {
        FlightPhase::OnGround => Color::from_rgba(100, 100, 100, 255),
        FlightPhase::Takeoff => Color::from_rgba(200, 200, 50, 255),
        FlightPhase::InFlight => Color::from_rgba(50, 200, 50, 255),
        FlightPhase::Landing => Color::from_rgba(200, 150, 50, 255),
    };
    
    draw_text(phase_text, main_area_x + 20.0, 30.0, 24.0, phase_color);
    
    
    draw_text(&format!("Mass: {:.0}kg | Speed: {:.1}m/s | Stall: {:.1}m/s", 
        state.analysis.total_mass, state.analysis.effective_airspeed, state.analysis.stall_speed),
        main_area_x + 20.0, main_area_height - 30.0, 16.0, Color::from_rgba(60, 60, 80, 255));
}

fn draw_physics_equations(ui: &mut egui::Ui, analysis: &FlightAnalysis, params: &FlightParams) {
    ui.heading("Flight Physics Equations");
    
    ui.group(|ui| {
        ui.label(RichText::new("Basic Forces").strong());
        ui.label("Weight: W = mg");
        ui.label(format!("W = {:.0} × 9.81 = {:.0} N", analysis.total_mass, analysis.weight_force));
        ui.separator();
        
        if analysis.flight_phase != FlightPhase::OnGround {
            ui.label("Lift: L = CL × ½ρV² × S");
            ui.label(format!("L = {:.3} × ½×{:.3}×{:.1}² × {:.1}", 
                analysis.current_lift_coefficient, 
                analysis.dynamic_pressure * 2.0 / analysis.effective_airspeed.powi(2).max(0.01), // actual air density
                analysis.effective_airspeed,
                analysis.wing_area));
            ui.label(format!("L = {:.0} N", analysis.lift_force));
            ui.separator();
            
            ui.label("Drag: D = CD × ½ρV² × S");
            ui.label(format!("D = {:.4} × ½×{:.3}×{:.1}² × {:.1}", 
                analysis.current_drag_coefficient,
                analysis.dynamic_pressure * 2.0 / analysis.effective_airspeed.powi(2).max(0.01), // actual air density
                analysis.effective_airspeed,
                analysis.wing_area));
            ui.label(format!("D = {:.0} N", analysis.drag_force));
        } else {
            ui.label("Lift = 0 N (on ground)");
            ui.label("Drag = 0 N (on ground)");
        }
    });
    
    ui.separator();
    
    ui.group(|ui| {
        ui.label(RichText::new("Flight Conditions").strong());
        ui.label("Stall Speed: Vs = √(2W/ρSCLmax)");
        ui.label(format!("Vs = √(2×{:.0}/{:.3}×{:.1}×{:.2})", 
            analysis.weight_force, 
            params.air_density, // Use consistent air density from params
            analysis.wing_area, 
            2.2));  // Use realistic CLmax
        ui.label(format!("Vs = {:.1} m/s", analysis.stall_speed));
        ui.separator();
        
        ui.label("Wing Loading: WL = W/S");
        ui.label(format!("WL = {:.0}/{:.1} = {:.1} N/m²", 
            analysis.weight_force, analysis.wing_area, analysis.wing_loading));
    });
    
    if analysis.flight_phase != FlightPhase::OnGround {
        ui.separator();
        ui.group(|ui| {
            ui.label(RichText::new("Power Requirements").strong());
            ui.label("Power for Drag: P = D × V");
            ui.label(format!("P = {:.0} × {:.1} = {:.0} W", 
                analysis.drag_force, analysis.effective_airspeed, analysis.power_to_overcome_drag));
            ui.separator();
            
            ui.label("Total Power Required:");
            ui.label(format!("P_total = {:.0} W", analysis.total_power_required));
        });
    }
    
    ui.separator();
    ui.group(|ui| {
        ui.label(RichText::new("Flight Phase Logic").strong());
        ui.label("• Speed < 1 m/s → ON GROUND");
        ui.label("• 1 m/s ≤ Speed < Vs×1.1 + High Power → TAKEOFF");
        ui.label("• Speed ≥ Vs → IN FLIGHT");
        ui.separator();
        ui.label(format!("Current: {:?}", analysis.flight_phase));
        ui.label(format!("Speed: {:.1} m/s", analysis.effective_airspeed));
        ui.label(format!("Required for flight: ≥{:.1} m/s", analysis.stall_speed));
    });
}

fn optimize_parameters(base_params: &FlightParams) -> FlightParams {
    let mut best_params = base_params.clone();
    let mut best_score = score_configuration(&best_params);
    
    let param_ranges = vec![
        ("wing_span", 2.0, 8.0, 0.5),
        ("wing_chord", 0.5, 3.0, 0.25),
        ("motor_power", 0.0, 5000.0, 500.0),
        ("forward_speed", 8.0, 20.0, 1.0),
    ];
    
    for _ in 0..10 {
        for (param_name, min_val, max_val, step) in &param_ranges {
            let mut val = *min_val;
            while val <= *max_val {
                let mut test_params = best_params.clone();
                
                match param_name.as_ref() {
                    "wing_span" => test_params.wing_span = val,
                    "wing_chord" => test_params.wing_chord = val,
                    "motor_power" => test_params.motor_power = val,
                    "forward_speed" => test_params.forward_speed = val,
                    _ => {}
                }
                
                let score = score_configuration(&test_params);
                if score > best_score {
                    best_score = score;
                    best_params = test_params;
                }
                
                val += step;
            }
        }
    }
    
    best_params
}

fn score_configuration(params: &FlightParams) -> f32 {
    let analysis = calculate_comprehensive_flight_analysis(params);
    
    let mut score = 0.0;
    
    // Primary objective: sustained flight capability
    if analysis.can_sustain_level_flight && analysis.structural.structural_feasible {
        score += 1000.0;
    } else if analysis.can_takeoff && analysis.structural.structural_feasible {
        score += 300.0;  // Takeoff without sustain is less valuable
    }
    
    // Efficiency metrics (minimize power requirements)
    let power_efficiency = if analysis.total_power_required > 0.0 {
        1000.0 / analysis.total_power_required  // Higher score for lower power
    } else {
        0.0
    };
    score += power_efficiency;
    
    // Mass penalty (lighter is better)
    score -= analysis.total_mass * 5.0;
    
    // Wing loading penalty (lower wing loading is better for human flight)
    score -= analysis.wing_loading * 0.1;
    
    // Stall speed penalty (lower stall speed is safer)
    score -= analysis.stall_speed * 10.0;
    
    // Bonus for realistic motor endurance
    if analysis.motor_flight_time > 10.0 && analysis.motor_flight_time < 60.0 {
        score += 100.0;
    }
    
    // Penalty for excessive takeoff distance
    if analysis.takeoff_distance < 100.0 {
        score += 100.0;
    } else if analysis.takeoff_distance > 500.0 {
        score -= (analysis.takeoff_distance - 500.0) * 0.1;
    }
    
    score.max(0.0)  // Ensure non-negative scores
}

fn draw_parameter_heatmap(ui: &mut egui::Ui, params: &FlightParams) {
    ui.heading("Parameter Sensitivity Analysis");
    
    let param1_range = (1.0, 8.0, 12);
    let param2_range = (0.5, 3.0, 12);
    
    let mut heatmap_data = Vec::new();
    let mut max_score = 0.0f32;
    let mut min_score = f32::MAX;
    
    for i in 0..param1_range.2 {
        let mut row = Vec::new();
        for j in 0..param2_range.2 {
            let wing_span = param1_range.0 + (param1_range.1 - param1_range.0) * (i as f32 / (param1_range.2 - 1) as f32);
            let wing_chord = param2_range.0 + (param2_range.1 - param2_range.0) * (j as f32 / (param2_range.2 - 1) as f32);
            
            let mut test_params = params.clone();
            test_params.wing_span = wing_span;
            test_params.wing_chord = wing_chord;
            
            let analysis = calculate_comprehensive_flight_analysis(&test_params);
            let score = if analysis.can_sustain_level_flight { 
                analysis.lift_force / analysis.weight_force 
            } else { 
                0.0 
            };
            
            max_score = max_score.max(score);
            min_score = min_score.min(score);
            row.push(score);
        }
        heatmap_data.push(row);
    }
    
    let plot_size = EguiVec2::new(320.0, 240.0);
    let response = ui.allocate_response(plot_size, egui::Sense::hover());
    let painter = ui.painter_at(response.rect);
    let rect = response.rect;
    
    let cell_width = rect.width() / param1_range.2 as f32;
    let cell_height = rect.height() / param2_range.2 as f32;
    
    for (i, row) in heatmap_data.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            let normalized = if max_score > min_score {
                (value - min_score) / (max_score - min_score)
            } else {
                0.5
            };
            
            let color = if normalized > 0.7 {
                Color32::from_rgb(50, 200, 50)
            } else if normalized > 0.3 {
                Color32::from_rgb(200, 200, 50)
            } else {
                Color32::from_rgb(200, 50, 50)
            };
            
            let cell_rect = egui::Rect::from_min_size(
                egui::Pos2::new(
                    rect.left() + i as f32 * cell_width,
                    rect.top() + j as f32 * cell_height,
                ),
                EguiVec2::new(cell_width, cell_height),
            );
            
            painter.rect_filled(cell_rect, 0.0, color);
        }
    }
    
    ui.label(format!("Wing Span: {:.1}m - {:.1}m", param1_range.0, param1_range.1));
    ui.label(format!("Wing Chord: {:.1}m - {:.1}m", param2_range.0, param2_range.1));
    ui.label("Green = Viable, Yellow = Marginal, Red = Not Viable");
}

fn draw_real_time_plots(ui: &mut egui::Ui, history: &HistoryData) {
    ui.heading("Performance History");
    
    let plot_height = 120.0;
    
    ui.group(|ui| {
        ui.label("Power Required (W)");
        let response = ui.allocate_response(EguiVec2::new(300.0, plot_height), egui::Sense::hover());
        let painter = ui.painter_at(response.rect);
        let rect = response.rect;
        
        if !history.power_history.is_empty() {
            let max_power = history.power_history.iter().fold(0.0f32, |a, &b| a.max(b));
            let min_power = history.power_history.iter().fold(max_power, |a, &b| a.min(b));
            let range = (max_power - min_power).max(1.0);
            
            let points: Vec<egui::Pos2> = history.power_history
                .iter()
                .enumerate()
                .map(|(i, &power)| {
                    let x = rect.left() + (i as f32 / 99.0) * rect.width();
                    let y = rect.bottom() - ((power - min_power) / range) * rect.height();
                    egui::Pos2::new(x, y)
                })
                .collect();
            
            for window in points.windows(2) {
                painter.line_segment([window[0], window[1]], Stroke::new(2.0, Color32::from_rgb(200, 50, 50)));
            }
            
            painter.text(
                egui::Pos2::new(rect.right() - 40.0, rect.top() + 5.0),
                egui::Align2::RIGHT_TOP,
                format!("{:.0}W", history.power_history.back().unwrap_or(&0.0)),
                egui::FontId::proportional(10.0),
                Color32::from_rgb(200, 50, 50),
            );
        }
    });
    
    ui.group(|ui| {
        ui.label("Lift-to-Weight Ratio");
        let response = ui.allocate_response(EguiVec2::new(300.0, plot_height), egui::Sense::hover());
        let painter = ui.painter_at(response.rect);
        let rect = response.rect;
        
        if !history.lift_history.is_empty() {
            let points: Vec<egui::Pos2> = history.lift_history
                .iter()
                .enumerate()
                .map(|(i, &lift)| {
                    let x = rect.left() + (i as f32 / 99.0) * rect.width();
                    let y = rect.bottom() - (lift.min(2.0) / 2.0) * rect.height();
                    egui::Pos2::new(x, y)
                })
                .collect();
            
            for window in points.windows(2) {
                painter.line_segment([window[0], window[1]], Stroke::new(2.0, Color32::from_rgb(50, 150, 50)));
            }
            
            painter.line_segment(
                [egui::Pos2::new(rect.left(), rect.bottom() - rect.height() / 2.0),
                 egui::Pos2::new(rect.right(), rect.bottom() - rect.height() / 2.0)],
                Stroke::new(1.0, Color32::from_rgb(100, 100, 100)),
            );
            
            painter.text(
                egui::Pos2::new(rect.right() - 40.0, rect.top() + 5.0),
                egui::Align2::RIGHT_TOP,
                format!("{:.2}", history.lift_history.back().unwrap_or(&0.0)),
                egui::FontId::proportional(10.0),
                Color32::from_rgb(50, 150, 50),
            );
        }
    });
}

#[macroquad::main("Advanced Human Flight Engineering System")]
async fn main() {
    let mut state = SimulationState {
        params: FlightParams::default(),
        analysis: calculate_comprehensive_flight_analysis(&FlightParams::default()),
        history: HistoryData::new(),
        optimization_running: false,
        optimization_result: None,
        camera_rotation: 0.0,
        time: 0.0,
        selected_preset: FlightPreset::Default,
        active_tab: UITab::Configuration,
    };
    
    loop {
        clear_background(Color::from_rgba(245, 248, 252, 255));
        
        state.time += get_frame_time();
        state.camera_rotation += get_frame_time() * 0.3;
        
        state.history.update(&state.analysis, state.time);
        
        draw_main_visualization(&state);
        
        egui_macroquad::ui(|ctx| {
            egui::SidePanel::left("control_panel")
                .resizable(false)
                .default_width(380.0)
                .show(ctx, |ui| {
                    ui.heading("Flight Engineering System");
                    
                    let status_color = if state.analysis.can_sustain_level_flight && state.analysis.structural.structural_feasible {
                        Color32::from_rgb(50, 200, 50)
                    } else if state.analysis.can_takeoff {
                        Color32::from_rgb(200, 200, 50)
                    } else {
                        Color32::from_rgb(200, 50, 50)
                    };
                    
                    ui.colored_label(status_color, RichText::new(
                        if !state.analysis.structural.structural_feasible {
                            "⚠️ STRUCTURAL FAILURE"
                        } else if !state.analysis.can_takeoff {
                            "❌ TAKEOFF IMPOSSIBLE"
                        } else if !state.analysis.can_sustain_level_flight {
                            "⚡ UNSUSTAINABLE FLIGHT"
                        } else {
                            "✅ FLIGHT VIABLE"
                        }
                    ).size(16.0));
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut state.active_tab, UITab::Configuration, "Config");
                        ui.selectable_value(&mut state.active_tab, UITab::Analysis, "Analysis");
                        ui.selectable_value(&mut state.active_tab, UITab::Physics, "Physics");
                        ui.selectable_value(&mut state.active_tab, UITab::Optimization, "Optimize");
                    });
                    
                    ui.separator();
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        match state.active_tab {
                            UITab::Configuration => {
                                ui.heading("Flight Scenarios");
                                ui.label("Human-powered flight requires extreme efficiency:");
                                
                                if ui.button("🪶 Pure Human Power").clicked() {
                                    state.params = FlightParams::from_preset(FlightPreset::UltralightGlider);
                                    state.selected_preset = FlightPreset::UltralightGlider;
                                }
                                ui.label("• Like Gossamer Albatross • 12m wings • Elite athlete");
                                ui.separator();
                                
                                if ui.button("🚀 Motor-Assisted Takeoff").clicked() {
                                    state.params = FlightParams::from_preset(FlightPreset::PoweredTakeoff);
                                    state.selected_preset = FlightPreset::PoweredTakeoff;
                                }
                                ui.label("• 8kW motor for takeoff • Large wings • Hybrid power");
                                ui.separator();
                                
                                if ui.button("✈️ Long-Distance Flight").clicked() {
                                    state.params = FlightParams::from_preset(FlightPreset::SustainedFlight);
                                    state.selected_preset = FlightPreset::SustainedFlight;
                                }
                                ui.label("• 15m wings • Continuous motor • Ultra-efficient");
                                ui.separator();
                                
                                ui.horizontal(|ui| {
                                    if ui.button("Efficient").clicked() {
                                        state.params = FlightParams::from_preset(FlightPreset::MaxEfficiency);
                                        state.selected_preset = FlightPreset::MaxEfficiency;
                                    }
                                    if ui.button("Minimal").clicked() {
                                        state.params = FlightParams::from_preset(FlightPreset::MinimalWeight);
                                        state.selected_preset = FlightPreset::MinimalWeight;
                                    }
                                    if ui.button("Racing").clicked() {
                                        state.params = FlightParams::from_preset(FlightPreset::RacingConfig);
                                        state.selected_preset = FlightPreset::RacingConfig;
                                    }
                                });
                                
                                ui.label(format!("Current: {:?}", state.selected_preset));
                                ui.separator();
                                
                                ui.heading("Pilot & Power");
                                ui.add(egui::Slider::new(&mut state.params.pilot_mass, 50.0..=120.0)
                                    .text("Pilot Mass")
                                    .suffix(" kg"));
                                
                                ui.add(egui::Slider::new(&mut state.params.pilot_power_sustained, 75.0..=500.0)
                                    .text("Sustained Power")
                                    .suffix(" W"));
                                
                                ui.add(egui::Slider::new(&mut state.params.pilot_power_burst, 200.0..=1500.0)
                                    .text("Burst Power")
                                    .suffix(" W"));
                                
                                ui.add(egui::Slider::new(&mut state.params.motor_power, 0.0..=5000.0)
                                    .text("Motor Power")
                                    .suffix(" W"));
                                
                                ui.add(egui::Slider::new(&mut state.params.battery_capacity, 0.0..=2000.0)
                                    .text("Battery")
                                    .suffix(" Wh"));
                                
                                ui.separator();
                                
                                ui.heading("Wing Configuration");
                                ui.horizontal(|ui| {
                                    ui.label("Wings:");
                                    if ui.button(format!("{}", state.params.wing_count)).clicked() {
                                        state.params.wing_count = if state.params.wing_count == 2 { 4 } else { 2 };
                                    }
                                });
                                
                                ui.add(egui::Slider::new(&mut state.params.wing_span, 1.5..=8.0)
                                    .text("Wing Span")
                                    .suffix(" m"));
                                
                                ui.add(egui::Slider::new(&mut state.params.wing_chord, 0.3..=3.0)
                                    .text("Wing Chord")
                                    .suffix(" m"));
                                
                                ui.add(egui::Slider::new(&mut state.params.wing_thickness_ratio, 0.05..=0.20)
                                    .text("Thickness")
                                    .suffix(""));
                                
                                ui.horizontal(|ui| {
                                    ui.label("Wing:");
                                    ui.selectable_value(&mut state.params.wing_material, WingMaterial::Fabric, "Fabric");
                                    ui.selectable_value(&mut state.params.wing_material, WingMaterial::Carbon, "Carbon");
                                });
                                ui.horizontal(|ui| {
                                    ui.selectable_value(&mut state.params.wing_material, WingMaterial::Wood, "Wood");
                                    ui.selectable_value(&mut state.params.wing_material, WingMaterial::Aluminum, "Aluminum");
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Spar:");
                                    ui.selectable_value(&mut state.params.spar_material, SparMaterial::Carbon, "Carbon");
                                    ui.selectable_value(&mut state.params.spar_material, SparMaterial::Aluminum, "Aluminum");
                                });
                                ui.horizontal(|ui| {
                                    ui.selectable_value(&mut state.params.spar_material, SparMaterial::Wood, "Wood");
                                    ui.selectable_value(&mut state.params.spar_material, SparMaterial::Steel, "Steel");
                                });
                                
                                ui.label(format!("Wing Area: {:.1} m²", state.params.wing_area()));
                                ui.label(format!("Aspect Ratio: {:.2}", state.params.aspect_ratio()));
                                
                                ui.separator();
                                
                                ui.heading("Flight Dynamics");
                                ui.add(egui::Slider::new(&mut state.params.forward_speed, 3.0..=35.0)
                                    .text("Forward Speed")
                                    .suffix(" m/s"));
                                
                                ui.label(format!("Stall Speed: {:.1} m/s", state.analysis.stall_speed));
                                
                                ui.add(egui::Slider::new(&mut state.params.flapping_frequency, 0.0..=4.0)
                                    .text("Flapping Freq")
                                    .suffix(" Hz"));
                                
                                ui.add(egui::Slider::new(&mut state.params.flapping_amplitude, 5.0..=45.0)
                                    .text("Flapping Amp")
                                    .suffix("°"));
                                
                                ui.add(egui::Slider::new(&mut state.params.wind_speed, -10.0..=10.0)
                                    .text("Wind Speed")
                                    .suffix(" m/s"));
                            },
                            
                            UITab::Analysis => {
                                ui.heading("Performance Metrics");
                                ui.label(format!("Total Mass: {:.0} kg", state.analysis.total_mass));
                                ui.label(format!("Wing Loading: {:.1} N/m²", state.analysis.wing_loading));
                                ui.label(format!("Reynolds: {:.0}", state.analysis.reynolds_number));
                                ui.separator();
                                
                                ui.heading("Current Flight State");
                                ui.label(format!("Phase: {:?}", state.analysis.flight_phase));
                                ui.label(format!("Speed: {:.1} m/s", state.analysis.effective_airspeed));
                                if state.analysis.flight_phase != FlightPhase::OnGround {
                                    ui.label(format!("Lift Coeff: {:.3}", state.analysis.current_lift_coefficient));
                                    ui.label(format!("Drag Coeff: {:.4}", state.analysis.current_drag_coefficient));
                                    ui.label(format!("L/W Ratio: {:.2}", state.analysis.lift_force / state.analysis.weight_force));
                                }
                                
                                ui.separator();
                                
                                ui.heading("Power Analysis");
                                ui.label(format!("Total Power: {:.0} W", state.analysis.total_power_required));
                                if state.analysis.flight_phase != FlightPhase::OnGround {
                                    let total = state.analysis.total_power_required.max(1.0);
                                    ui.label(format!("Drag: {:.0}W ({:.0}%)", 
                                        state.analysis.power_to_overcome_drag,
                                        (state.analysis.power_to_overcome_drag / total) * 100.0));
                                    ui.label(format!("Flapping: {:.0}W ({:.0}%)", 
                                        state.analysis.power_for_flapping,
                                        (state.analysis.power_for_flapping / total) * 100.0));
                                }
                                
                                ui.separator();
                                
                                ui.heading("Structural");
                                let color = if state.analysis.structural.structural_feasible {
                                    Color32::from_rgb(50, 200, 50)
                                } else {
                                    Color32::from_rgb(200, 50, 50)
                                };
                                
                                ui.colored_label(color, if state.analysis.structural.structural_feasible { 
                                    "STRUCTURALLY SAFE" 
                                } else { 
                                    "STRUCTURAL FAILURE" 
                                });
                                
                                ui.label(format!("Wing Mass: {:.1} kg", state.analysis.structural.wing_mass));
                                ui.label(format!("Total Mass: {:.1} kg", state.analysis.structural.total_structural_mass));
                                ui.label(format!("Load Factor: {:.2} g", state.analysis.structural.max_load_factor));
                                
                                ui.separator();
                                
                                ui.heading("Flight Status");
                                ui.horizontal(|ui| {
                                    ui.label("Takeoff:");
                                    ui.colored_label(
                                        if state.analysis.can_takeoff { Color32::GREEN } else { Color32::RED },
                                        if state.analysis.can_takeoff { "YES" } else { "NO" }
                                    );
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Sustained:");
                                    ui.colored_label(
                                        if state.analysis.can_sustain_level_flight { Color32::GREEN } else { Color32::RED },
                                        if state.analysis.can_sustain_level_flight { "YES" } else { "NO" }
                                    );
                                });
                                
                                ui.separator();
                                draw_real_time_plots(ui, &state.history);
                            },
                            
                            UITab::Physics => {
                                draw_physics_equations(ui, &state.analysis, &state.params);
                            },
                            
                            UITab::Optimization => {
                                ui.heading("Parameter Optimization");
                                
                                if ui.button("🔍 Find Optimal Configuration").clicked() {
                                    state.optimization_running = true;
                                    state.optimization_result = Some(optimize_parameters(&state.params));
                                    state.optimization_running = false;
                                }
                                
                                if let Some(ref optimal) = state.optimization_result {
                                    ui.separator();
                                    ui.heading("Optimization Results");
                                    ui.label(format!("Wing Span: {:.1}m", optimal.wing_span));
                                    ui.label(format!("Wing Chord: {:.1}m", optimal.wing_chord));
                                    ui.label(format!("Speed: {:.1}m/s", optimal.forward_speed));
                                    ui.label(format!("Motor: {:.0}W", optimal.motor_power));
                                    
                                    if ui.button("✅ Apply Optimal Parameters").clicked() {
                                        state.params = optimal.clone();
                                        state.selected_preset = FlightPreset::Default;
                                    }
                                }
                                
                                ui.separator();
                                
                                draw_parameter_heatmap(ui, &state.params);
                            }
                        }
                    });
                    
                    state.analysis = calculate_comprehensive_flight_analysis(&state.params);
                });
        });
        
        egui_macroquad::draw();
        
        next_frame().await
    }
}
