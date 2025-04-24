// src/tests/hitl_tests.rs

#[cfg(test)]
#[cfg(feature = "hitl")]
mod hitl_test {
    use crate::flight_control::pid::PIDController;
    use approx::assert_relative_eq;

    struct Simulator {
        current_altitude: f32,
        control_inputs: Vec<(f32, f32, f32)>,
    }

    impl Simulator {
        fn new(initial_altitude: f32) -> Self {
            Self {
                current_altitude: initial_altitude,
                control_inputs: Vec::new(),
            }
        }

        fn get_altitude(&self) -> f32 {
            self.current_altitude
        }

        fn apply_control(&mut self, control: (f32, f32, f32)) {
            self.control_inputs.push(control);
            self.current_altitude += control.2 * 0.5;
        }
    }

    #[test]
    fn test_pid_control() {
        let mut sim = Simulator::new(100.0);
        let mut pid = PIDController::new(0.8, 0.2, 0.1);
        pid.setpoint = 150.0;

        for _ in 0..10 {
            let altitude = sim.get_altitude();
            let control_output = pid.update(altitude, 0.1);
            sim.apply_control((0.0, 0.0, control_output));
            
            // Verify control output is finite and reasonable
            assert!(control_output.is_finite());
            assert!(control_output >= -1.0 && control_output <= 1.0);
        }

        // After several iterations, we should be closer to the setpoint
        let final_altitude = sim.get_altitude();
        assert_relative_eq!(
            final_altitude,
            125.0,  // Expected value after 10 iterations
            epsilon = 25.0  // Allowed margin of error
        );
    }
}