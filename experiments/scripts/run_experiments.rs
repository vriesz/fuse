// experiments/scripts/run_experiments.rs
use std::fs;
use serde_yaml;
use uav_arch_gen::{generate_architecture, models::UavConstraints};

struct Experiment {
    config: ExperimentConfig,
    generated_arch: UavArchitecture,
    metrics: ExperimentMetrics,
}

fn main() {
    let config_files = fs::read_dir("experiments/configs").unwrap();
    
    for config_file in config_files {
        let config: ExperimentConfig = serde_yaml::from_reader(fs::File::open(config_file?).unwrap();
        let constraints = convert_to_constraints(config);
        let arch = generate_architecture(&constraints);
        
        // Evaluate and save results
        let metrics = evaluate_architecture(&arch, &config);
        save_results(metrics, config.experiment_id);
    }
}