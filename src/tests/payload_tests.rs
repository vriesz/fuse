// src/tests/payload_tests.rs

#[cfg(test)]
mod tests {
    use crate::payload::*;
    use crate::sensor_fusion::{Situation, ThreatLevel};
    use approx::assert_relative_eq;

    #[test]
    fn test_payload_manager_creation() {
        let payload = PayloadManager::new(None);
        
        assert!(payload.current_payload.is_none());
        assert_eq!(payload.power_consumption_w, 0.0);
        assert_eq!(payload.operational, false);
    }
    
    #[test]
    fn test_payload_manager_with_camera() {
        let camera = PayloadType::SurveillanceCamera {
            resolution_mpx: 20.0,
            zoom_level: 10,
            thermal_capable: true,
        };
        
        let payload = PayloadManager::new(Some(camera));
        assert!(payload.current_payload.is_some());
        assert_eq!(payload.power_consumption_w, 0.0);
        assert_eq!(payload.operational, false);
    }
    
    #[test]
    fn test_payload_activation() {
        let camera = PayloadType::SurveillanceCamera {
            resolution_mpx: 20.0,
            zoom_level: 10,
            thermal_capable: true,
        };
        
        let mut payload = PayloadManager::new(Some(camera.clone()));
        payload.activate();
        
        assert_eq!(payload.operational, true);
        assert_eq!(payload.power_consumption_w, 45.5);
    }
    
    #[test]
    fn test_payload_toggle() {
        let lidar = PayloadType::LidarScanner {
            range_m: 1000.0,
            point_cloud_density: 50000,
        };
        
        let mut payload = PayloadManager::new(Some(lidar));
        
        // Initially off
        assert_eq!(payload.operational, false);
        
        // Turn on
        payload.toggle_operational();
        assert_eq!(payload.operational, true);
        assert!(payload.power_consumption_w > 0.0);
        
        // Turn off
        payload.toggle_operational();
        assert_eq!(payload.operational, false);
        assert_eq!(payload.power_consumption_w, 0.0);
    }
    
    #[test]
    fn test_payload_standby() {
        let lidar = PayloadType::LidarScanner {
            range_m: 1000.0,
            point_cloud_density: 50000,
        };
        
        let mut payload = PayloadManager::new(Some(lidar.clone()));
        payload.activate();
        
        let initial_power = payload.power_consumption_w;
        payload.standby();
        
        assert_eq!(payload.operational, false);
        assert_relative_eq!(payload.power_consumption_w, initial_power * 0.3);
    }
    
    #[test]
    fn test_payload_ooda_configuration() {
        let camera = PayloadType::SurveillanceCamera {
            resolution_mpx: 20.0,
            zoom_level: 10,
            thermal_capable: true,
        };
        
        let mut payload = PayloadManager::new(Some(camera.clone()));
        
        // Test high threat level response
        let high_threat = Situation {
            threat_level: ThreatLevel::High,
        };
        
        payload.ooda_configure(&high_threat);
        assert_eq!(payload.operational, true);
        assert!(payload.power_consumption_w > 45.5);
        
        // Test medium threat level response
        let mut payload = PayloadManager::new(Some(camera.clone()));
        let medium_threat = Situation {
            threat_level: ThreatLevel::Medium,
        };
        
        payload.ooda_configure(&medium_threat);
        assert_eq!(payload.operational, true);
        assert_relative_eq!(payload.power_consumption_w, 45.5);
        
        // Test low threat level response
        let mut payload = PayloadManager::new(Some(camera.clone()));
        payload.activate();
        payload.power_consumption_w = 60.0;
        
        let low_threat = Situation {
            threat_level: ThreatLevel::Low,
        };
        
        payload.ooda_configure(&low_threat);
        assert_eq!(payload.operational, false);
        assert!(payload.power_consumption_w < 60.0);
    }
    
    #[test]
    fn test_payload_serialization() {
        let camera = PayloadType::SurveillanceCamera {
            resolution_mpx: 20.0,
            zoom_level: 10,
            thermal_capable: true,
        };
        
        let mut payload = PayloadManager::new(Some(camera.clone()));
        payload.activate();
        
        let serialized = serde_yaml::to_string(&payload).expect("Failed to serialize PayloadManager");
        let deserialized: PayloadManager = serde_yaml::from_str(&serialized).expect("Failed to deserialize PayloadManager");
        
        assert_eq!(payload.operational, deserialized.operational);
        assert_eq!(payload.power_consumption_w, deserialized.power_consumption_w);
        
        match &deserialized.current_payload {
            Some(PayloadType::SurveillanceCamera { resolution_mpx, zoom_level, thermal_capable }) => {
                assert_eq!(*resolution_mpx, 20.0);
                assert_eq!(*zoom_level, 10);
                assert_eq!(*thermal_capable, true);
            },
            _ => panic!("Wrong payload type after deserialization"),
        }
    }
}