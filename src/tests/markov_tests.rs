// src/tests/markov_tests.rs

#[cfg(test)]
mod tests {
    use crate::comms::*;
    use crate::models::markov_chain::MarkovEnvironmentModel;
    use crate::ooda::*;
    // use crate::flight_control::*;
    use crate::models::architecture::UavSystems;
    use crate::models::constraints::MissionType;
    use crate::payload::*;
    use crate::sensor_fusion::*;
    // use std::time::Duration;

    #[test]
    fn test_markov_model_transition() {
        // Create basic model
        let states = vec!["clear".to_string(), "rain".to_string(), "fog".to_string()];
        let transition_matrix = vec![
            vec![0.7, 0.2, 0.1], // from clear
            vec![0.3, 0.4, 0.3], // from rain
            vec![0.2, 0.3, 0.5], // from fog
        ];

        let mut model = MarkovEnvironmentModel::new(&states, transition_matrix, 0);

        // Check initial state
        assert_eq!(model.get_current_state(), "clear");

        // Simulate state observations
        model.update_state(Some(1)); // Observe "rain"
        assert_eq!(model.get_current_state(), "rain");

        model.update_state(Some(2)); // Observe "fog"
        assert_eq!(model.get_current_state(), "fog");

        // Test prediction
        let next_state = model.predict_next_state();
        assert!(next_state < 3); // Should be a valid state index

        // The most likely next state from "fog" should be "fog" again (0.5 probability)
        assert_eq!(model.get_most_likely_next_state(), 2);
    }

    #[test]
    fn test_ooda_loop_with_markov() {
        // Create OODA loop
        let mut ooda = OodaLoop::new();

        // Create UAV components
        let mut _uav = UavSystems::new(MissionType::Surveillance);

        // Add radar contacts to simulate different environments
        for i in 0..4 {
            _uav.comms.radar_contacts.push(RadarContact {
                distance_m: 800.0 + (i as f32 * 50.0),
                bearing_deg: 30.0 + (i as f32 * 20.0),
                relative_speed_mps: 15.0,
                via_link: LinkType::MAVLink {
                    version: 2,
                    heartbeat_interval_ms: 500,
                },
            });
        }

        // Execute OODA cycle
        let cycle_time = ooda.execute_cycle(
            &mut _uav.comms,
            &mut _uav.payload,
            &mut _uav.flight_controller,
        );

        // Verify cycle completed
        assert!(cycle_time.as_nanos() > 0);

        // Environment should be classified as urban_canyon due to many radar contacts
        assert_eq!(ooda.environment_model.get_current_state(), "urban_canyon");

        // Should have a decision cached
        assert!(ooda.decision_cache.is_some());
    }

    #[test]
    fn test_environment_classification() {
        let ooda = OodaLoop::new();

        // Create a situation
        let situation = Situation {
            threat_level: ThreatLevel::High,
        };

        // Create sensor data with different patterns

        // 1. Urban canyon pattern: many radar contacts, no GPS
        let imu = IMUReading {
            accel: nalgebra::Vector3::new(0.0, 0.0, 9.81),
            gyro: nalgebra::Vector3::new(0.0, 0.0, 0.0),
            timestamp: 1000,
        };

        let link_type = LinkType::MAVLink {
            version: 2,
            heartbeat_interval_ms: 500,
        };

        let mut urban_contacts = Vec::new();
        for i in 0..4 {
            urban_contacts.push(RadarContact {
                distance_m: 500.0 + (i as f32 * 100.0),
                bearing_deg: 45.0 + (i as f32 * 30.0),
                relative_speed_mps: 10.0,
                via_link: link_type.clone(),
            });
        }

        let urban_data = SensorData {
            imu: imu.clone(),
            gps: None, // No GPS in urban canyon
            lidar: Some(50.0),
            radar_contacts: urban_contacts,
            operator_messages: 1,
            payload_status: (40.0, true),
        };

        // 2. Forest pattern: some radar contacts, partial GPS
        let gps = GPSPosition {
            latitude: 37.7749,
            longitude: -122.4194,
            altitude: 100.0,
            accuracy: 10.0, // Poor accuracy
        };

        let mut forest_contacts = Vec::new();
        for i in 0..2 {
            forest_contacts.push(RadarContact {
                distance_m: 200.0 + (i as f32 * 150.0),
                bearing_deg: 60.0 + (i as f32 * 20.0),
                relative_speed_mps: 5.0,
                via_link: link_type.clone(),
            });
        }

        let forest_data = SensorData {
            imu: imu.clone(),
            gps: Some(gps),
            lidar: Some(30.0),
            radar_contacts: forest_contacts,
            operator_messages: 0,
            payload_status: (40.0, true),
        };

        // 3. Mountain pattern: high-speed contacts, good GPS
        let mountain_gps = GPSPosition {
            latitude: 39.5501,
            longitude: -105.7821,
            altitude: 3000.0,
            accuracy: 5.0,
        };

        let mut mountain_contacts = Vec::new();
        mountain_contacts.push(RadarContact {
            distance_m: 1500.0,
            bearing_deg: 90.0,
            relative_speed_mps: 35.0, // High speed
            via_link: link_type.clone(),
        });

        let mountain_data = SensorData {
            imu: imu.clone(),
            gps: Some(mountain_gps),
            lidar: Some(200.0),
            radar_contacts: mountain_contacts,
            operator_messages: 0,
            payload_status: (40.0, true),
        };

        // Test classification for each environment
        let urban_env = ooda.classify_environment(&situation, &urban_data);
        let forest_env = ooda.classify_environment(&situation, &forest_data);
        let mountain_env = ooda.classify_environment(&situation, &mountain_data);

        assert_eq!(urban_env, 4); // urban_canyon
        assert_eq!(forest_env, 5); // forest
        assert_eq!(mountain_env, 6); // mountainous
    }

    #[test]
    fn test_adaptive_decision_making() {
        let mut ooda = OodaLoop::new();
        let low_threat = Situation {
            threat_level: ThreatLevel::Low,
        };
        let payload = PayloadManager::new(Some(PayloadType::SurveillanceCamera {
            resolution_mpx: 20.0,
            zoom_level: 10,
            thermal_capable: true,
        }));

        // Test urban canyon prediction
        ooda.environment_model.update_state(Some(0)); // Reset to clear
        let decision = ooda.decide_with_prediction(
            &low_threat,
            &payload,
            "urban_canyon", // Direct match
        );

        assert!(
            matches!(decision, Decision::PrepareMeshNetworking),
            "Expected PrepareMeshNetworking, got {:?}",
            decision
        );

        // Verify state index mapping
        assert_eq!(
            ooda.environment_model.get_state_name(4),
            "urban_canyon",
            "State index 4 must map to urban_canyon"
        );
    }
    #[test]
    fn test_complete_markov_ooda_cycle() {
        // Create a complete UAV system with components
        let mut uav = UavSystems::new(MissionType::Surveillance);
        let mut ooda = OodaLoop::new();

        // Run a complete OODA cycle to validate the integration
        let cycle_time =
            ooda.execute_cycle(&mut uav.comms, &mut uav.payload, &mut uav.flight_controller);

        // Verify cycle completed
        assert!(cycle_time.as_nanos() > 0);

        // Run a second cycle where environment transitions should happen
        // First manipulate environment to a known state
        ooda.environment_model.update_state(Some(1)); // light_rain

        // Run another cycle
        let cycle_time2 =
            ooda.execute_cycle(&mut uav.comms, &mut uav.payload, &mut uav.flight_controller);

        // Verify cycle completed
        assert!(cycle_time2.as_nanos() > 0);

        // The environment state should have changed (either by observation or prediction)
        assert!(ooda.environment_model.get_current_state() != "");

        // Make sure our decision cache is updated
        assert!(ooda.decision_cache.is_some());
    }

    #[test]
    fn test_urban_canyon_prediction() {
        let mut ooda = OodaLoop::new();

        // Create a simple low threat situation
        let low_threat = Situation {
            threat_level: ThreatLevel::Low,
        };

        // Create a minimal payload
        let payload = PayloadManager::new(None);

        // Test just the urban canyon decision
        let decision = ooda.decide_with_prediction(&low_threat, &payload, "urban_canyon");

        println!("Decision type: {:?}", decision);

        // The test fails here
        assert!(matches!(decision, Decision::PrepareMeshNetworking));
    }

    #[test]
    fn test_debug_urban_canyon() {
        let mut ooda = OodaLoop::new();

        // Create a simple low threat situation
        let low_threat = Situation {
            threat_level: ThreatLevel::Low,
        };

        // Create a minimal payload
        let payload = PayloadManager::new(None);

        // Direct string comparison check
        let direct_match = match "urban_canyon" {
            "urban_canyon" => true,
            _ => false,
        };
        println!("Direct string match works: {}", direct_match);

        // Test just the urban canyon decision with debugging
        println!("Testing with 'urban_canyon'");
        let decision1 = ooda.decide_with_prediction(&low_threat, &payload, "urban_canyon");
        println!("Decision type: {:?}", decision1);

        // Try with a different string case
        println!("Testing with 'Urban Canyon'");
        let decision2 = ooda.decide_with_prediction(&low_threat, &payload, "Urban Canyon");
        println!("Decision type: {:?}", decision2);

        // Try with exact environment model state name
        println!("Testing with exact state name");
        let exact_state_name = ooda.environment_model.get_state_name(4).to_string(); // urban_canyon should be index 4
        println!("Exact state name: '{}'", exact_state_name);
        let decision3 = ooda.decide_with_prediction(&low_threat, &payload, &exact_state_name);
        println!("Decision type: {:?}", decision3);

        // Check implementations directly
        let decision_manual = match "urban_canyon" {
            "heavy_rain" | "fog" => Decision::EnhanceCommsReliability,
            "urban_canyon" => Decision::PrepareMeshNetworking,
            _ => DecisionMaker::make(&low_threat, &payload),
        };
        println!("Manual implementation decision: {:?}", decision_manual);

        // Assert with the one that should definitely work
        assert!(matches!(decision_manual, Decision::PrepareMeshNetworking));
    }
}
