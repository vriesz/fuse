use uav_arch_gen::models::{
    UavConstraints, SWaPConstraints,
    FlightControllerType, CommsSystem, DataFusion,
    KalmanConfig, NeuralNetworkConfig, Processor,
    SensorSuite
};
use uav_arch_gen::engine::generate_architecture;
use rstest::rstest;
mod test_utils;
use test_utils::default_test_constraints;

#[test]
fn test_default_constraints() {
    let constraints = default_test_constraints();
    assert_eq!(constraints.autonomy_level, 3); // Example assertion
}

#[test]
fn test_default_architecture() {
    let constraints = UavConstraints::default();
    let arch = generate_architecture(&constraints);
    
    assert!(matches!(arch.processor, Processor::QualcommRB5));
    assert!(matches!(arch.data_fusion, DataFusion::KalmanFilter(_)));
    assert!(matches!(arch.flight_control, FlightControllerType::PX4(_)));
    assert!(matches!(arch.sensors, SensorSuite::GpsEnhanced));
    assert!(matches!(arch.comms, CommsSystem::MAVLink { version: 2 }));
}

#[rstest]
#[case(true, DataFusion::DnnFusion(NeuralNetworkConfig { model_path: "".into(), inference_rate: 30 }))]
#[case(false, DataFusion::KalmanFilter(KalmanConfig { process_noise: 0.1, sensor_weights: (0.7, 0.3) }))]
fn test_ai_requirements(#[case] requires_ai: bool, #[case] expected_fusion: DataFusion) {
    let constraints = UavConstraints {
        requires_ai,
        swap: SWaPConstraints {  // Add missing SWaP fields
            max_weight_kg: 10.0,
            max_power_w: 100.0,
            max_size_cm: (100.0, 100.0, 50.0),
            min_compute_threshold: Some(1.0),
            max_cost: Some(5000.0),
        },
        ..UavConstraints::default()
    };
    let arch = generate_architecture(&constraints);
    match (arch.data_fusion, expected_fusion) {
        (DataFusion::DnnFusion(_), DataFusion::DnnFusion(_)) => (),
        (DataFusion::KalmanFilter(_), DataFusion::KalmanFilter(_)) => (),
        _ => panic!("Unexpected data fusion variant"),
    }
}

#[test]
fn test_secure_comms() {
    let constraints = UavConstraints {
        secure_comms: true,
        swap: SWaPConstraints {  // Add missing SWaP fields
            max_weight_kg: 10.0,
            max_power_w: 100.0,
            max_size_cm: (100.0, 100.0, 50.0),
            min_compute_threshold: Some(1.0),
            max_cost: Some(5000.0),
        },
        ..UavConstraints::default()
    };
    let arch = generate_architecture(&constraints);
    
    if let CommsSystem::MilitaryEncrypted { key_rotation } = arch.comms {
        assert_eq!(key_rotation, 24);
    } else {
        panic!("Expected MilitaryEncrypted comms");
    }
}

#[test]
fn test_ai_comms() {
    let constraints = UavConstraints {
        requires_ai: true,
        swap: SWaPConstraints {  // Add missing SWaP fields
            max_weight_kg: 10.0,
            max_power_w: 100.0,
            max_size_cm: (100.0, 100.0, 50.0),
            min_compute_threshold: Some(1.0),
            max_cost: Some(5000.0),
        },
        ..UavConstraints::default()
    };
    let arch = generate_architecture(&constraints);
    assert!(matches!(arch.comms, CommsSystem::WiFiDirect { bandwidth: 100 }));
}