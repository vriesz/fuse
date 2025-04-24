// src/flight_control/mod.rs

pub mod pid;
pub use pid::PIDController;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FlightMode {
    Manual,
    GPSHold,
    Autonomous,
    EmergencyLand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightController {
    pub mode: FlightMode,
    pub pid: PIDController,
    pub stability_threshold: f32,
}

impl FlightController {
    pub fn new() -> Self {
        Self {
            mode: FlightMode::Manual,
            pid: PIDController::new(0.8, 0.2, 0.1),
            stability_threshold: 2.5,
        }
    }
    
    pub fn from_params(params: (f32, f32, f32)) -> Self {
        Self {
            mode: FlightMode::Manual,
            pid: PIDController::new(params.0, params.1, params.2),
            stability_threshold: 2.5,
        }
    }
    
    pub fn adjust_altitude(&mut self, delta: f32) {
        // Implementation
        println!("Adjusting altitude by {}", delta);
    }
}