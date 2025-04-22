use uav_arch_gen::models::{UavConstraints, SWaPConstraints, MissionType};
use uav_arch_gen::engine::generate_architecture;

fn main() {
    let constraints = UavConstraints {
        mission: MissionType::Surveillance,
        swap: SWaPConstraints {
            max_weight_kg: 5.0,
            max_power_w: 50.0,
            max_size_cm: (80.0, 80.0, 30.0),
            min_compute_threshold: Some(0.5),
            max_cost: Some(3000.0),
        },
        autonomy_level: 4,
        requires_ai: true,
        secure_comms: false,
    };

    let arch = generate_architecture(&constraints);
    println!("Generated Architecture: {:?}", arch);
}