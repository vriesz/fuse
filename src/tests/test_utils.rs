use uav_arch_gen::models::{UavConstraints, SWaPConstraints, MissionType};

#[allow(dead_code)]
pub fn default_test_constraints() -> UavConstraints {
    UavConstraints {
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