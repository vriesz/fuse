// src/tests/engine_tests.rs

#[cfg(test)]
mod tests {
    use crate::engine::*;
    use crate::models::components::*;
    use crate::models::constraints::*;
    
    #[test]
    fn test_default_architecture_generation() {
        let constraints = UavConstraints::default();
        let architecture = generate_architecture(&constraints);
        
        // Check expected default values
        assert_eq!(architecture.processor, Processor::QualcommRB5);
        
        match &architecture.data_fusion {
            DataFusion::KalmanFilter(config) => {
                assert_eq!(config.process_noise, 0.1);
                assert_eq!(config.sensor_weights, (0.7, 0.3));
            },
            _ => panic!("Expected KalmanFilter for default constraints"),
        }
        
        match &architecture.flight_control {
            FlightControllerType::PX4(pid_params) => {
                assert_eq!(pid_params.roll, (1.2, 0.2, 0.1));
                assert_eq!(pid_params.pitch, (1.2, 0.2, 0.1));
                assert_eq!(pid_params.yaw, (1.5, 0.1, 0.2));
            },
            _ => panic!("Expected PX4 for default constraints with autonomy level 3"),
        }
        
        assert_eq!(architecture.sensors, SensorSuite::GpsEnhanced);
        
        match &architecture.comms {
            CommsSystem::MAVLink { version } => {
                assert_eq!(*version, 2);
            },
            _ => panic!("Expected MAVLink for default constraints"),
        }
    }
    
    #[test]
    fn test_secure_architecture_generation() {
        let mut constraints = UavConstraints::default();
        constraints.secure_comms = true;
        
        let architecture = generate_architecture(&constraints);
        
        // Check secure architecture components
        assert_eq!(architecture.processor, Processor::XilinxZynqUltraScale);
        assert_eq!(architecture.sensors, SensorSuite::FullNavigation);
        
        match &architecture.comms {
            CommsSystem::MilitaryEncrypted { key_rotation } => {
                assert_eq!(*key_rotation, 24);
            },
            _ => panic!("Expected MilitaryEncrypted for secure constraints"),
        }
    }
    
    #[test]
    fn test_ai_architecture_generation() {
        let mut constraints = UavConstraints::default();
        constraints.requires_ai = true;
        
        let architecture = generate_architecture(&constraints);
        
        // Check AI architecture components
        assert_eq!(architecture.processor, Processor::NvidiaJetsonAGXOrin);
        assert_eq!(architecture.sensors, SensorSuite::Autonomous);
        
        match &architecture.data_fusion {
            DataFusion::DnnFusion(config) => {
                assert_eq!(config.model_path, "models/object_detection.tract");
                assert_eq!(config.inference_rate, 30);
            },
            _ => panic!("Expected DnnFusion for AI constraints"),
        }
        
        match &architecture.comms {
            CommsSystem::WiFiDirect { bandwidth } => {
                assert_eq!(*bandwidth, 100);
            },
            _ => panic!("Expected WiFiDirect for AI constraints"),
        }
    }
    
    #[test]
    fn test_low_autonomy_architecture() {
        let mut constraints = UavConstraints::default();
        constraints.autonomy_level = 1;
        
        let architecture = generate_architecture(&constraints);
        
        // Check low autonomy level affects flight control
        match &architecture.flight_control {
            FlightControllerType::Betaflight => (),
            _ => panic!("Expected Betaflight for low autonomy constraints"),
        }
    }
    
    #[test]
    fn test_high_autonomy_architecture() {
        let mut constraints = UavConstraints::default();
        constraints.autonomy_level = 5;
        
        let architecture = generate_architecture(&constraints);
        
        // Check high autonomy level affects flight control
        match &architecture.flight_control {
            FlightControllerType::Custom(pid_params) => {
                assert_eq!(pid_params.roll, (1.2, 0.2, 0.1));
                assert_eq!(pid_params.pitch, (1.2, 0.2, 0.1));
                assert_eq!(pid_params.yaw, (1.5, 0.1, 0.2));
            },
            _ => panic!("Expected Custom flight controller for high autonomy constraints"),
        }
    }
    
    #[test]
    fn test_cost_optimization() {
        // Create multiple architectures with different processors
        let arch1 = UAVArchitecture {
            processor: Processor::XilinxZynqUltraScale,
            data_fusion: DataFusion::KalmanFilter(KalmanConfig {
                process_noise: 0.1,
                sensor_weights: (0.7, 0.3),
            }),
            flight_control: FlightControllerType::Betaflight,
            sensors: SensorSuite::BasicImu,
            comms: CommsSystem::MAVLink { version: 2 },
        };
        
        let mut arch2 = arch1.clone();
        arch2.processor = Processor::NvidiaJetsonAGXOrin;
        
        let mut arch3 = arch1.clone();
        arch3.processor = Processor::QualcommRB5;
        
        let mut arch4 = arch1.clone();
        arch4.processor = Processor::RaspberryPiCM4;
        
        let architectures = vec![arch1, arch2, arch3, arch4];
        
        // Optimize for cost
        let optimized = optimize_cost(architectures);
        
        // Should select the cheapest processor (RaspberryPiCM4)
        assert_eq!(optimized.processor, Processor::RaspberryPiCM4);
    }
}