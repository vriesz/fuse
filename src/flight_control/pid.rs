// src/flight_control/pid.rs

#[derive(Debug, Clone)]
pub struct PIDController {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
    pub setpoint: f32,
    integral: f32,
    last_error: f32,
}

impl PIDController {
    pub fn new(kp: f32, ki: f32, kd: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            setpoint: 0.0,
            integral: 0.0,
            last_error: 0.0,
        }
    }

    pub fn update(&mut self, measurement: f32, dt: f32) -> f32 {
        let error = self.setpoint - measurement;
        
        // Anti-windup and integral clamping
        self.integral = (self.integral + error * dt).clamp(-100.0, 100.0);
        
        let derivative = if dt > 0.0 {
            (error - self.last_error) / dt
        } else {
            0.0
        };
        
        self.last_error = error;
        
        (self.kp * error + self.ki * self.integral + self.kd * derivative)
            .clamp(-1.0, 1.0)
    }
}

impl From<(f32, f32, f32)> for PIDController {
    fn from(params: (f32, f32, f32)) -> Self {
        PIDController::new(params.0, params.1, params.2)
    }
}