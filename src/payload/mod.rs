use serde::{Serialize, Deserialize};
use crate::sensor_fusion::{Situation, ThreatLevel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PayloadType {
    SurveillanceCamera {
        resolution_mpx: f32,
        zoom_level: u8,
        thermal_capable: bool,
    },
    LidarScanner {
        range_m: f32,
        point_cloud_density: u32,
    },
    CargoContainer {
        max_weight_kg: f32,
        secure_locking: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadManager {
    pub current_payload: Option<PayloadType>,
    pub power_consumption_w: f32,
    pub operational: bool,
}

impl PayloadManager {
    pub fn new(payload: Option<PayloadType>) -> Self {
        Self {
            current_payload: payload,
            power_consumption_w: 0.0,
            operational: false,
        }
    }

    pub fn activate(&mut self) {
        if let Some(payload) = &self.current_payload {
            self.power_consumption_w = match payload {
                PayloadType::SurveillanceCamera { .. } => 45.5,
                PayloadType::LidarScanner { .. } => 120.0,
                PayloadType::CargoContainer { .. } => 5.0,
            };
            self.operational = true;
        }
    }

    pub fn get_status(&self) -> (f32, bool) {
        (self.power_consumption_w, self.operational)
    }
    
    pub fn toggle_operational(&mut self) {
        self.operational = !self.operational;
        if !self.operational {
            self.power_consumption_w = 0.0;
        } else {
            self.activate();
        }
    }

    pub fn standby(&mut self) {
        self.operational = false;
        self.power_consumption_w *= 0.3; // Reduce to 30%
    }

    pub fn set_high_alert_mode(&mut self, enabled: bool) {
        if enabled {
            self.activate();
            // Increase power if needed for high alert mode
            self.power_consumption_w *= 1.2;
        }
    }
    
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