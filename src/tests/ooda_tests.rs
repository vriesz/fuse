// src/tests/ooda_tests.rs

#[cfg(test)]
mod tests {
    use crate::ooda::*;
    use crate::comms::*;
    use crate::payload::*;
    use crate::flight_control::*;
    use crate::sensor_fusion::*;
    
    #[test]
    fn test_ooda_loop_creation() {
        let ooda = OodaLoop::new();
        
        assert_eq!(ooda.last_cycle_time.as_secs(), 0);
        assert_eq!(ooda.last_cycle_time.subsec_nanos(), 0);
        assert!(ooda.decision_cache.is_none());
    }
    
    #[test]
    fn test_decision_matching() {
        let altitude_decision = Decision::ChangeAltitude(100.0);
        let mode_decision = Decision::SwitchPayloadMode;
        
        let high_threat = Situation {
            threat_level: ThreatLevel::High,
        };
        
        let medium_threat = Situation {
            threat_level: ThreatLevel::Medium,
        };
        
        let low_threat = Situation {
            threat_level: ThreatLevel::Low,
        };
        
        // Check correct matches
        assert!(altitude_decision.matches(&high_threat));
        assert!(mode_decision.matches(&medium_threat));
        assert!(mode_decision.matches(&low_threat));
        
        // Check incorrect matches
        assert!(!altitude_decision.matches(&medium_threat));
        assert!(!altitude_decision.matches(&low_threat));
        assert!(!mode_decision.matches(&high_threat));
    }
    
    #[test]
    fn test_decision_maker() {
        let camera = PayloadType::SurveillanceCamera {
            resolution_mpx: 20.0,
            zoom_level: 10,
            thermal_capable: true,
        };
        
        let payload = PayloadManager::new(Some(camera));
        
        // Test high threat decision
        let high_threat = Situation {
            threat_level: ThreatLevel::High,
        };
        
        let high_decision = DecisionMaker::make(&high_threat, &payload);
        match high_decision {
            Decision::ChangeAltitude(_) => (),
            _ => panic!("Expected ChangeAltitude decision for high threat"),
        }
        
        // Test medium threat decision
        let medium_threat = Situation {
            threat_level: ThreatLevel::Medium,
        };
        
        let medium_decision = DecisionMaker::make(&medium_threat, &payload);
        match medium_decision {
            Decision::SwitchPayloadMode => (),
            _ => panic!("Expected SwitchPayloadMode decision for medium threat"),
        }
    }
    
    #[test]
    fn test_ooda_cycle_execution() {
        // Set up components
        let link_type = LinkType::MAVLink { 
            version: 2, 
            heartbeat_interval_ms: 500
        };
        
        let mut comms = CommunicationHub::new(link_type.clone(), false);
        
        let camera = PayloadType::SurveillanceCamera {
            resolution_mpx: 20.0,
            zoom_level: 10,
            thermal_capable: true,
        };
        
        let mut payload = PayloadManager::new(Some(camera));
        let mut flight_controller = FlightController::new();
        
        let mut ooda = OodaLoop::new();
        
        // Add 3 radar contacts to trigger high threat
        comms.radar_contacts = vec![
            RadarContact {
                distance_m: 1000.0,
                bearing_deg: 45.0,
                relative_speed_mps: 20.0,
                via_link: link_type.clone(),
            },
            RadarContact {
                distance_m: 800.0,
                bearing_deg: 90.0,
                relative_speed_mps: 15.0,
                via_link: link_type.clone(),
            },
            RadarContact {
                distance_m: 1200.0,
                bearing_deg: 180.0,
                relative_speed_mps: 25.0,
                via_link: link_type.clone(),
            }
        ];
        
        // Execute OODA cycle
        let duration = ooda.execute_cycle(&mut comms, &mut payload, &mut flight_controller);
        
        // Check that the cycle completed and took some time
        assert!(duration.as_nanos() > 0);
        
        // Check that a decision was cached
        assert!(ooda.decision_cache.is_some());
        
        // For high threat, we expect flight controller to be called to change altitude
        match &ooda.decision_cache {
            Some(Decision::ChangeAltitude(_)) => (),
            _ => panic!("Expected ChangeAltitude decision for high threat"),
        }
    }
}