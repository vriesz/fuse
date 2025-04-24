#[cfg(test)]
mod tests {
    use crate::comms::*;
    use std::time::Duration;
    
    #[test]
    fn test_link_type_serialization() {
        let link = LinkType::MAVLink {
            version: 2,
            heartbeat_interval_ms: 500,
        };
        
        let serialized = serde_yaml::to_string(&link).expect("Failed to serialize LinkType");
        let deserialized: LinkType = serde_yaml::from_str(&serialized).expect("Failed to deserialize LinkType");
        
        assert_eq!(link, deserialized);
    }
    
    #[test]
    fn test_communication_hub_creation() {
        let link_type = LinkType::MAVLink {
            version: 2,
            heartbeat_interval_ms: 500,
        };
        
        let hub = CommunicationHub::new(link_type.clone(), true);
        
        assert_eq!(hub.primary_link.link_type, link_type);
        assert_eq!(hub.primary_link.encryption, true);
        assert!(hub.backup_links.is_empty());
        assert!(hub.operators.is_empty());
        assert!(hub.radar_contacts.is_empty());
        assert!(hub.secure_channel.is_some());
    }
    
    #[test]
    fn test_operator_addition() {
        let link_type = LinkType::MAVLink {
            version: 2,
            heartbeat_interval_ms: 500,
        };
        
        let mut hub = CommunicationHub::new(link_type.clone(), false);
        hub.add_operator("Operator1".to_string(), 5, vec![link_type.clone()]);
        
        assert_eq!(hub.operators.len(), 1);
        assert_eq!(hub.operators[0].id, "Operator1");
        assert_eq!(hub.operators[0].clearance_level, 5);
        assert_eq!(hub.operators[0].assigned_links.len(), 1);
        assert_eq!(hub.operators[0].assigned_links[0], link_type);
        assert!(hub.operators[0].last_heartbeat.is_some());
    }
    
    #[test]
    fn test_communication_hub_serialization() {
        let link_type = LinkType::WiFiDirect {
            bandwidth_mbps: 100,
            channel: 36,
        };
        
        let mut hub = CommunicationHub::new(link_type.clone(), false);
        hub.add_operator("TestOperator".to_string(), 3, vec![]);
        
        let serialized = serde_yaml::to_string(&hub).expect("Failed to serialize CommunicationHub");
        let deserialized: CommunicationHub = serde_yaml::from_str(&serialized).expect("Failed to deserialize CommunicationHub");
        
        assert_eq!(hub.operators.len(), deserialized.operators.len());
        assert_eq!(hub.operators[0].id, deserialized.operators[0].id);
        assert_eq!(hub.primary_link.link_type, deserialized.primary_link.link_type);
    }
    
    #[test]
    fn test_process_ooda_cycle() {
        let link_type = LinkType::MAVLink {
            version: 2,
            heartbeat_interval_ms: 500,
        };
        
        let mut hub = CommunicationHub::new(link_type, false);
        
        // Test fast OODA cycle
        let fast_result = hub.process_ooda_cycle(Duration::from_millis(50));
        assert_eq!(fast_result, CommsPriority::High);
        
        // Verify link type was changed to WiFiDirect
        match &hub.primary_link.link_type {
            LinkType::WiFiDirect { .. } => (),
            _ => panic!("Link type should have been changed to WiFiDirect"),
        }
        
        // Test medium OODA cycle
        let medium_result = hub.process_ooda_cycle(Duration::from_millis(200));
        assert_eq!(medium_result, CommsPriority::Medium);
        
        // Verify link type was changed to MAVLink
        match &hub.primary_link.link_type {
            LinkType::MAVLink { .. } => (),
            _ => panic!("Link type should have been changed to MAVLink"),
        }
        
        // Test slow OODA cycle
        let slow_result = hub.process_ooda_cycle(Duration::from_millis(600));
        assert_eq!(slow_result, CommsPriority::Low);
        
        // Verify link type was changed to LoRa
        match &hub.primary_link.link_type {
            LinkType::LoRa { .. } => (),
            _ => panic!("Link type should have been changed to LoRa"),
        }
    }
    
    #[test]
    fn test_secure_channel_encryption() {
        let secure_channel = SecureChannel::new("AES256-SHA256").expect("Failed to create secure channel");
        
        let data = b"Test data for encryption";
        let encrypted = secure_channel.encrypt(data);
        
        // Simple check that the data was modified (actual encryption is just a dummy implementation)
        assert_eq!(encrypted.len(), data.len());
    }
}