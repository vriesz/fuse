use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionType {
    Surveillance,
    Strike,
    Logistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SWaPConstraints {
    pub max_weight_kg: f32,
    pub max_power_w: f32,
    pub max_size_cm: (f32, f32, f32),
    pub min_compute_threshold: Option<f32>,
    pub max_cost: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UavConstraints {
    pub mission: MissionType,
    pub swap: SWaPConstraints,
    pub autonomy_level: u8,
    pub requires_ai: bool,
    pub secure_comms: bool,
}

impl Default for UavConstraints {
    fn default() -> Self {
        Self {
            mission: MissionType::Surveillance,
            swap: SWaPConstraints {
                max_weight_kg: 10.0,
                max_power_w: 100.0,
                max_size_cm: (100.0, 100.0, 50.0),
                min_compute_threshold: Some(1.0),
                max_cost: Some(5000.0),
            },
            autonomy_level: 3,
            requires_ai: false,
            secure_comms: false,
        }
    }
}