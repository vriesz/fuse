use crate::models::architecture::UavSystems;
use crate::comms::{RadarContact, NavigationBeacon, LinkType};

impl UavSystems {
    pub fn scan_surroundings(&mut self) -> Vec<RadarContact> {
        // Simulate finding some radar contacts
        let contacts = vec![
            RadarContact {
                distance_m: 1000.0,
                bearing_deg: 45.0,
                relative_speed_mps: 20.0,
                via_link: LinkType::MAVLink {
                    version: 2,
                    heartbeat_interval_ms: 500,
                },
            }
        ];
        
        // Add the contacts to the comms system
        self.comms.radar_contacts = contacts.clone();
        contacts
    }
    
    pub fn register_navigation_beacon(&mut self, id: &str, position: (f32, f32)) {
        self.comms.log_beacon(NavigationBeacon {
            id: id.to_string(),
            position,
            signal_strength: 0.85,
            link_used: LinkType::LoRa {
                frequency_mhz: 915,
                spreading_factor: 10,
            },
        });
    }
}