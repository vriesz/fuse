// src/engine/mod.rs

use crate::models::{
    UavConstraints, UavArchitecture,
    Processor, DataFusion, FlightControllerType,
    SensorSuite, CommsSystem, KalmanConfig,
    NeuralNetworkConfig, PIDParams
};

pub fn generate_architecture(constraints: &UavConstraints) -> UavArchitecture {
    // Processor and Data Fusion
    let (processor, data_fusion) = match (constraints.secure_comms, constraints.requires_ai) {
        (true, _) => (Processor::XilinxZynqUltraScale, 
                     if constraints.requires_ai { 
                         DataFusion::DnnFusion(NeuralNetworkConfig {
                             model_path: "models/object_detection.tract".into(),
                             inference_rate: 30,
                         })
                     } else { 
                         DataFusion::KalmanFilter(KalmanConfig {
                             process_noise: 0.1,
                             sensor_weights: (0.7, 0.3),
                         }) 
                     }),
        (false, true) => (Processor::NvidiaJetsonAGXOrin, 
                          DataFusion::DnnFusion(NeuralNetworkConfig {
                              model_path: "models/object_detection.tract".into(),
                              inference_rate: 30,
                          })),
        _ => (Processor::QualcommRB5, 
              DataFusion::KalmanFilter(KalmanConfig {
                  process_noise: 0.1,
                  sensor_weights: (0.7, 0.3),
              })),
    };

    // Flight Controller Selection with PID parameters
    let pid_params = match constraints.autonomy_level {
        0..=2 => PIDParams {
            roll: (0.8, 0.1, 0.05),
            pitch: (0.8, 0.1, 0.05),
            yaw: (1.0, 0.0, 0.1),
        },
        _ => PIDParams {
            roll: (1.2, 0.2, 0.1),
            pitch: (1.2, 0.2, 0.1),
            yaw: (1.5, 0.1, 0.2),
        },
    };

    let flight_control = match constraints.autonomy_level {
        0..=2 => FlightControllerType::Betaflight,
        3..=4 => FlightControllerType::PX4(pid_params),
        _ => FlightControllerType::Custom(pid_params),
    };

    // Sensor Suite Selection
    let sensors = match (constraints.requires_ai, constraints.secure_comms) {
        (true, _) => SensorSuite::Autonomous,
        (false, true) => SensorSuite::FullNavigation,
        _ => SensorSuite::GpsEnhanced,
    };

    // Communication System
    let comms = if constraints.secure_comms {
        CommsSystem::MilitaryEncrypted { key_rotation: 24 }
    } else if constraints.requires_ai {
        CommsSystem::WiFiDirect { bandwidth: 100 }
    } else {
        CommsSystem::MAVLink { version: 2 }
    };

    UavArchitecture {
        processor,
        data_fusion,
        flight_control,
        sensors,
        comms,
    }
}

pub fn optimize_cost(architectures: Vec<UavArchitecture>) -> UavArchitecture {
    architectures.into_iter()
        .min_by_key(|arch| match arch.processor {
            Processor::XilinxZynqUltraScale => 1200,
            Processor::NvidiaJetsonAGXOrin => 800,
            Processor::QualcommRB5 => 300,
            Processor::IntelCorei7 => 600,
            Processor::RaspberryPiCM4 => 100,
        })
        .expect("At least one architecture should be provided")
}