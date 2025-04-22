use serde::{Serialize, Deserialize};

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
    current_payload: Option<PayloadType>,
    power_consumption_w: f32,
    operational: bool,
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
}