use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

#[derive(Clone, Debug)]
struct FlightParams {
    mass: f32, // kg
    wing_span: f32, // m
    wing_area: f32, // m²
    lift_coefficient: f32,
    drag_coefficient: f32,
    flapping_frequency: f32, // Hz
    forward_speed: f32, // m/s
}

impl Default for FlightParams {
    fn default() -> Self {
        Self {
            mass: 80.0,
            wing_span: 4.0,
            wing_area: 3.0,
            lift_coefficient: 1.2,
            drag_coefficient: 0.3,
            flapping_frequency: 3.0,
            forward_speed: 10.0,
        }
    }
}

#[derive(Clone)]
struct Preset {
    name: &'static str,
    params: FlightParams,
    description: &'static str,
}

fn get_presets() -> Vec<Preset> {
    vec![
        Preset {
            name: "Default",
            params: FlightParams::default(),
            description: "Standard human with basic mechanical wings",
        },
        Preset {
            name: "Optimized",
            params: FlightParams {
                mass: 70.0,
                wing_span: 6.0,
                wing_area: 5.0,
                lift_coefficient: 1.6,
                drag_coefficient: 0.18,
                flapping_frequency: 2.5,
                forward_speed: 12.0,
            },
            description: "Lightweight human with efficient wing design",
        },
        Preset {
            name: "Large Wings",
            params: FlightParams {
                mass: 80.0,
                wing_span: 8.0,
                wing_area: 8.0,
                lift_coefficient: 1.4,
                drag_coefficient: 0.22,
                flapping_frequency: 2.0,
                forward_speed: 15.0,
            },
            description: "Large wing area for maximum lift generation",
        },
    ]
}

#[derive(Clone)]
struct SimulationResult {
    lift_force: f32,
    weight: f32,
    power_required: f32,
    can_fly: bool,
    lift_to_weight_ratio: f32,
    efficiency_score: f32,
}

fn calculate_flight_physics(params: &FlightParams) -> SimulationResult {
    let gravity = 9.81;
    let weight = params.mass * gravity;
    
    // Basic lift calculation
    let dynamic_pressure = 0.5 * 1.225 * params.forward_speed.powi(2);
    let base_lift = params.lift_coefficient * dynamic_pressure * params.wing_area;
    
    // Flapping enhancement
    let flapping_velocity = 2.0 * std::f32::consts::PI * params.flapping_frequency * params.wing_span * 0.5;
    let flapping_enhancement = 1.0 + (flapping_velocity / params.forward_speed.max(0.1)).min(1.5);
    let lift_force = base_lift * flapping_enhancement;
    
    // Power calculation
    let drag_force = params.drag_coefficient * dynamic_pressure * params.wing_area;
    let power_for_flight = drag_force * params.forward_speed;
    let power_for_flapping = params.mass * gravity * flapping_velocity * 0.1;
    let power_required = power_for_flight + power_for_flapping;
    
    // Efficiency score
    let power_efficiency = (400.0 - power_required.min(400.0)) / 400.0;
    let lift_efficiency = (lift_force / weight).min(2.0) / 2.0;
    let efficiency_score = (power_efficiency * 0.6 + lift_efficiency * 0.4) * 100.0;
    
    SimulationResult {
        lift_force,
        weight,
        power_required,
        can_fly: lift_force >= weight,
        lift_to_weight_ratio: lift_force / weight,
        efficiency_score,
    }
}

fn draw_graph(x: f32, y: f32, width: f32, height: f32, params: &FlightParams, graph_type: &str) {
    draw_rectangle(x, y, width, height, Color::from_rgba(25, 25, 30, 255));
    draw_rectangle_lines(x, y, width, height, 2.0, Color::from_rgba(100, 100, 120, 255));
    
    draw_text(graph_type, x + 10.0, y + 20.0, 16.0, WHITE);
    
    let mut data = Vec::new();
    for speed in 0..21 {
        let mut test_params = params.clone();
        test_params.forward_speed = speed as f32;
        let result = calculate_flight_physics(&test_params);
        
        let value = match graph_type {
            "Lift/Weight Ratio" => result.lift_to_weight_ratio,
            _ => result.power_required,
        };
        data.push((speed as f32, value));
    }
    
    if !data.is_empty() {
        let max_y = data.iter().map(|(_, y)| *y).fold(0.0f32, f32::max);
        let min_y = data.iter().map(|(_, y)| *y).fold(max_y, f32::min);
        let y_range = (max_y - min_y).max(0.1);
        
        let color = match graph_type {
            "Lift/Weight Ratio" => GREEN,
            _ => ORANGE,
        };
        
        for i in 1..data.len() {
            let x1 = x + (data[i-1].0 / 20.0) * (width - 20.0) + 10.0;
            let y1 = y + height - ((data[i-1].1 - min_y) / y_range) * (height - 40.0) - 30.0;
            let x2 = x + (data[i].0 / 20.0) * (width - 20.0) + 10.0;
            let y2 = y + height - ((data[i].1 - min_y) / y_range) * (height - 40.0) - 30.0;
            
            draw_line(x1, y1, x2, y2, 3.0, color);
            draw_circle(x2, y2, 2.0, color);
        }
        
        // Reference lines
        if graph_type == "Lift/Weight Ratio" {
            let ref_y = y + height - ((1.0 - min_y) / y_range) * (height - 40.0) - 30.0;
            draw_line(x + 10.0, ref_y, x + width - 10.0, ref_y, 1.0, Color::from_rgba(255, 255, 255, 100));
            draw_text("Flight threshold", x + width - 120.0, ref_y - 5.0, 12.0, WHITE);
        }
    }
}

#[macroquad::main("Human Flight Physics Simulator")]
async fn main() {
    let mut params = FlightParams::default();
    let mut result = calculate_flight_physics(&params);
    let mut selected_preset = 0;
    let presets = get_presets();
    
    loop {
        clear_background(Color::from_rgba(20, 20, 25, 255));
        
        // Header
        draw_rectangle(0.0, 0.0, screen_width(), 90.0, Color::from_rgba(30, 30, 40, 255));
        draw_text("Human Flight Physics Simulator", 30.0, 35.0, 36.0, WHITE);
        draw_text("Can humans achieve powered flight? Let's find out!", 30.0, 60.0, 18.0, Color::from_rgba(180, 180, 200, 255));
        
        // Presets
        let mut preset_x = 20.0;
        for (i, preset) in presets.iter().enumerate() {
            let color = if i == selected_preset {
                Color::from_rgba(100, 200, 255, 255)
            } else {
                Color::from_rgba(60, 60, 80, 255)
            };
            
            draw_rectangle(preset_x, 100.0, 140.0, 35.0, color);
            draw_rectangle_lines(preset_x, 100.0, 140.0, 35.0, 1.0, WHITE);
            draw_text(preset.name, preset_x + 10.0, 120.0, 16.0, WHITE);
            
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mouse_x, mouse_y) = mouse_position();
                if mouse_x >= preset_x && mouse_x <= preset_x + 140.0 && 
                   mouse_y >= 100.0 && mouse_y <= 135.0 {
                    selected_preset = i;
                    params = preset.params.clone();
                    result = calculate_flight_physics(&params);
                }
            }
            preset_x += 150.0;
        }
        
        // Result indicator
        let result_y = 150.0;
        let result_color = if result.can_fly {
            Color::from_rgba(50, 200, 50, 255)
        } else {
            Color::from_rgba(200, 50, 50, 255)
        };
        
        draw_rectangle(30.0, result_y, screen_width() - 60.0, 100.0, result_color);
        draw_rectangle(35.0, result_y + 5.0, screen_width() - 70.0, 90.0, Color::from_rgba(30, 30, 40, 255));
        
        if result.can_fly {
            draw_text("FLIGHT ACHIEVED!", 60.0, result_y + 40.0, 36.0, Color::from_rgba(50, 200, 50, 255));
            draw_text(&format!("Lift/Weight: {:.2}x  |  Efficiency: {:.0}%  |  Power: {:.0}W", 
                result.lift_to_weight_ratio, result.efficiency_score, result.power_required), 
                60.0, result_y + 75.0, 20.0, WHITE);
        } else {
            draw_text("CANNOT ACHIEVE FLIGHT", 60.0, result_y + 40.0, 36.0, Color::from_rgba(200, 50, 50, 255));
            draw_text(&format!("Need {:.0}% more lift  |  Power: {:.0}W  |  L/W Ratio: {:.2}", 
                (1.0 / result.lift_to_weight_ratio - 1.0) * 100.0, result.power_required, result.lift_to_weight_ratio), 
                60.0, result_y + 75.0, 20.0, WHITE);
        }
        
        // Parameters panel
        root_ui().window(hash!(), vec2(30., 270.), vec2(350., 400.), |ui| {
            ui.label(None, "Flight Parameters");
            ui.separator();
            
            ui.label(None, &format!("Mass: {:.0} kg", params.mass));
            let old_mass = params.mass;
            widgets::Slider::new(hash!(), 40.0..120.0).ui(ui, &mut params.mass);
            if old_mass != params.mass {
                result = calculate_flight_physics(&params);
            }
            
            ui.label(None, &format!("Wing Span: {:.1} m", params.wing_span));
            let old_span = params.wing_span;
            widgets::Slider::new(hash!(), 2.0..10.0).ui(ui, &mut params.wing_span);
            if old_span != params.wing_span {
                result = calculate_flight_physics(&params);
            }
            
            ui.label(None, &format!("Wing Area: {:.1} m²", params.wing_area));
            let old_area = params.wing_area;
            widgets::Slider::new(hash!(), 1.0..15.0).ui(ui, &mut params.wing_area);
            if old_area != params.wing_area {
                result = calculate_flight_physics(&params);
            }
            
            ui.separator();
            
            ui.label(None, &format!("Lift Coefficient: {:.2}", params.lift_coefficient));
            let old_lift = params.lift_coefficient;
            widgets::Slider::new(hash!(), 0.5..2.5).ui(ui, &mut params.lift_coefficient);
            if old_lift != params.lift_coefficient {
                result = calculate_flight_physics(&params);
            }
            
            ui.label(None, &format!("Drag Coefficient: {:.2}", params.drag_coefficient));
            let old_drag = params.drag_coefficient;
            widgets::Slider::new(hash!(), 0.1..0.8).ui(ui, &mut params.drag_coefficient);
            if old_drag != params.drag_coefficient {
                result = calculate_flight_physics(&params);
            }
            
            ui.separator();
            
            ui.label(None, &format!("Forward Speed: {:.1} m/s", params.forward_speed));
            let old_speed = params.forward_speed;
            widgets::Slider::new(hash!(), 1.0..25.0).ui(ui, &mut params.forward_speed);
            if old_speed != params.forward_speed {
                result = calculate_flight_physics(&params);
            }
            
            ui.label(None, &format!("Flapping Rate: {:.1} Hz", params.flapping_frequency));
            let old_freq = params.flapping_frequency;
            widgets::Slider::new(hash!(), 0.5..8.0).ui(ui, &mut params.flapping_frequency);
            if old_freq != params.flapping_frequency {
                result = calculate_flight_physics(&params);
            }
            
            ui.separator();
            
            let wing_loading = result.weight / params.wing_area;
            ui.label(None, &format!("Wing Loading: {:.1} N/m²", wing_loading));
            ui.label(None, &format!("Aspect Ratio: {:.1}", params.wing_span.powi(2) / params.wing_area));
        });
        
        // Graphs
        draw_graph(400.0, 270.0, 280.0, 190.0, &params, "Lift/Weight Ratio");
        draw_graph(400.0, 480.0, 280.0, 190.0, &params, "Power Required (W)");
        
        // Force visualization
        let viz_x = 700.0;
        let viz_y = 270.0;
        draw_rectangle(viz_x, viz_y, 250.0, 190.0, Color::from_rgba(25, 25, 30, 255));
        draw_rectangle_lines(viz_x, viz_y, 250.0, 190.0, 2.0, WHITE);
        draw_text("Force Balance", viz_x + 10.0, viz_y + 20.0, 16.0, WHITE);
        
        let center_x = viz_x + 125.0;
        let center_y = viz_y + 100.0;
        let scale = 80.0 / result.weight.max(result.lift_force);
        
        // Draw human figure
        draw_circle(center_x, center_y - 10.0, 8.0, WHITE);
        draw_line(center_x, center_y, center_x, center_y + 25.0, 2.0, WHITE);
        let wing_scale = params.wing_span * 15.0;
        draw_line(center_x - wing_scale, center_y, center_x + wing_scale, center_y, 2.0, GRAY);
        
        // Forces
        draw_line(center_x, center_y, center_x, center_y + result.weight * scale, 3.0, RED);
        draw_text("Weight", center_x + 5.0, center_y + result.weight * scale + 15.0, 12.0, RED);
        
        draw_line(center_x, center_y, center_x, center_y - result.lift_force * scale, 3.0, GREEN);
        draw_text("Lift", center_x + 5.0, center_y - result.lift_force * scale - 5.0, 12.0, GREEN);
        
        // Tips
        let tip_y = screen_height() - 80.0;
        draw_rectangle(0.0, tip_y, screen_width(), 80.0, Color::from_rgba(30, 30, 40, 255));
        
        let tip_text = if result.can_fly {
            if result.efficiency_score > 70.0 {
                "Excellent! Very efficient flight design. Human-powered flight achieved!"
            } else {
                "Flight achieved! Try optimizing for better efficiency and lower power requirements."
            }
        } else if result.lift_to_weight_ratio > 0.8 {
            "Very close! Try increasing wing area, lift coefficient, or forward speed slightly."
        } else if result.power_required > 500.0 {
            "Reduce power requirements by improving aerodynamics or wing efficiency."
        } else {
            "Need more lift: increase wing area, lift coefficient, or forward speed significantly."
        };
        
        draw_text(tip_text, 30.0, tip_y + 25.0, 18.0, WHITE);
        draw_text(&format!("Current: {} - {}", presets[selected_preset].name, presets[selected_preset].description), 
            30.0, tip_y + 50.0, 14.0, Color::from_rgba(160, 160, 180, 255));
        
        next_frame().await
    }
}