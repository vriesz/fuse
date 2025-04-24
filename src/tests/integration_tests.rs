#[cfg(test)]
mod tests {
    use crate::*;
    use crate::models::constraints::*;
    use crate::models::components::*;
    use crate::models::architecture::*;
    use crate::payload::*;
    use crate::comms::*;
    use crate::ooda::*;
    use std::time::Duration;
    
    #[test]
    fn test_full_system_integration() {
        // 1. Generate architecture based on constraints
        let mut constraints = UavConstraints::default();
        constraints.mission = MissionType::Surveillance;
        constraints.requires_ai = true;
        
        let architecture = generate_architecture(&constraints);
        
        // 2. Create a UAV system from the architecture
        let mut uav_system = UavSystems::new(constraints.mission);
        
        // 3. Configure flight controller based on architecture
        let fc_type = architecture.flight_control.clone();
        uav_system.flight_controller = fc_type.into();
        
        // 4. Run OODA loop for a few cycles
        let mut ooda_loop = OodaLoop::new();
        
        // Add some radar contacts to simulate environment
        uav_system.scan_surroundings();
        
        // Run OODA cycle
        let cycle_time = ooda_loop.execute_cycle(
            &mut uav_system.comms, 
            &mut uav_system.payload, 
            &mut uav_system.flight_controller
        );
        
        // 5. Verify system behavior
        assert!(cycle_time > Duration::from_nanos(0));
        
        // High threat level should have activated the payload
        assert_eq!(uav_system.payload.get_status().1, true);
        
        // Verify comms adjusted based on OODA cycle time
        let priority = uav_system.comms.process_ooda_cycle(cycle_time);
        assert!(matches!(priority, CommsPriority::High) || 
                matches!(priority, CommsPriority::Medium) || 
                matches!(priority, CommsPriority::Low));
    }
    
    #[test]
    fn test_surveillance_mission_profile() {
        // Create a surveillance mission
        let mut uav = UavSystems::new(MissionType::Surveillance);
        
        // Verify surveillance-specific configuration
        match &uav.payload.current_payload {
            Some(PayloadType::SurveillanceCamera { .. }) => (),
            _ => panic!("Expected surveillance camera for surveillance mission"),
        }
        
        // Add operators
        uav.comms.add_operator(
            "Commander".to_string(),
            5,
            vec![LinkType::MAVLink {
                version: 2,
                heartbeat_interval_ms: 500,
            }]
        );
        
        // Register navigation beacon
        uav.register_navigation_beacon("NAV-1", (34.5, -118.2));
        
        // Run an OODA cycle
        let mut ooda = OodaLoop::new();
        ooda.execute_cycle(&mut uav.comms, &mut uav.payload, &mut uav.flight_controller);
        
        // Verify payload was activated
        assert_eq!(uav.payload.get_status().1, true);
    }
    
    #[test]
    fn test_architecture_changes_based_on_constraints() {
        // Test base case
        let base_constraints = UavConstraints::default();
        let base_arch = generate_architecture(&base_constraints);
        
        // Test secure communications
        let mut secure_constraints = base_constraints.clone();
        secure_constraints.secure_comms = true;
        let secure_arch = generate_architecture(&secure_constraints);
        
        assert_ne!(base_arch.processor, secure_arch.processor);
        assert_ne!(base_arch.comms, secure_arch.comms);
        
        // Test AI requirements
        let mut ai_constraints = base_constraints.clone();
        ai_constraints.requires_ai = true;
        let ai_arch = generate_architecture(&ai_constraints);
        
        assert_ne!(base_arch.processor, ai_arch.processor);
        assert_ne!(base_arch.data_fusion, ai_arch.data_fusion);
        
        // Test autonomy levels
        let mut low_autonomy = base_constraints.clone();
        low_autonomy.autonomy_level = 1;
        let low_arch = generate_architecture(&low_autonomy);
        
        let mut high_autonomy = base_constraints.clone();
        high_autonomy.autonomy_level = 5;
        let high_arch = generate_architecture(&high_autonomy);
        
        assert_ne!(low_arch.flight_control, high_arch.flight_control);
    }
    
    #[test]
    fn test_serialization_roundtrip() {
        // Generate architecture and serialize
        let constraints = UavConstraints::default();
        let original_arch = generate_architecture(&constraints);
        
        let serialized = serde_yaml::to_string(&original_arch).expect("Failed to serialize architecture");
        let deserialized: UavArchitecture = serde_yaml::from_str(&serialized).expect("Failed to deserialize architecture");
        
        // Verify key components remained unchanged
        assert_eq!(original_arch.processor, deserialized.processor);
        assert_eq!(original_arch.sensors, deserialized.sensors);
        assert_eq!(original_arch.comms, deserialized.comms);
        
        // Create full UAV system and serialize
        let original_system = UavSystems::new(MissionType::Surveillance);
        
        let system_serialized = serde_yaml::to_string(&original_system).expect("Failed to serialize UAV system");
        let system_deserialized: UavSystems = serde_yaml::from_str(&system_serialized).expect("Failed to deserialize UAV system");
        
        // Verify UAV system components
        match &system_deserialized.payload.current_payload {
            Some(PayloadType::SurveillanceCamera { .. }) => (),
            _ => panic!("Expected surveillance camera after deserialization"),
        }
        
        // Flight controller should match
        assert_eq!(original_system.flight_controller.mode, system_deserialized.flight_controller.mode);
    }
}