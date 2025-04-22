impl PayloadManager {
    pub fn ooda_configure(&mut self, situation: &Situation) {
        match situation.threat_level {
            ThreatLevel::High => {
                if let Some(PayloadType::SurveillanceCamera { .. }) = &self.current_payload {
                    self.set_high_alert_mode(true);
                }
            },
            ThreatLevel::Medium => {
                self.activate();
            },
            ThreatLevel::Low => {
                if self.power_consumption_w > 50.0 {
                    self.standby();
                }
            }
        }
    }
}