use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

#[derive(Clone)]
struct FlightParams {
    // Human characteristics
    mass: f32, // kg
    height: f32, // m
    
    // Wing parameters
    wing_span: f32, // m
    wing_area: f32, // m²
    wing_aspect_ratio: f32,
    
    // Aerodynamic coefficients
    lift_coefficient: f32,
    drag_coefficient: f32,
    
    // Flight conditions
    flapping_frequency: f32, // Hz
    flapping_amplitude: f32, // degrees
    forward_speed: f32, // m/s
    
    // Environmental
    air_density: f32, // kg/m³
}

impl Default for FlightParams {
    fn default() -> Self {
        Self {
            mass: 80.0,
            height: 1.75,
            
            wing_span: 4.0,
            wing_area: 3.0,
            wing_aspect_ratio: 5.33,
            
            lift_coefficient: 1.2,
            drag_coefficient: 0.3,
            
            flapping_frequency: 3.0,
            flapping_amplitude: 30.0,
            forward_speed: 10.0,
            
            air_density: 1.225,
        }
    }
}

struct SimulationResult {
    lift_force: f32,
    drag_force: f32,
    weight: f32,
    thrust_required: f32,
    power_required: f32,
    can_fly: bool,
    net_vertical_force: f32,
    lift_to_weight_ratio: f32,
    reynolds_number: f32,
    froude_number: f32,
}

fn calculate_flight_physics(params: &FlightParams) -> SimulationResult {
    // Constants
    let gravity = 9.81; // m/s²
    let kinematic_viscosity = 1.5e-5; // m²/s for air
    
    // Calculate weight
    let weight = params.mass * gravity;
    
    // Calculate wing loading
    let _wing_loading = weight / params.wing_area;
    
    // Dynamic pressure
    let dynamic_pressure = 0.5 * params.air_density * params.forward_speed.powi(2);
    
    // Basic lift calculation (simplified)
    let base_lift = params.lift_coefficient * dynamic_pressure * params.wing_area;
    
    // Flapping enhancement factor (simplified model)
    let flapping_angle_rad = params.flapping_amplitude.to_radians();
    let flapping_velocity = 2.0 * std::f32::consts::PI * params.flapping_frequency * params.wing_span * 0.5 * flapping_angle_rad;
    let flapping_enhancement = 1.0 + (flapping_velocity / params.forward_speed).min(1.5);
    
    let lift_force = base_lift * flapping_enhancement;
    
    // Drag calculation
    let drag_force = params.drag_coefficient * dynamic_pressure * params.wing_area;
    
    // Thrust required to overcome drag
    let thrust_required = drag_force;
    
    // Power calculations
    let power_for_flight = thrust_required * params.forward_speed;
    let power_for_flapping = params.mass * gravity * flapping_velocity * 0.1; // Simplified
    let power_required = power_for_flight + power_for_flapping;
    
    // Reynolds number (characteristic of flow)
    let chord_length = params.wing_area / params.wing_span;
    let reynolds_number = params.forward_speed * chord_length / kinematic_viscosity;
    
    // Froude number (ratio of inertia to gravity)
    let froude_number = params.forward_speed / (gravity * params.height).sqrt();
    
    // Net vertical force
    let net_vertical_force = lift_force - weight;
    
    // Can fly check
    let can_fly = lift_force >= weight;
    
    SimulationResult {
        lift_force,
        drag_force,
        weight,
        thrust_required,
        power_required,
        can_fly,
        net_vertical_force,
        lift_to_weight_ratio: lift_force / weight,
        reynolds_number,
        froude_number,
    }
}

#[macroquad::main("Human Flight Simulator")]
async fn main() {
    let mut params = FlightParams::default();
    let mut result = calculate_flight_physics(&params);
    let mut show_advanced = false;
    
    loop {
        clear_background(Color::from_rgba(240, 240, 245, 255));
        
        // Title
        draw_text("Human Flight Physics Simulator", 20.0, 30.0, 32.0, BLACK);
        draw_text("Adjust parameters and click 'Run Simulation'", 20.0, 55.0, 20.0, DARKGRAY);
        
        // GUI Panel
        root_ui().window(hash!(), vec2(20., 80.), vec2(400., 600.), |ui| {
            ui.label(None, "Human Characteristics");
            ui.separator();
            
            ui.label(None, &format!("Mass: {:.1} kg", params.mass));
            widgets::Slider::new(hash!(), 30.0..150.0)
                .ui(ui, &mut params.mass);
            
            ui.label(None, &format!("Height: {:.2} m", params.height));
            widgets::Slider::new(hash!(), 1.0..2.5)
                .ui(ui, &mut params.height);
            
            ui.separator();
            ui.label(None, "Wing Parameters");
            ui.separator();
            
            ui.label(None, &format!("Wing Span: {:.1} m", params.wing_span));
            widgets::Slider::new(hash!(), 2.0..8.0)
                .ui(ui, &mut params.wing_span);
            
            ui.label(None, &format!("Wing Area: {:.1} m²", params.wing_area));
            widgets::Slider::new(hash!(), 1.0..10.0)
                .ui(ui, &mut params.wing_area);
            
            // Update aspect ratio when span or area changes
            params.wing_aspect_ratio = params.wing_span.powi(2) / params.wing_area;
            ui.label(None, &format!("Aspect Ratio: {:.2}", params.wing_aspect_ratio));
            
            ui.separator();
            ui.label(None, "Aerodynamics");
            ui.separator();
            
            ui.label(None, &format!("Lift Coefficient: {:.2}", params.lift_coefficient));
            widgets::Slider::new(hash!(), 0.5..2.5)
                .ui(ui, &mut params.lift_coefficient);
            
            ui.label(None, &format!("Drag Coefficient: {:.2}", params.drag_coefficient));
            widgets::Slider::new(hash!(), 0.1..1.0)
                .ui(ui, &mut params.drag_coefficient);
            
            ui.separator();
            ui.label(None, "Flight Conditions");
            ui.separator();
            
            ui.label(None, &format!("Forward Speed: {:.1} m/s", params.forward_speed));
            widgets::Slider::new(hash!(), 0.0..30.0)
                .ui(ui, &mut params.forward_speed);
            
            ui.label(None, &format!("Flapping Frequency: {:.1} Hz", params.flapping_frequency));
            widgets::Slider::new(hash!(), 0.0..10.0)
                .ui(ui, &mut params.flapping_frequency);
            
            ui.label(None, &format!("Flapping Amplitude: {:.0}°", params.flapping_amplitude));
            widgets::Slider::new(hash!(), 0.0..90.0)
                .ui(ui, &mut params.flapping_amplitude);
            
            ui.separator();
            
            if ui.button(None, "Run Simulation") {
                result = calculate_flight_physics(&params);
            }
            
            ui.checkbox(hash!(), "Show advanced metrics", &mut show_advanced);
        });
        
        // Results Panel
        root_ui().window(hash!(), vec2(450., 80.), vec2(400., 600.), |ui| {
            ui.label(None, "Simulation Results");
            ui.separator();
            
            // Flight status
            if result.can_fly {
                ui.label(None, "✓ FLIGHT POSSIBLE");
                draw_rectangle(455.0, 120.0, 380.0, 30.0, GREEN);
                draw_text("CAN FLY!", 460.0, 140.0, 24.0, WHITE);
            } else {
                ui.label(None, "✗ CANNOT FLY");
                draw_rectangle(455.0, 120.0, 380.0, 30.0, RED);
                draw_text("CANNOT FLY", 460.0, 140.0, 24.0, WHITE);
            }
            
            ui.separator();
            ui.label(None, "Forces:");
            ui.label(None, &format!("  Weight: {:.1} N", result.weight));
            ui.label(None, &format!("  Lift Force: {:.1} N", result.lift_force));
            ui.label(None, &format!("  Drag Force: {:.1} N", result.drag_force));
            ui.label(None, &format!("  Net Vertical: {:.1} N", result.net_vertical_force));
            
            ui.separator();
            ui.label(None, "Key Metrics:");
            ui.label(None, &format!("  Lift/Weight Ratio: {:.2}", result.lift_to_weight_ratio));
            ui.label(None, &format!("  Thrust Required: {:.1} N", result.thrust_required));
            ui.label(None, &format!("  Power Required: {:.1} W ({:.2} hp)", 
                result.power_required, result.power_required / 746.0));
            
            if show_advanced {
                ui.separator();
                ui.label(None, "Advanced Metrics:");
                ui.label(None, &format!("  Reynolds Number: {:.0}", result.reynolds_number));
                ui.label(None, &format!("  Froude Number: {:.2}", result.froude_number));
                ui.label(None, &format!("  Wing Loading: {:.1} N/m²", result.weight / params.wing_area));
            }
            
            ui.separator();
            ui.label(None, "Notes:");
            if result.lift_to_weight_ratio < 1.0 {
                ui.label(None, "• Need more lift (increase speed/area/CL)");
            }
            if result.power_required > 400.0 {
                ui.label(None, "• Power requirement exceeds human capability");
                ui.label(None, "  (Human max ~400W sustained)");
            }
            if params.wing_span > 6.0 {
                ui.label(None, "• Wing span may be impractical");
            }
        });
        
        // Visualization
        let viz_x = 870.0;
        let viz_y = 100.0;
        let viz_width = 300.0;
        let viz_height = 400.0;
        
        draw_rectangle(viz_x, viz_y, viz_width, viz_height, WHITE);
        draw_rectangle_lines(viz_x, viz_y, viz_width, viz_height, 2.0, BLACK);
        draw_text("Force Diagram", viz_x + 10.0, viz_y + 25.0, 24.0, BLACK);
        
        // Draw human figure
        let center_x = viz_x + viz_width / 2.0;
        let center_y = viz_y + viz_height / 2.0;
        
        // Scale forces for visualization
        let max_force = result.weight.max(result.lift_force).max(result.drag_force);
        let scale = 100.0 / max_force;
        
        // Draw forces as arrows
        // Weight (down)
        draw_line(center_x, center_y, center_x, center_y + result.weight * scale, 3.0, RED);
        draw_text("Weight", center_x + 5.0, center_y + result.weight * scale + 15.0, 16.0, RED);
        
        // Lift (up)
        draw_line(center_x, center_y, center_x, center_y - result.lift_force * scale, 3.0, GREEN);
        draw_text("Lift", center_x + 5.0, center_y - result.lift_force * scale - 5.0, 16.0, GREEN);
        
        // Drag (left)
        draw_line(center_x, center_y, center_x - result.drag_force * scale, center_y, 3.0, BLUE);
        draw_text("Drag", center_x - result.drag_force * scale - 40.0, center_y - 5.0, 16.0, BLUE);
        
        // Draw human silhouette
        draw_circle(center_x, center_y - 10.0, 10.0, BLACK); // Head
        draw_line(center_x, center_y, center_x, center_y + 30.0, 3.0, BLACK); // Body
        
        // Draw wings
        let wing_scale = params.wing_span * 20.0;
        draw_line(center_x - wing_scale, center_y, center_x + wing_scale, center_y, 2.0, DARKGRAY);
        
        next_frame().await
    }
}