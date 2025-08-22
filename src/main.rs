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
    lift_force: f32,
    weight_force: f32,
    drag_force: f32,
    
    power_to_overcome_drag: f32,
    power_for_flapping: f32,
    power_for_climb: f32,
    total_power_required: f32,
    
    lift_to_weight_ratio: f32,
    wing_loading: f32,
    power_loading: f32,
    
    can_takeoff: bool,
    can_sustain_level_flight: bool,
    can_climb: f32,
    stall_speed: f32,
    
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
    show_3d_view: bool,
    camera_rotation: f32,
    time: f32,
}

struct HistoryData {
    power_history: VecDeque<f32>,
    lift_history: VecDeque<f32>,
    speed_history: VecDeque<f32>,
    time_stamps: VecDeque<f32>,
}

impl HistoryData {
    fn new() -> Self {
        Self {
            power_history: VecDeque::with_capacity(100),
            lift_history: VecDeque::with_capacity(100),
            speed_history: VecDeque::with_capacity(100),
            time_stamps: VecDeque::with_capacity(100),
        }
    }
    
    fn update(&mut self, analysis: &FlightAnalysis, time: f32) {
        if self.power_history.len() >= 100 {
            self.power_history.pop_front();
            self.lift_history.pop_front();
            self.speed_history.pop_front();
            self.time_stamps.pop_front();
        }
        
        self.power_history.push_back(analysis.total_power_required);
        self.lift_history.push_back(analysis.lift_to_weight_ratio);
        self.speed_history.push_back(analysis.stall_speed);
        self.time_stamps.push_back(time);
    }
}

fn calculate_structural_properties(params: &FlightParams) -> StructuralAnalysis {
    let wing_area_single = params.wing_span * params.wing_chord;
    
    let (wing_density, _wing_strength) = match params.wing_material {
        WingMaterial::Fabric => (0.2, 50_000.0),
        WingMaterial::Carbon => (1.6, 600_000_000.0),
        WingMaterial::Wood => (0.6, 40_000_000.0),
        WingMaterial::Aluminum => (2.7, 270_000_000.0),
    };
    
    let (spar_density, spar_strength) = match params.spar_material {
        SparMaterial::Carbon => (1600.0, 3_500_000_000.0),
        SparMaterial::Aluminum => (2700.0, 276_000_000.0),
        SparMaterial::Wood => (500.0, 100_000_000.0),
        SparMaterial::Steel => (7850.0, 250_000_000.0),
    };
    
    let wing_skin_mass = wing_area_single * wing_density * 0.002;
    let spar_volume = params.wing_span * 0.05 * 0.02;
    let spar_mass = spar_volume * spar_density / 1000.0;
    let wing_mass = wing_skin_mass + spar_mass + 2.0;
    
    let total_structural_mass = wing_mass * params.wing_count as f32 + params.motor_mass;
    
    let dynamic_pressure = 0.5 * params.air_density * params.forward_speed.powi(2);
    let max_lift_per_wing = params.airfoil_cl_max * dynamic_pressure * wing_area_single;
    let total_weight = (params.pilot_mass + total_structural_mass) * 9.81;
    let max_load_factor = (max_lift_per_wing * params.wing_count as f32) / total_weight;
    
    let moment_of_inertia = 0.05 * 0.02_f32.powi(3) / 12.0;
    let distributed_load = max_lift_per_wing / params.wing_span;
    let wing_deflection = (distributed_load * params.wing_span.powi(4)) / (8.0 * 200_000_000_000.0 * moment_of_inertia);
    
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
    
    let total_mass = params.pilot_mass + structural.total_structural_mass;
    let weight_force = total_mass * 9.81;
    
    let wing_area = params.wing_area();
    let effective_airspeed = params.forward_speed - params.wind_speed;
    let dynamic_pressure = 0.5 * params.air_density * effective_airspeed.powi(2);
    
    let stall_speed = (2.0 * weight_force / (params.air_density * wing_area * params.airfoil_cl_max)).sqrt();
    
    let reduced_frequency = params.flapping_frequency * params.wing_span / effective_airspeed.max(0.1);
    let flapping_boost = if params.flapping_frequency > 0.1 {
        1.0 + (reduced_frequency * 0.3 * (params.flapping_amplitude / 45.0)).min(0.8)
    } else {
        1.0
    };
    
    let current_cl = if effective_airspeed < stall_speed {
        0.3
    } else {
        let base_cl = weight_force / (dynamic_pressure * wing_area);
        (base_cl * flapping_boost).min(params.airfoil_cl_max)
    };
    
    let lift_force = current_cl * dynamic_pressure * wing_area * flapping_boost;
    
    let induced_drag_coeff = current_cl.powi(2) / (std::f32::consts::PI * params.aspect_ratio() * params.oswald_efficiency);
    let total_drag_coeff = params.airfoil_cd_min + induced_drag_coeff + 
                          if params.wing_count == 4 { 0.02 } else { 0.0 };
    let drag_force = total_drag_coeff * dynamic_pressure * wing_area;
    
    let power_to_overcome_drag = drag_force * effective_airspeed;
    let power_for_flapping = if params.flapping_frequency > 0.1 {
        let flapping_velocity = 2.0 * std::f32::consts::PI * params.flapping_frequency * 
                               params.wing_span * 0.7 * (params.flapping_amplitude.to_radians().sin());
        total_mass * 9.81 * flapping_velocity * 0.2 * params.wing_count as f32
    } else {
        0.0
    };
    
    let power_for_climb = 0.0;
    let total_power_required = power_to_overcome_drag + power_for_flapping + power_for_climb;
    
    let can_takeoff = (params.pilot_power_burst + params.motor_power * params.motor_efficiency) > 
                     (total_power_required * 1.5) && lift_force > weight_force * 0.8;
    
    let can_sustain_level_flight = params.pilot_power_sustained > total_power_required && 
                                  lift_force >= weight_force && 
                                  effective_airspeed > stall_speed;
    
    let excess_power = params.pilot_power_sustained - total_power_required;
    let can_climb = if excess_power > 0.0 { excess_power / weight_force } else { -0.5 };
    
    let motor_flight_time = params.battery_capacity / (params.motor_power / 1000.0) * 60.0;
    let takeoff_distance = if can_takeoff {
        let acceleration = ((params.pilot_power_burst + params.motor_power * params.motor_efficiency) / effective_airspeed - drag_force) / total_mass;
        effective_airspeed.powi(2) / (2.0 * acceleration.max(0.1))
    } else {
        f32::INFINITY
    };
    
    let chord_length = params.wing_chord;
    let reynolds_number = effective_airspeed * chord_length / 1.5e-5;
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

fn draw_3d_visualization(state: &SimulationState) {
    let center_x = screen_width() / 2.0;
    let center_y = screen_height() / 2.0 + 100.0;
    
    let wing_scale = 40.0;
    let rotation = state.camera_rotation;
    
    draw_rectangle(0.0, 100.0, screen_width(), screen_height() - 100.0, 
        Color::from_rgba(240, 245, 250, 255));
    
    let pilot_color = if state.analysis.can_sustain_level_flight {
        Color::from_rgba(50, 200, 50, 180)
    } else if state.analysis.can_takeoff {
        Color::from_rgba(200, 200, 50, 180)
    } else {
        Color::from_rgba(200, 50, 50, 180)
    };
    
    draw_circle(center_x, center_y, 20.0, pilot_color);
    draw_text("PILOT", center_x - 20.0, center_y + 5.0, 14.0, WHITE);
    
    let flap_angle = (state.time * state.params.flapping_frequency * 2.0 * std::f32::consts::PI).sin() 
        * state.params.flapping_amplitude.to_radians();
    
    for i in 0..state.params.wing_count {
        let angle = rotation + (i as f32 * 2.0 * std::f32::consts::PI / state.params.wing_count as f32);
        let wing_x = center_x + angle.cos() * 30.0;
        let wing_y = center_y + angle.sin() * 15.0;
        
        let wing_end_x = wing_x + angle.cos() * state.params.wing_span * wing_scale;
        let wing_end_y = wing_y + angle.sin() * state.params.wing_span * wing_scale * 0.3 
            + flap_angle.sin() * 20.0;
        
        let wing_color = if state.analysis.structural.structural_feasible {
            Color::from_rgba(100, 150, 200, 150)
        } else {
            Color::from_rgba(200, 100, 100, 150)
        };
        
        draw_line(wing_x, wing_y, wing_end_x, wing_end_y, 
            state.params.wing_chord * 10.0, wing_color);
        
        draw_line(wing_x, wing_y, wing_end_x, wing_end_y, 2.0, 
            Color::from_rgba(50, 50, 100, 255));
    }
    
    draw_text(&format!("Wing Configuration: {} wings × {:.1}m span", 
        state.params.wing_count, state.params.wing_span),
        center_x - 100.0, center_y + 150.0, 16.0, Color::from_rgba(50, 50, 100, 255));
    
    let status_color = if state.analysis.structural.structural_feasible {
        Color::from_rgba(50, 150, 50, 255)
    } else {
        Color::from_rgba(150, 50, 50, 255)
    };
    
    draw_text(&format!("Structural Status: {}", 
        if state.analysis.structural.structural_feasible { "SAFE" } else { "UNSAFE" }),
        center_x - 60.0, center_y + 170.0, 16.0, status_color);
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
    
    if analysis.can_sustain_level_flight {
        score += 1000.0;
    }
    if analysis.can_takeoff {
        score += 500.0;
    }
    if analysis.structural.structural_feasible {
        score += 500.0;
    }
    
    score += analysis.lift_to_weight_ratio * 100.0;
    score -= analysis.total_power_required / 100.0;
    score -= (params.pilot_mass + analysis.structural.total_structural_mass) * 2.0;
    score += analysis.motor_flight_time.min(30.0) * 10.0;
    score -= analysis.takeoff_distance.min(200.0) / 2.0;
    
    score
}

fn draw_parameter_heatmap(ui: &mut egui::Ui, params: &FlightParams) {
    ui.heading("Parameter Sensitivity Analysis");
    
    let param1_range = (1.0, 8.0, 15);
    let param2_range = (0.5, 3.0, 15);
    
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
                analysis.lift_to_weight_ratio 
            } else { 
                0.0 
            };
            
            max_score = max_score.max(score);
            min_score = min_score.min(score);
            row.push(score);
        }
        heatmap_data.push(row);
    }
    
    let plot_size = EguiVec2::new(400.0, 300.0);
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
    ui.heading("Real-Time Performance Metrics");
    
    let plot_height = 150.0;
    
    ui.group(|ui| {
        ui.label("Power Required (W)");
        let response = ui.allocate_response(EguiVec2::new(350.0, plot_height), egui::Sense::hover());
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
                egui::Pos2::new(rect.right() - 50.0, rect.top() + 10.0),
                egui::Align2::RIGHT_TOP,
                format!("{:.0}W", history.power_history.back().unwrap_or(&0.0)),
                egui::FontId::proportional(12.0),
                Color32::from_rgb(200, 50, 50),
            );
        }
    });
    
    ui.group(|ui| {
        ui.label("Lift-to-Weight Ratio");
        let response = ui.allocate_response(EguiVec2::new(350.0, plot_height), egui::Sense::hover());
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
                egui::Pos2::new(rect.right() - 50.0, rect.top() + 10.0),
                egui::Align2::RIGHT_TOP,
                format!("{:.2}", history.lift_history.back().unwrap_or(&0.0)),
                egui::FontId::proportional(12.0),
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
        show_3d_view: true,
        camera_rotation: 0.0,
        time: 0.0,
    };
    
    loop {
        clear_background(Color::from_rgba(245, 248, 252, 255));
        
        state.time += get_frame_time();
        state.camera_rotation += get_frame_time() * 0.3;
        
        state.history.update(&state.analysis, state.time);
        
        if state.show_3d_view {
            draw_3d_visualization(&state);
        }
        
        egui_macroquad::ui(|ctx| {
            egui::Window::new("Flight Configuration")
                .default_pos(egui::Pos2::new(10.0, 10.0))
                .show(ctx, |ui| {
                    let status_color = if state.analysis.can_sustain_level_flight && state.analysis.structural.structural_feasible {
                        Color32::from_rgb(50, 200, 50)
                    } else if state.analysis.can_takeoff {
                        Color32::from_rgb(200, 200, 50)
                    } else {
                        Color32::from_rgb(200, 50, 50)
                    };
                    
                    ui.colored_label(status_color, RichText::new(
                        if !state.analysis.structural.structural_feasible {
                            "⚠️ STRUCTURAL FAILURE RISK"
                        } else if !state.analysis.can_takeoff {
                            "❌ TAKEOFF NOT POSSIBLE"
                        } else if !state.analysis.can_sustain_level_flight {
                            "⚡ LEVEL FLIGHT NOT SUSTAINABLE"
                        } else {
                            "✅ FLIGHT CONFIGURATION VIABLE"
                        }
                    ).size(18.0));
                    
                    ui.separator();
                    
                    ui.collapsing("Pilot & Power", |ui| {
                        ui.add(egui::Slider::new(&mut state.params.pilot_mass, 50.0..=120.0)
                            .text("Pilot Mass (kg)")
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
                    });
                    
                    ui.collapsing("Wing Configuration", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Wing Count:");
                            if ui.button(format!("{} wings", state.params.wing_count)).clicked() {
                                state.params.wing_count = if state.params.wing_count == 2 { 4 } else { 2 };
                            }
                        });
                        
                        ui.add(egui::Slider::new(&mut state.params.wing_span, 1.5..=8.0)
                            .text("Wing Span")
                            .suffix(" m"));
                        
                        ui.add(egui::Slider::new(&mut state.params.wing_chord, 0.3..=3.0)
                            .text("Wing Chord")
                            .suffix(" m"));
                        
                        ui.label(format!("Total Wing Area: {:.1} m²", state.params.wing_area()));
                        ui.label(format!("Aspect Ratio: {:.2}", state.params.aspect_ratio()));
                    });
                    
                    ui.collapsing("Flight Dynamics", |ui| {
                        ui.add(egui::Slider::new(&mut state.params.forward_speed, 3.0..=35.0)
                            .text("Forward Speed")
                            .suffix(" m/s"));
                        
                        ui.label(format!("Stall Speed: {:.1} m/s", state.analysis.stall_speed));
                        
                        ui.add(egui::Slider::new(&mut state.params.flapping_frequency, 0.0..=4.0)
                            .text("Flapping Frequency")
                            .suffix(" Hz"));
                        
                        ui.add(egui::Slider::new(&mut state.params.flapping_amplitude, 5.0..=45.0)
                            .text("Flapping Amplitude")
                            .suffix("°"));
                        
                        ui.add(egui::Slider::new(&mut state.params.wind_speed, -10.0..=10.0)
                            .text("Wind Speed")
                            .suffix(" m/s"));
                    });
                    
                    ui.separator();
                    
                    if ui.button("Run Optimization").clicked() {
                        state.optimization_running = true;
                        state.optimization_result = Some(optimize_parameters(&state.params));
                        state.optimization_running = false;
                    }
                    
                    if let Some(ref optimal) = state.optimization_result {
                        ui.label("Optimization Result:");
                        ui.label(format!("Optimal Wing Span: {:.1}m", optimal.wing_span));
                        ui.label(format!("Optimal Wing Chord: {:.1}m", optimal.wing_chord));
                        ui.label(format!("Optimal Speed: {:.1}m/s", optimal.forward_speed));
                        ui.label(format!("Optimal Motor: {:.0}W", optimal.motor_power));
                        
                        if ui.button("Apply Optimal Parameters").clicked() {
                            state.params = optimal.clone();
                        }
                    }
                    
                    ui.checkbox(&mut state.show_3d_view, "Show 3D Visualization");
                    
                    state.analysis = calculate_comprehensive_flight_analysis(&state.params);
                });
            
            egui::Window::new("Performance Analysis")
                .default_pos(egui::Pos2::new(400.0, 10.0))
                .show(ctx, |ui| {
                    ui.heading("Key Metrics");
                    
                    ui.label(format!("Lift-to-Weight Ratio: {:.3}", state.analysis.lift_to_weight_ratio));
                    ui.label(format!("Total Power Required: {:.1} W", state.analysis.total_power_required));
                    ui.label(format!("Wing Loading: {:.1} N/m²", state.analysis.wing_loading));
                    ui.label(format!("Reynolds Number: {:.0}", state.analysis.reynolds_number));
                    
                    ui.separator();
                    
                    ui.heading("Power Breakdown");
                    let total = state.analysis.total_power_required.max(1.0);
                    ui.label(format!("Drag: {:.0}W ({:.0}%)", 
                        state.analysis.power_to_overcome_drag,
                        (state.analysis.power_to_overcome_drag / total) * 100.0));
                    ui.label(format!("Flapping: {:.0}W ({:.0}%)", 
                        state.analysis.power_for_flapping,
                        (state.analysis.power_for_flapping / total) * 100.0));
                    
                    ui.separator();
                    
                    draw_real_time_plots(ui, &state.history);
                });
            
            egui::Window::new("Structural Analysis")
                .default_pos(egui::Pos2::new(800.0, 10.0))
                .show(ctx, |ui| {
                    let color = if state.analysis.structural.structural_feasible {
                        Color32::from_rgb(50, 200, 50)
                    } else {
                        Color32::from_rgb(200, 50, 50)
                    };
                    
                    ui.colored_label(color, RichText::new(
                        if state.analysis.structural.structural_feasible { "STRUCTURALLY SAFE" } else { "STRUCTURAL FAILURE" }
                    ).size(16.0));
                    
                    ui.label(format!("Total Wing Mass: {:.1} kg", 
                        state.analysis.structural.wing_mass * state.params.wing_count as f32));
                    ui.label(format!("Max Load Factor: {:.2} g", 
                        state.analysis.structural.max_load_factor));
                    ui.label(format!("Wing Deflection: {:.1} mm", 
                        state.analysis.structural.wing_deflection * 1000.0));
                    ui.label(format!("Flutter Speed: {:.1} m/s", 
                        state.analysis.structural.critical_flutter_speed));
                    ui.label(format!("Flutter Margin: {:.2}x", 
                        state.analysis.flutter_margin));
                    
                    ui.separator();
                    
                    draw_parameter_heatmap(ui, &state.params);
                });
            
            egui::Window::new("Flight Status")
                .default_pos(egui::Pos2::new(400.0, 400.0))
                .show(ctx, |ui| {
                    ui.heading("Flight Capabilities");
                    
                    ui.horizontal(|ui| {
                        ui.label("Takeoff:");
                        ui.colored_label(
                            if state.analysis.can_takeoff { Color32::GREEN } else { Color32::RED },
                            if state.analysis.can_takeoff { "POSSIBLE" } else { "NOT POSSIBLE" }
                        );
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Sustained Flight:");
                        ui.colored_label(
                            if state.analysis.can_sustain_level_flight { Color32::GREEN } else { Color32::RED },
                            if state.analysis.can_sustain_level_flight { "POSSIBLE" } else { "NOT POSSIBLE" }
                        );
                    });
                    
                    ui.label(format!("Climb Rate: {:.2} m/s", state.analysis.can_climb));
                    ui.label(format!("Takeoff Distance: {:.0} m", 
                        if state.analysis.takeoff_distance.is_infinite() { 
                            9999.0 
                        } else { 
                            state.analysis.takeoff_distance 
                        }));
                    ui.label(format!("Motor Endurance: {:.1} min", state.analysis.motor_flight_time));
                });
        });
        
        egui_macroquad::draw();
        
        next_frame().await
    }
}
