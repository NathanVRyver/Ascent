use bevy::prelude::*;
use rand::Rng;

#[derive(Debug, Clone, Resource)]
pub struct WeatherParams {
    pub base_wind: Vec3,
    pub turbulence_intensity: f32,
    pub gust_frequency: f32,
    pub gust_strength: f32,
    pub temperature: f32,
    pub pressure: f32,
    pub humidity: f32,
}

impl Default for WeatherParams {
    fn default() -> Self {
        Self {
            base_wind: Vec3::new(5.0, 0.0, 2.0),
            turbulence_intensity: 0.1,
            gust_frequency: 0.1,
            gust_strength: 2.0,
            temperature: 15.0,
            pressure: 101325.0,
            humidity: 0.5,
        }
    }
}

pub fn calculate_air_density(temperature: f32, pressure: f32, humidity: f32) -> f32 {
    let r_dry = 287.05;
    let r_vapor = 461.495;
    let temperature_kelvin = temperature + 273.15;
    
    let saturation_pressure = 611.0 * (17.502 * temperature / (240.97 + temperature)).exp();
    let vapor_pressure = humidity * saturation_pressure;
    let dry_pressure = pressure - vapor_pressure;
    
    (dry_pressure / (r_dry * temperature_kelvin)) + (vapor_pressure / (r_vapor * temperature_kelvin))
}

pub fn calculate_wind_with_turbulence(
    weather: &WeatherParams,
    position: Vec3,
    time: f32,
) -> Vec3 {
    let mut rng = rand::thread_rng();
    
    let turbulence = Vec3::new(
        (position.x * 0.1 + time * 0.5).sin() * weather.turbulence_intensity,
        (position.y * 0.15 + time * 0.7).cos() * weather.turbulence_intensity,
        (position.z * 0.12 + time * 0.6).sin() * weather.turbulence_intensity,
    ) * 10.0;
    
    let gust_chance: f32 = rng.gen_range(0.0..1.0);
    let gust = if gust_chance < weather.gust_frequency {
        Vec3::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-0.5..0.5),
            rng.gen_range(-1.0..1.0),
        ) * weather.gust_strength
    } else {
        Vec3::ZERO
    };
    
    weather.base_wind + turbulence + gust
}

pub fn calculate_density_altitude(pressure_altitude: f32, temperature: f32) -> f32 {
    let standard_temp = 15.0 - 0.00198 * pressure_altitude;
    let temp_correction = 37.2 * (temperature - standard_temp);
    pressure_altitude + temp_correction
}

pub fn apply_wind_to_velocity(aircraft_velocity: Vec3, wind_velocity: Vec3) -> Vec3 {
    aircraft_velocity - wind_velocity
}