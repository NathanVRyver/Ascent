use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

#[derive(Clone, Debug)]
struct FlightParams {
    // Human characteristics
    pilot_mass: f32, // kg
    pilot_power_sustained: f32, // watts (typical human: 75-400W)
    pilot_power_burst: f32, // watts (short duration, for takeoff)
    
    // Wing configuration
    wing_count: u32, // 2 or 4 wings
    wing_span: f32, // m (per wing)
    wing_chord: f32, // m (average chord length)
    wing_thickness_ratio: f32, // t/c ratio (affects strength vs weight)
    
    // Wing structure
    wing_material: WingMaterial,
    spar_material: SparMaterial,
    wing_safety_factor: f32, // structural safety factor
    
    // Propulsion
    motor_power: f32, // watts (for takeoff assistance)
    motor_mass: f32, // kg
    battery_capacity: f32, // Wh
    motor_efficiency: f32, // 0-1
    
    // Aerodynamics
    airfoil_cl_max: f32, // maximum lift coefficient before stall
    airfoil_cd_min: f32, // minimum drag coefficient
    oswald_efficiency: f32, // induced drag efficiency (0.7-0.95)
    
    // Flight conditions
    forward_speed: f32, // m/s
    flapping_frequency: f32, // Hz
    flapping_amplitude: f32, // degrees
    air_density: f32, // kg/m³
    wind_speed: f32, // m/s headwind/tailwind
}

#[derive(Clone, Debug)]
enum WingMaterial {
    Fabric, // ripstop nylon, mylar
    Carbon, // carbon fiber
    Wood,   // balsa/spruce
    Aluminum, // thin sheet
}

#[derive(Clone, Debug)]
enum SparMaterial {
    Carbon,
    Aluminum,
    Wood,
    Steel,
}

impl FlightParams {
    fn wing_area(&self) -> f32 {
        self.wing_count as f32 * self.wing_span * self.wing_chord
    }
    
    fn aspect_ratio(&self) -> f32 {
        self.wing_span / self.wing_chord
    }
}

impl Default for FlightParams {
    fn default() -> Self {
        Self {
            pilot_mass: 80.0,
            pilot_power_sustained: 200.0, // watts - realistic human output
            pilot_power_burst: 600.0,
            
            wing_count: 4,
            wing_span: 3.0,
            wing_chord: 1.5,
            wing_thickness_ratio: 0.12,
            
            wing_material: WingMaterial::Fabric,
            spar_material: SparMaterial::Carbon,
            wing_safety_factor: 2.0,
            
            motor_power: 2000.0, // 2kW for takeoff
            motor_mass: 8.0, // kg
            battery_capacity: 500.0, // Wh
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
    wing_mass: f32, // kg per wing
    spar_mass: f32, // kg per wing
    total_structural_mass: f32,
    max_load_factor: f32, // g's before structural failure
    wing_deflection: f32, // tip deflection in meters
    critical_flutter_speed: f32, // m/s
    structural_feasible: bool,
}

#[derive(Clone)]
struct FlightAnalysis {
    // Forces
    lift_force: f32,
    weight_force: f32,
    drag_force: f32,
    
    // Power breakdown
    power_to_overcome_drag: f32,
    power_for_flapping: f32,
    power_for_climb: f32,
    total_power_required: f32,
    
    // Performance metrics
    lift_to_weight_ratio: f32,
    wing_loading: f32, // N/m²
    power_loading: f32, // W/N
    
    // Flight phases
    can_takeoff: bool,
    can_sustain_level_flight: bool,
    can_climb: f32, // m/s climb rate
    stall_speed: f32,
    
    // Motor analysis
    motor_flight_time: f32, // minutes
    takeoff_distance: f32, // meters
    
    // Structural
    structural: StructuralAnalysis,
    
    // Environmental
    reynolds_number: f32,
    flutter_margin: f32,
}

fn calculate_structural_properties(params: &FlightParams) -> StructuralAnalysis {
    let wing_area_single = params.wing_span * params.wing_chord;
    
    // Material properties
    let (wing_density, _wing_strength) = match params.wing_material {
        WingMaterial::Fabric => (0.2, 50_000.0), // kg/m², Pa
        WingMaterial::Carbon => (1.6, 600_000_000.0),
        WingMaterial::Wood => (0.6, 40_000_000.0),
        WingMaterial::Aluminum => (2.7, 270_000_000.0),
    };
    
    let (spar_density, spar_strength) = match params.spar_material {
        SparMaterial::Carbon => (1600.0, 3_500_000_000.0), // kg/m³, Pa
        SparMaterial::Aluminum => (2700.0, 276_000_000.0),
        SparMaterial::Wood => (500.0, 100_000_000.0),
        SparMaterial::Steel => (7850.0, 250_000_000.0),
    };
    
    // Wing mass calculation (simplified)
    let wing_skin_mass = wing_area_single * wing_density * 0.002; // 2mm average thickness
    let spar_volume = params.wing_span * 0.05 * 0.02; // simplified box beam
    let spar_mass = spar_volume * spar_density / 1000.0;
    let wing_mass = wing_skin_mass + spar_mass + 2.0; // +2kg for control mechanisms
    
    let total_structural_mass = wing_mass * params.wing_count as f32 + params.motor_mass;
    
    // Load factor calculation
    let dynamic_pressure = 0.5 * params.air_density * params.forward_speed.powi(2);
    let max_lift_per_wing = params.airfoil_cl_max * dynamic_pressure * wing_area_single;
    let total_weight = (params.pilot_mass + total_structural_mass) * 9.81;
    let max_load_factor = (max_lift_per_wing * params.wing_count as f32) / total_weight;
    
    // Wing deflection (simplified beam theory)
    let moment_of_inertia = 0.05 * 0.02_f32.powi(3) / 12.0; // simplified I-beam
    let distributed_load = max_lift_per_wing / params.wing_span;
    let wing_deflection = (distributed_load * params.wing_span.powi(4)) / (8.0 * 200_000_000_000.0 * moment_of_inertia);
    
    // Flutter speed (simplified)
    let critical_flutter_speed = (spar_strength / (spar_density * params.wing_span.powi(2))).sqrt() * 0.1;
    
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
    
    // Total system mass
    let total_mass = params.pilot_mass + structural.total_structural_mass;
    let weight_force = total_mass * 9.81;
    
    // Aerodynamic calculations
    let wing_area = params.wing_area();
    let effective_airspeed = params.forward_speed - params.wind_speed; // headwind reduces effective speed
    let dynamic_pressure = 0.5 * params.air_density * effective_airspeed.powi(2);
    
    // Stall speed calculation
    let stall_speed = (2.0 * weight_force / (params.air_density * wing_area * params.airfoil_cl_max)).sqrt();
    
    // Lift calculation with flapping enhancement
    let reduced_frequency = params.flapping_frequency * params.wing_span / effective_airspeed.max(0.1);
    let flapping_boost = if params.flapping_frequency > 0.1 {
        1.0 + (reduced_frequency * 0.3 * (params.flapping_amplitude / 45.0)).min(0.8)
    } else {
        1.0
    };
    
    // Current lift coefficient
    let current_cl = if effective_airspeed < stall_speed {
        0.3 // post-stall
    } else {
        let base_cl = weight_force / (dynamic_pressure * wing_area);
        (base_cl * flapping_boost).min(params.airfoil_cl_max)
    };
    
    let lift_force = current_cl * dynamic_pressure * wing_area * flapping_boost;
    
    // Drag calculation with induced drag
    let induced_drag_coeff = current_cl.powi(2) / (std::f32::consts::PI * params.aspect_ratio() * params.oswald_efficiency);
    let total_drag_coeff = params.airfoil_cd_min + induced_drag_coeff + 
                          if params.wing_count == 4 { 0.02 } else { 0.0 }; // interference drag
    let drag_force = total_drag_coeff * dynamic_pressure * wing_area;
    
    // Power calculations
    let power_to_overcome_drag = drag_force * effective_airspeed;
    let power_for_flapping = if params.flapping_frequency > 0.1 {
        // Simplified flapping power
        let flapping_velocity = 2.0 * std::f32::consts::PI * params.flapping_frequency * 
                               params.wing_span * 0.7 * (params.flapping_amplitude.to_radians().sin());
        total_mass * 9.81 * flapping_velocity * 0.2 * params.wing_count as f32
    } else {
        0.0
    };
    
    let power_for_climb = 0.0; // assume level flight for now
    let total_power_required = power_to_overcome_drag + power_for_flapping + power_for_climb;
    
    // Flight capability analysis
    let can_takeoff = (params.pilot_power_burst + params.motor_power * params.motor_efficiency) > 
                     (total_power_required * 1.5) && lift_force > weight_force * 0.8;
    
    let can_sustain_level_flight = params.pilot_power_sustained > total_power_required && 
                                  lift_force >= weight_force && 
                                  effective_airspeed > stall_speed;
    
    let excess_power = params.pilot_power_sustained - total_power_required;
    let can_climb = if excess_power > 0.0 { excess_power / weight_force } else { -0.5 };
    
    // Motor analysis
    let motor_flight_time = params.battery_capacity / (params.motor_power / 1000.0) * 60.0; // minutes
    let takeoff_distance = if can_takeoff {
        let acceleration = ((params.pilot_power_burst + params.motor_power * params.motor_efficiency) / effective_airspeed - drag_force) / total_mass;
        effective_airspeed.powi(2) / (2.0 * acceleration.max(0.1))
    } else {
        f32::INFINITY
    };
    
    // Environmental factors
    let chord_length = params.wing_chord;
    let reynolds_number = effective_airspeed * chord_length / 1.5e-5; // kinematic viscosity of air
    let flutter_margin = structural.critical_flutter_speed / effective_airspeed.max(1.0);
    
    FlightAnalysis {
        lift_force,
        weight_force,
        drag_force,
        power_to_overcome_drag,
        power_for_flapping,
        power_for_climb,
        total_power_required,
        lift_to_weight_ratio: lift_force / weight_force,
        wing_loading: weight_force / wing_area,
        power_loading: total_power_required / weight_force,
        can_takeoff,
        can_sustain_level_flight,
        can_climb,
        stall_speed,
        motor_flight_time,
        takeoff_distance,
        structural,
        reynolds_number,
        flutter_margin,
    }
}

fn draw_enhanced_performance_chart(x: f32, y: f32, width: f32, height: f32, params: &FlightParams, metric: &str, x_unit: &str, y_unit: &str) {
    // Professional engineering chart background
    draw_rectangle(x, y, width, height, Color::from_rgba(252, 253, 255, 255));
    draw_rectangle_lines(x, y, width, height, 2.0, Color::from_rgba(30, 40, 60, 255));
    
    // Enhanced grid with more lines
    for i in 0..12 {
        let grid_y = y + (i as f32 / 11.0) * height;
        let alpha = if i % 2 == 0 { 120 } else { 60 };
        draw_line(x, grid_y, x + width, grid_y, 0.5, Color::from_rgba(180, 190, 200, alpha));
    }
    for i in 0..15 {
        let grid_x = x + (i as f32 / 14.0) * width;
        let alpha = if i % 3 == 0 { 120 } else { 60 };
        draw_line(grid_x, y, grid_x, y + height, 0.5, Color::from_rgba(180, 190, 200, alpha));
    }
    
    // Enhanced chart title with units
    draw_text(metric, x + 8.0, y + 18.0, 14.0, Color::from_rgba(30, 40, 60, 255));
    draw_text(&format!("Y: {} | X: {}", y_unit, x_unit), x + 8.0, y + 35.0, 10.0, Color::from_rgba(80, 90, 110, 255));
    
    // Generate analysis data
    let mut data = Vec::new();
    let (x_param, x_range) = match metric {
        "Power vs Speed" => ("speed", 5.0..30.0),
        "Lift vs Wing Area" => ("area", 5.0..25.0),
        "Structural Mass vs Span" => ("span", 2.0..8.0),
        _ => ("speed", 5.0..30.0),
    };
    
    for i in 0..25 {
        let x_val = x_range.start + (x_range.end - x_range.start) * (i as f32 / 24.0);
        let mut test_params = params.clone();
        
        match x_param {
            "speed" => test_params.forward_speed = x_val,
            "area" => test_params.wing_chord = x_val / test_params.wing_count as f32 / test_params.wing_span,
            "span" => test_params.wing_span = x_val,
            _ => {}
        }
        
        let analysis = calculate_comprehensive_flight_analysis(&test_params);
        
        let y_val = match metric {
            "Power vs Speed" => analysis.total_power_required / 1000.0,
            "Lift vs Wing Area" => analysis.lift_to_weight_ratio,
            "Structural Mass vs Span" => analysis.structural.total_structural_mass,
            _ => analysis.total_power_required / 1000.0,
        };
        
        data.push((x_val, y_val));
    }
    
    if !data.is_empty() {
        let max_y = data.iter().map(|(_, y)| *y).fold(0.0f32, f32::max);
        let min_y = data.iter().map(|(_, y)| *y).fold(max_y, f32::min).min(0.0);
        let y_range = (max_y - min_y).max(0.1);
        
        let line_color = match metric {
            "Power vs Speed" => Color::from_rgba(220, 60, 60, 255),
            "Lift vs Wing Area" => Color::from_rgba(60, 180, 60, 255),
            "Structural Mass vs Span" => Color::from_rgba(60, 120, 220, 255),
            _ => Color::from_rgba(100, 100, 100, 255),
        };
        
        // Plot data with enhanced styling
        for i in 1..data.len() {
            let x1 = x + ((data[i-1].0 - x_range.start) / (x_range.end - x_range.start)) * (width - 25.0) + 15.0;
            let y1 = y + height - ((data[i-1].1 - min_y) / y_range) * (height - 50.0) - 25.0;
            let x2 = x + ((data[i].0 - x_range.start) / (x_range.end - x_range.start)) * (width - 25.0) + 15.0;
            let y2 = y + height - ((data[i].1 - min_y) / y_range) * (height - 50.0) - 25.0;
            
            draw_line(x1, y1, x2, y2, 2.5, line_color);
            if i % 4 == 0 {
                draw_circle(x2, y2, 2.0, line_color);
            }
        }
        
        // Enhanced axis labels with units
        draw_text(&format!("{:.2}", min_y), x - 35.0, y + height - 15.0, 9.0, Color::from_rgba(60, 70, 90, 255));
        draw_text(&format!("{:.2}", max_y), x - 35.0, y + 35.0, 9.0, Color::from_rgba(60, 70, 90, 255));
        draw_text(&format!("{:.1}", x_range.start), x + 10.0, y + height + 15.0, 9.0, Color::from_rgba(60, 70, 90, 255));
        draw_text(&format!("{:.1}", x_range.end), x + width - 25.0, y + height + 15.0, 9.0, Color::from_rgba(60, 70, 90, 255));
    }
}

fn draw_enhanced_structural_panel(x: f32, y: f32, width: f32, height: f32, analysis: &FlightAnalysis, params: &FlightParams) {
    draw_rectangle(x, y, width, height, Color::from_rgba(252, 253, 255, 255));
    draw_rectangle_lines(x, y, width, height, 2.0, Color::from_rgba(30, 40, 60, 255));
    draw_text("STRUCTURAL INTEGRITY", x + 8.0, y + 18.0, 14.0, Color::from_rgba(30, 40, 60, 255));
    
    let struct_color = if analysis.structural.structural_feasible { 
        Color::from_rgba(40, 140, 40, 255) 
    } else { 
        Color::from_rgba(200, 40, 40, 255) 
    };
    
    let wing_material_str = match params.wing_material {
        WingMaterial::Fabric => "Fabric",
        WingMaterial::Carbon => "Carbon",
        WingMaterial::Wood => "Wood", 
        WingMaterial::Aluminum => "Aluminum",
    };
    
    let spar_material_str = match params.spar_material {
        SparMaterial::Carbon => "Carbon",
        SparMaterial::Aluminum => "Aluminum",
        SparMaterial::Wood => "Wood",
        SparMaterial::Steel => "Steel",
    };
    
    draw_text(&format!("Materials: {} wings, {} spars", wing_material_str, spar_material_str), 
        x + 8.0, y + 38.0, 10.0, Color::from_rgba(60, 70, 90, 255));
    draw_text(&format!("Total Wing Mass: {:.1} kg", analysis.structural.wing_mass * params.wing_count as f32), 
        x + 8.0, y + 55.0, 11.0, struct_color);
    draw_text(&format!("Max Load Factor: {:.2} g", analysis.structural.max_load_factor), 
        x + 8.0, y + 72.0, 11.0, struct_color);
    draw_text(&format!("Wing Deflection: {:.1} mm", analysis.structural.wing_deflection * 1000.0), 
        x + 8.0, y + 89.0, 11.0, struct_color);
    draw_text(&format!("Flutter Speed: {:.1} m/s", analysis.structural.critical_flutter_speed), 
        x + 8.0, y + 106.0, 11.0, struct_color);
    draw_text(&format!("Safety Factor: {:.1}", params.wing_safety_factor), 
        x + 8.0, y + 123.0, 11.0, Color::from_rgba(60, 70, 90, 255));
    
    let status_text = if analysis.structural.structural_feasible { "✅ SAFE" } else { "❌ UNSAFE" };
    draw_text(status_text, x + 8.0, y + 145.0, 16.0, struct_color);
}

fn draw_power_breakdown_panel(x: f32, y: f32, width: f32, height: f32, analysis: &FlightAnalysis) {
    draw_rectangle(x, y, width, height, Color::from_rgba(252, 253, 255, 255));
    draw_rectangle_lines(x, y, width, height, 2.0, Color::from_rgba(30, 40, 60, 255));
    draw_text("POWER ANALYSIS BREAKDOWN", x + 8.0, y + 18.0, 14.0, Color::from_rgba(30, 40, 60, 255));
    
    let total_power = analysis.total_power_required;
    let drag_pct = (analysis.power_to_overcome_drag / total_power * 100.0).max(0.0);
    let flap_pct = (analysis.power_for_flapping / total_power * 100.0).max(0.0);
    let climb_pct = (analysis.power_for_climb / total_power * 100.0).max(0.0);
    
    // Power breakdown bars
    let bar_width = width - 200.0;
    let bar_y = y + 45.0;
    
    // Drag power bar
    if drag_pct > 0.0 {
        draw_rectangle(x + 150.0, bar_y, bar_width * (drag_pct / 100.0), 15.0, Color::from_rgba(220, 80, 80, 255));
    }
    draw_text(&format!("Drag Power: {:.1}W ({:.0}%)", analysis.power_to_overcome_drag, drag_pct), 
        x + 8.0, bar_y + 12.0, 11.0, Color::from_rgba(60, 70, 90, 255));
    
    // Flapping power bar
    if flap_pct > 0.0 {
        draw_rectangle(x + 150.0, bar_y + 20.0, bar_width * (flap_pct / 100.0), 15.0, Color::from_rgba(80, 160, 80, 255));
    }
    draw_text(&format!("Flapping Power: {:.1}W ({:.0}%)", analysis.power_for_flapping, flap_pct), 
        x + 8.0, bar_y + 32.0, 11.0, Color::from_rgba(60, 70, 90, 255));
    
    // Climb power bar
    if climb_pct > 0.0 {
        draw_rectangle(x + 150.0, bar_y + 40.0, bar_width * (climb_pct / 100.0), 15.0, Color::from_rgba(80, 80, 220, 255));
    }
    draw_text(&format!("Climb Power: {:.1}W ({:.0}%)", analysis.power_for_climb, climb_pct), 
        x + 8.0, bar_y + 52.0, 11.0, Color::from_rgba(60, 70, 90, 255));
    
    // Total power summary
    draw_text(&format!("TOTAL REQUIRED: {:.1}W ({:.2}kW)", total_power, total_power / 1000.0), 
        x + 8.0, y + 105.0, 13.0, Color::from_rgba(40, 60, 80, 255));
}

fn draw_performance_chart(x: f32, y: f32, width: f32, height: f32, params: &FlightParams, metric: &str) {
    // Professional engineering chart background
    draw_rectangle(x, y, width, height, Color::from_rgba(248, 248, 252, 255));
    draw_rectangle_lines(x, y, width, height, 1.0, Color::from_rgba(40, 40, 60, 255));
    
    // Grid
    for i in 0..8 {
        let grid_y = y + (i as f32 / 7.0) * height;
        draw_line(x, grid_y, x + width, grid_y, 0.5, Color::from_rgba(200, 200, 210, 255));
    }
    for i in 0..10 {
        let grid_x = x + (i as f32 / 9.0) * width;
        draw_line(grid_x, y, grid_x, y + height, 0.5, Color::from_rgba(200, 200, 210, 255));
    }
    
    // Chart title
    draw_text(metric, x + 5.0, y + 15.0, 12.0, Color::from_rgba(40, 40, 60, 255));
    
    // Generate analysis data
    let mut data = Vec::new();
    let (x_param, x_range) = match metric {
        "Power vs Speed" => ("speed", 5.0..30.0),
        "Lift vs Wing Area" => ("area", 5.0..25.0),
        "Structural Mass vs Span" => ("span", 2.0..8.0),
        _ => ("speed", 5.0..30.0),
    };
    
    for i in 0..20 {
        let x_val = x_range.start + (x_range.end - x_range.start) * (i as f32 / 19.0);
        let mut test_params = params.clone();
        
        match x_param {
            "speed" => test_params.forward_speed = x_val,
            "area" => test_params.wing_chord = x_val / test_params.wing_count as f32 / test_params.wing_span,
            "span" => test_params.wing_span = x_val,
            _ => {}
        }
        
        let analysis = calculate_comprehensive_flight_analysis(&test_params);
        
        let y_val = match metric {
            "Power vs Speed" => analysis.total_power_required / 1000.0,
            "Lift vs Wing Area" => analysis.lift_to_weight_ratio,
            "Structural Mass vs Span" => analysis.structural.total_structural_mass,
            _ => analysis.total_power_required / 1000.0,
        };
        
        data.push((x_val, y_val));
    }
    
    if !data.is_empty() {
        let max_y = data.iter().map(|(_, y)| *y).fold(0.0f32, f32::max);
        let min_y = data.iter().map(|(_, y)| *y).fold(max_y, f32::min).min(0.0);
        let y_range = (max_y - min_y).max(0.1);
        
        let line_color = match metric {
            "Power vs Speed" => Color::from_rgba(200, 60, 60, 255),
            "Lift vs Wing Area" => Color::from_rgba(60, 150, 60, 255),
            "Structural Mass vs Span" => Color::from_rgba(60, 60, 200, 255),
            _ => Color::from_rgba(100, 100, 100, 255),
        };
        
        // Plot data
        for i in 1..data.len() {
            let x1 = x + ((data[i-1].0 - x_range.start) / (x_range.end - x_range.start)) * (width - 20.0) + 10.0;
            let y1 = y + height - ((data[i-1].1 - min_y) / y_range) * (height - 25.0) - 15.0;
            let x2 = x + ((data[i].0 - x_range.start) / (x_range.end - x_range.start)) * (width - 20.0) + 10.0;
            let y2 = y + height - ((data[i].1 - min_y) / y_range) * (height - 25.0) - 15.0;
            
            draw_line(x1, y1, x2, y2, 2.0, line_color);
            if i % 3 == 0 {
                draw_circle(x2, y2, 1.5, line_color);
            }
        }
        
        // Axis labels
        draw_text(&format!("{:.1}", min_y), x - 25.0, y + height - 10.0, 8.0, Color::from_rgba(80, 80, 100, 255));
        draw_text(&format!("{:.1}", max_y), x - 25.0, y + 20.0, 8.0, Color::from_rgba(80, 80, 100, 255));
    }
}

#[macroquad::main("Advanced Human Flight Engineering System")]
async fn main() {
    let mut params = FlightParams::default();
    let mut analysis = calculate_comprehensive_flight_analysis(&params);
    
    loop {
        clear_background(Color::from_rgba(248, 250, 252, 255));
        
        // Enhanced Header with more information
        draw_rectangle(0.0, 0.0, screen_width(), 80.0, Color::from_rgba(20, 30, 45, 255));
        draw_text("ADVANCED HUMAN FLIGHT ENGINEERING SYSTEM", 20.0, 25.0, 20.0, WHITE);
        draw_text("Comprehensive Aerodynamic & Structural Analysis Platform", 20.0, 45.0, 12.0, Color::from_rgba(180, 200, 220, 255));
        
        // Current configuration summary in header
        let config_text = format!("Configuration: {}kg pilot | {} wings | {:.1}m span | {:.0}W motor", 
            params.pilot_mass, params.wing_count, params.wing_span, params.motor_power);
        draw_text(&config_text, 20.0, 65.0, 10.0, Color::from_rgba(160, 180, 200, 255));
        
        // Enhanced Critical status panel with more metrics
        let status_y = 90.0;
        let status_height = 90.0;
        let status_color = if analysis.can_sustain_level_flight && analysis.structural.structural_feasible {
            Color::from_rgba(220, 255, 220, 255)
        } else if analysis.can_takeoff {
            Color::from_rgba(255, 250, 200, 255)
        } else {
            Color::from_rgba(255, 220, 220, 255)
        };
        
        draw_rectangle(20.0, status_y, screen_width() - 40.0, status_height, status_color);
        draw_rectangle_lines(20.0, status_y, screen_width() - 40.0, status_height, 2.0, Color::from_rgba(80, 100, 120, 255));
        
        let flight_status = if !analysis.structural.structural_feasible {
            "⚠️ STRUCTURAL FAILURE RISK"
        } else if !analysis.can_takeoff {
            "❌ TAKEOFF NOT POSSIBLE"
        } else if !analysis.can_sustain_level_flight {
            "⚡ LEVEL FLIGHT NOT SUSTAINABLE"
        } else {
            "✅ FLIGHT CONFIGURATION VIABLE"
        };
        
        draw_text(flight_status, 40.0, status_y + 25.0, 22.0, Color::from_rgba(40, 60, 80, 255));
        
        // Key metrics in status panel
        draw_text(&format!("Lift-to-Weight: {:.3} | Power Required: {:.1}kW | Total Mass: {:.1}kg", 
            analysis.lift_to_weight_ratio, analysis.total_power_required / 1000.0, 
            params.pilot_mass + analysis.structural.total_structural_mass), 
            40.0, status_y + 50.0, 13.0, Color::from_rgba(60, 80, 100, 255));
            
        draw_text(&format!("Stall Speed: {:.1}m/s | Wing Loading: {:.1}N/m² | Aspect Ratio: {:.2}", 
            analysis.stall_speed, analysis.wing_loading, params.aspect_ratio()), 
            40.0, status_y + 70.0, 13.0, Color::from_rgba(60, 80, 100, 255));
        
        // Enhanced Left panel with better organization and more info
        root_ui().window(hash!(), vec2(20., 190.), vec2(320., 580.), |ui| {
            ui.label(None, "┌─ PILOT & PROPULSION CHARACTERISTICS ─┐");
            ui.separator();
            
            ui.label(None, &format!("Pilot Mass: {:.0} kg (affects weight & power-to-weight)", params.pilot_mass));
            ui.label(None, "  Range: 50-120kg (typical adult range)");
            let old = params.pilot_mass;
            widgets::Slider::new(hash!(), 50.0..120.0).ui(ui, &mut params.pilot_mass);
            if old != params.pilot_mass { analysis = calculate_comprehensive_flight_analysis(&params); }
            
            ui.label(None, &format!("Sustained Power: {:.0} W (continuous output capability)", params.pilot_power_sustained));
            ui.label(None, "  Range: 75-500W (75W=sedentary, 400W=elite cyclist)");
            let old = params.pilot_power_sustained;
            widgets::Slider::new(hash!(), 75.0..500.0).ui(ui, &mut params.pilot_power_sustained);
            if old != params.pilot_power_sustained { analysis = calculate_comprehensive_flight_analysis(&params); }
            
            ui.label(None, &format!("Burst Power: {:.0} W (short-term peak for takeoff)", params.pilot_power_burst));
            ui.label(None, "  Range: 200-1500W (typically 2-3x sustained)");
            let old = params.pilot_power_burst;
            widgets::Slider::new(hash!(), 200.0..1500.0).ui(ui, &mut params.pilot_power_burst);
            if old != params.pilot_power_burst { analysis = calculate_comprehensive_flight_analysis(&params); }
            
            ui.label(None, &format!("Motor Power: {:.0} W (electric assist)", params.motor_power));
            ui.label(None, "  Range: 0-5000W (0=human-only, 2000W=e-bike motor)");
            let old = params.motor_power;
            widgets::Slider::new(hash!(), 0.0..5000.0).ui(ui, &mut params.motor_power);
            if old != params.motor_power { analysis = calculate_comprehensive_flight_analysis(&params); }
            
            ui.separator();
            ui.label(None, "┌─ WING CONFIGURATION & GEOMETRY ─┐");
            
            ui.label(None, &format!("Wing Count: {} (affects lift area & complexity)", params.wing_count));
            if ui.button(None, if params.wing_count == 2 { "Switch to 4 Wings (dragonfly)" } else { "Switch to 2 Wings (bird)" }) {
                params.wing_count = if params.wing_count == 2 { 4 } else { 2 };
                analysis = calculate_comprehensive_flight_analysis(&params);
            }
            
            ui.label(None, &format!("Wing Span: {:.2} m (tip-to-tip length per wing)", params.wing_span));
            ui.label(None, &format!("  Total Area: {:.1}m² | Aspect Ratio: {:.2}", params.wing_area(), params.aspect_ratio()));
            let old = params.wing_span;
            widgets::Slider::new(hash!(), 1.5..8.0).ui(ui, &mut params.wing_span);
            if old != params.wing_span { analysis = calculate_comprehensive_flight_analysis(&params); }
            
            ui.label(None, &format!("Wing Chord: {:.2} m (front-to-back width)", params.wing_chord));
            ui.label(None, "  Range: 0.3-3.0m (affects stall characteristics)");
            let old = params.wing_chord;
            widgets::Slider::new(hash!(), 0.3..3.0).ui(ui, &mut params.wing_chord);
            if old != params.wing_chord { analysis = calculate_comprehensive_flight_analysis(&params); }
            
            ui.separator();
            ui.label(None, "┌─ FLIGHT CONDITIONS & DYNAMICS ─┐");
            
            ui.label(None, &format!("Forward Speed: {:.1} m/s ({:.1} mph)", params.forward_speed, params.forward_speed * 2.237));
            ui.label(None, &format!("  Must exceed stall speed: {:.1} m/s", analysis.stall_speed));
            let old = params.forward_speed;
            widgets::Slider::new(hash!(), 3.0..35.0).ui(ui, &mut params.forward_speed);
            if old != params.forward_speed { analysis = calculate_comprehensive_flight_analysis(&params); }
            
            ui.label(None, &format!("Flapping Frequency: {:.1} Hz (beats per second)", params.flapping_frequency));
            ui.label(None, "  Range: 0-4Hz (0=glider, 2Hz=large bird, 4Hz=small bird)");
            let old = params.flapping_frequency;
            widgets::Slider::new(hash!(), 0.0..4.0).ui(ui, &mut params.flapping_frequency);
            if old != params.flapping_frequency { analysis = calculate_comprehensive_flight_analysis(&params); }
            
            ui.label(None, &format!("Flapping Amplitude: {:.0}° (wing stroke angle)", params.flapping_amplitude));
            ui.label(None, "  Range: 5-45° (affects power & lift generation)");
            let old = params.flapping_amplitude;
            widgets::Slider::new(hash!(), 5.0..45.0).ui(ui, &mut params.flapping_amplitude);
            if old != params.flapping_amplitude { analysis = calculate_comprehensive_flight_analysis(&params); }
            
            ui.label(None, &format!("Wind Speed: {:.1} m/s (headwind/tailwind)", params.wind_speed));
            ui.label(None, "  Range: -10 to +10 m/s (negative=tailwind)");
            let old = params.wind_speed;
            widgets::Slider::new(hash!(), -10.0..10.0).ui(ui, &mut params.wind_speed);
            if old != params.wind_speed { analysis = calculate_comprehensive_flight_analysis(&params); }
        });
        
        // Enhanced Performance charts with better labeling
        draw_enhanced_performance_chart(350.0, 190.0, 220.0, 160.0, &params, "Power vs Speed", "m/s", "kW");
        draw_enhanced_performance_chart(580.0, 190.0, 220.0, 160.0, &params, "Lift vs Wing Area", "m²", "L/W");
        draw_enhanced_performance_chart(350.0, 360.0, 220.0, 160.0, &params, "Structural Mass vs Span", "m", "kg");
        
        // Enhanced Structural analysis panel with more details
        draw_enhanced_structural_panel(580.0, 360.0, 220.0, 160.0, &analysis, &params);
        
        // Enhanced Power breakdown panel
        draw_power_breakdown_panel(350.0, 530.0, 450.0, 120.0, &analysis);
        
        // Enhanced Bottom status bar with detailed diagnostics
        let bottom_y = screen_height() - 100.0;
        draw_rectangle(0.0, bottom_y, screen_width(), 100.0, Color::from_rgba(235, 240, 245, 255));
        draw_line(0.0, bottom_y, screen_width(), bottom_y, 2.0, Color::from_rgba(100, 120, 140, 255));
        
        draw_text("FLIGHT ANALYSIS DIAGNOSTICS", 30.0, bottom_y + 20.0, 16.0, Color::from_rgba(40, 60, 80, 255));
        
        let critical_issues = vec![
            if analysis.total_power_required > params.pilot_power_sustained { "❌ Insufficient sustained power output" } else { "" },
            if params.forward_speed < analysis.stall_speed { "⚠️ Airspeed below stall threshold" } else { "" },
            if !analysis.structural.structural_feasible { "❌ Structural integrity compromised" } else { "" },
            if analysis.takeoff_distance > 100.0 { "⚠️ Excessive takeoff distance required" } else { "" },
            if analysis.motor_flight_time < 5.0 && params.motor_power > 0.0 { "⚡ Limited battery endurance" } else { "" },
        ];
        
        let issues: Vec<&str> = critical_issues.into_iter().filter(|&s| !s.is_empty()).collect();
        
        if issues.is_empty() {
            draw_text("✅ All systems nominal - Configuration viable for sustained human flight", 
                30.0, bottom_y + 40.0, 14.0, Color::from_rgba(40, 120, 40, 255));
            draw_text(&format!("Performance: {:.2}m/s climb | {:.1}min motor endurance | Reynolds: {:.0}K | Flutter margin: {:.1}x", 
                analysis.can_climb, analysis.motor_flight_time, analysis.reynolds_number / 1000.0, analysis.flutter_margin), 
                30.0, bottom_y + 60.0, 12.0, Color::from_rgba(60, 80, 100, 255));
        } else {
            draw_text("Critical Engineering Issues Identified:", 30.0, bottom_y + 40.0, 14.0, Color::from_rgba(150, 50, 50, 255));
            for (i, issue) in issues.iter().take(3).enumerate() {
                draw_text(issue, 30.0, bottom_y + 60.0 + (i as f32 * 15.0), 11.0, Color::from_rgba(120, 40, 40, 255));
            }
        }
        
        next_frame().await
    }
}