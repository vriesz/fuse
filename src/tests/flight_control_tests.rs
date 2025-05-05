// src/tests/flight_control_tests.rs

#[cfg(test)]
mod tests {
    use crate::flight_control::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_pid_controller_creation() {
        let pid = PIDController::new(1.0, 0.5, 0.2);
        
        assert_eq!(pid.kp, 1.0);
        assert_eq!(pid.ki, 0.5);
        assert_eq!(pid.kd, 0.2);
        assert_eq!(pid.setpoint, 0.0);
    }
    
    #[test]
    fn test_pid_controller_from_tuple() {
        let params = (1.0, 0.5, 0.2);
        let pid = PIDController::from(params);
        
        assert_eq!(pid.kp, 1.0);
        assert_eq!(pid.ki, 0.5);
        assert_eq!(pid.kd, 0.2);
    }
    
    #[test]
    fn test_pid_controller_update() {
        let mut pid = PIDController::new(1.0, 0.1, 0.05);
        pid.setpoint = 10.0;
        
        // Initial error = 10 - 5 = 5
        let output1 = pid.update(5.0, 0.1);
        
        // P component = 1.0 * 5 = 5
        // I component = 0.1 * 5 * 0.1 = 0.05
        // D component = 0.05 * (5 - 0) / 0.1 = 2.5
        // Total = 7.55, but clamped to 1.0
        assert_relative_eq!(output1, 1.0);
        
        // Second update with smaller error = 10 - 8 = 2
        let output2 = pid.update(8.0, 0.1);
        
        // P component = 1.0 * 2 = 2
        // I component = 0.1 * (5 * 0.1 + 2 * 0.1) = 0.07
        // D component = 0.05 * (2 - 5) / 0.1 = -1.5
        // Total = 0.57
        assert_relative_eq!(output2, 0.57, epsilon = 0.01);
    }
    
    #[test]
    fn test_flight_controller_creation() {
        let fc = FlightController::new();
        
        assert_eq!(fc.mode, FlightMode::Manual);
        assert_eq!(fc.stability_threshold, 2.5);
    }
    
    #[test]
    fn test_flight_controller_from_params() {
        let params = (1.0, 0.5, 0.2);
        let fc = FlightController::from_params(params);
        
        assert_eq!(fc.mode, FlightMode::Manual);
        assert_eq!(fc.pid.kp, 1.0);
        assert_eq!(fc.pid.ki, 0.5);
        assert_eq!(fc.pid.kd, 0.2);
    }
    
    #[test]
    fn test_flight_controller_serialization() {
        let fc = FlightController::new();
        
        let serialized = serde_yaml::to_string(&fc).expect("Failed to serialize FlightController");
        let deserialized: FlightController = serde_yaml::from_str(&serialized).expect("Failed to deserialize FlightController");
        
        assert_eq!(fc.mode, deserialized.mode);
        assert_eq!(fc.stability_threshold, deserialized.stability_threshold);
        assert_eq!(fc.pid.kp, deserialized.pid.kp);
        assert_eq!(fc.pid.ki, deserialized.pid.ki);
        assert_eq!(fc.pid.kd, deserialized.pid.kd);
    }
}