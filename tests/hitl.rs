#[cfg(feature = "hitl")]
mod hitl_test {
    use uav_arch_gen::flight_control::pid::PIDController;

    struct Simulator {
        current_altitude: f32,  // Changed to f32
        control_inputs: Vec<(f32, f32, f32)>,  // Changed to f32
    }

    impl Simulator {
        fn new(initial_altitude: f32) -> Self {  // f32
            Self {
                current_altitude: initial_altitude,
                control_inputs: Vec::new(),
            }
        }

        fn get_altitude(&self) -> f32 {  // f32
            self.current_altitude
        }

        fn apply_control(&mut self, control: (f32, f32, f32)) {  // f32
            self.control_inputs.push(control);
            self.current_altitude += control.2 * 0.5;
        }
    }

    #[test]
    fn test_pid_control() {
        let mut sim = Simulator::new(100.0);  // f32 literal
        let mut pid = PIDController::new(0.8, 0.2, 0.1);
        pid.setpoint = 150.0;  // f32

        for _ in 0..10 {
            let altitude = sim.get_altitude();
            let control_output = pid.update(altitude, 0.1);  // Both f32
            sim.apply_control((0.0, 0.0, control_output));
        }
    }
}
