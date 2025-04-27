// src/comms/ooda_integration.rs


impl CommunicationHub {
    pub fn process_ooda_cycle(&mut self, ooda: &OodaLoop) -> CommsPriority {
        let bandwidth_needed = match ooda.last_cycle_time {
            t if t < Duration::from_millis(100) => CommsPriority::High,
            t if t < Duration::from_millis(500) => CommsPriority::Medium,
            _ => CommsPriority::Low,
        };
        
        self.adjust_links(bandwidth_needed);
        bandwidth_needed
    }

    fn adjust_links(&mut self, priority: CommsPriority) {
        match priority {
            CommsPriority::High => {
                self.primary_link = LinkType::WiFiDirect {
                    bandwidth_mbps: 100,
                    channel: 36,
                };
            },
            CommsPriority::Medium => {
                self.primary_link = LinkType::MAVLink {
                    version: 2,
                    heartbeat_interval_ms: 500,
                };
            },
            CommsPriority::Low => {
                self.primary_link = LinkType::LoRa {
                    frequency_mhz: 915,
                    spreading_factor: 10,
                };
            }
        }
    }
}