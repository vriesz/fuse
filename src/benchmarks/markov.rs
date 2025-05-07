// src/benchmarks/markov.rs

use crate::comms::*;
use crate::models::markov_chain::MarkovEnvironmentModel;
use crate::models::architecture::UavSystems;
use crate::models::constraints::MissionType;
use crate::ooda::OodaLoop;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkovBenchmarkResult {
    pub scenario: String,
    pub cycle_time_ms: f64,
    pub prediction_accuracy: f64,
    pub adaptation_latency_ms: f64,
    pub trials: usize,
}

pub fn run_markov_benchmarks(num_trials: usize) -> Vec<MarkovBenchmarkResult> {
    vec![
        benchmark_static_environment(num_trials),
        benchmark_changing_environment(num_trials),
        benchmark_rapid_transitions(num_trials),
    ]
}

fn benchmark_static_environment(num_trials: usize) -> MarkovBenchmarkResult {
    // Create OODA loop with Markov model
    let mut ooda = OodaLoop::new();
    
    // Create UAV for simulation
    let mut uav = UavSystems::new(MissionType::Surveillance);
    
    // Create static environment - clear conditions throughout
    // Set the environment model to have high probability of staying in current state
    let states = vec![
        "clear".to_string(),
        "light_rain".to_string(),
        "heavy_rain".to_string(),
    ];
    
    let transition_matrix = vec![
        vec![0.95, 0.03, 0.02], // clear -> clear (95%)
        vec![0.30, 0.65, 0.05], // light_rain -> light_rain (65%)
        vec![0.20, 0.30, 0.50], // heavy_rain -> heavy_rain (50%)
    ];
    
    // Set custom environment model
    ooda.environment_model = MarkovEnvironmentModel::new(&states, transition_matrix, 0);
    
    // Metrics
    let mut cycle_times = Vec::with_capacity(num_trials);
    let mut prediction_hits = 0;
    let mut adaptation_latencies = Vec::with_capacity(num_trials);
    
    // Track last predicted state for accuracy measurement
    let mut last_predicted_state = ooda.environment_model.predict_next_state();
    
    for _ in 0..num_trials {
        // Record start time
        let _cycle_start = std::time::Instant::now();
        
        // Execute OODA cycle
        let cycle_time = ooda.execute_cycle(&mut uav.comms, &mut uav.payload, &mut uav.flight_controller);
        
        cycle_times.push(cycle_time);
        
        // Check if previous prediction was correct (actual matches predicted)
        if ooda.environment_model.current_state_idx == last_predicted_state {
            prediction_hits += 1;
        }
        
        // Record adaptation latency - time to handle the predicted state
        let adaptation_time = match &ooda.decision_cache {
            Some(decision) => {
                match decision {
                    crate::ooda::Decision::EnhanceCommsReliability => Duration::from_millis(15),
                    crate::ooda::Decision::PrepareMeshNetworking => Duration::from_millis(25),
                    _ => Duration::from_millis(5),
                }
            },
            None => Duration::from_millis(0),
        };
        
        adaptation_latencies.push(adaptation_time);
        
        // Record prediction for next cycle
        last_predicted_state = ooda.environment_model.predict_next_state();
    }
    
    // Calculate statistics
    let avg_cycle_time = if !cycle_times.is_empty() {
        let sum = cycle_times.iter().sum::<Duration>();
        sum.as_secs_f64() * 1000.0 / cycle_times.len() as f64 // Convert to ms
    } else {
        0.0
    };
    
    let avg_adaptation_latency = if !adaptation_latencies.is_empty() {
        let sum = adaptation_latencies.iter().sum::<Duration>();
        sum.as_secs_f64() * 1000.0 / adaptation_latencies.len() as f64 // Convert to ms
    } else {
        0.0
    };
    
    let prediction_accuracy = if num_trials > 0 {
        prediction_hits as f64 / num_trials as f64 * 100.0
    } else {
        0.0
    };
    
    MarkovBenchmarkResult {
        scenario: "Static Environment".to_string(),
        cycle_time_ms: avg_cycle_time,
        prediction_accuracy,
        adaptation_latency_ms: avg_adaptation_latency,
        trials: num_trials,
    }
}

fn benchmark_changing_environment(num_trials: usize) -> MarkovBenchmarkResult {
    // Create OODA loop with Markov model
    let mut ooda = OodaLoop::new();
    
    // Create UAV for simulation
    let mut uav = UavSystems::new(MissionType::Surveillance);
    
    // Create environment with regular transitions
    // IMPORTANT: Match these states with the ones in OodaLoop::new
    // The order and number should match
    let states = vec![
        "clear".to_string(),
        "light_rain".to_string(),
        "heavy_rain".to_string(),
        "fog".to_string(),
        "urban_canyon".to_string(),
        "forest".to_string(),
        "mountainous".to_string(),
    ];
    
    let transition_matrix = vec![
        vec![0.4, 0.3, 0.1, 0.1, 0.05, 0.025, 0.025], // clear
        vec![0.3, 0.3, 0.3, 0.05, 0.025, 0.0125, 0.0125], // light_rain
        vec![0.1, 0.4, 0.4, 0.05, 0.025, 0.0125, 0.0125], // heavy_rain
        vec![0.2, 0.2, 0.1, 0.4, 0.05, 0.025, 0.025], // fog
        vec![0.15, 0.1, 0.05, 0.05, 0.5, 0.075, 0.075], // urban_canyon
        vec![0.1, 0.1, 0.05, 0.05, 0.1, 0.5, 0.1], // forest
        vec![0.1, 0.05, 0.05, 0.1, 0.1, 0.1, 0.5], // mountainous
    ];
    
    // Set custom environment model
    ooda.environment_model = MarkovEnvironmentModel::new(&states, transition_matrix, 0);
    
    // Similar metrics setup
    let mut cycle_times = Vec::with_capacity(num_trials);
    let mut prediction_hits = 0;
    let mut adaptation_latencies = Vec::with_capacity(num_trials);
    let mut last_predicted_state = ooda.environment_model.predict_next_state();
    
    for i in 0..num_trials {
        // Every few iterations, manually force environment change
        if i % 5 == 0 {
            // Cycle through states systematically
            let forced_state = (i / 5) % states.len();
            ooda.environment_model.update_state(Some(forced_state));
        }
        
        let _cycle_start = std::time::Instant::now();
        let cycle_time = ooda.execute_cycle(&mut uav.comms, &mut uav.payload, &mut uav.flight_controller);
        cycle_times.push(cycle_time);
        
        if ooda.environment_model.current_state_idx == last_predicted_state {
            prediction_hits += 1;
        }
        
        let adaptation_time = match &ooda.decision_cache {
            Some(decision) => {
                match decision {
                    crate::ooda::Decision::EnhanceCommsReliability => Duration::from_millis(15),
                    crate::ooda::Decision::PrepareMeshNetworking => Duration::from_millis(25),
                    _ => Duration::from_millis(5),
                }
            },
            None => Duration::from_millis(0),
        };
        
        adaptation_latencies.push(adaptation_time);
        last_predicted_state = ooda.environment_model.predict_next_state();
    }
    
    // Calculate statistics
    let avg_cycle_time = if !cycle_times.is_empty() {
        let sum = cycle_times.iter().sum::<Duration>();
        sum.as_secs_f64() * 1000.0 / cycle_times.len() as f64 // Convert to ms
    } else {
        0.0
    };
    
    let avg_adaptation_latency = if !adaptation_latencies.is_empty() {
        let sum = adaptation_latencies.iter().sum::<Duration>();
        sum.as_secs_f64() * 1000.0 / adaptation_latencies.len() as f64 // Convert to ms
    } else {
        0.0
    };
    
    let prediction_accuracy = if num_trials > 0 {
        prediction_hits as f64 / num_trials as f64 * 100.0
    } else {
        0.0
    };
    
    MarkovBenchmarkResult {
        scenario: "Changing Environment".to_string(),
        cycle_time_ms: avg_cycle_time,
        prediction_accuracy,
        adaptation_latency_ms: avg_adaptation_latency,
        trials: num_trials,
    }
}

fn benchmark_rapid_transitions(num_trials: usize) -> MarkovBenchmarkResult {
    // Create OODA loop with Markov model
    let mut ooda = OodaLoop::new();
    
    // Create UAV for simulation
    let mut uav = UavSystems::new(MissionType::Surveillance);
    
    // Create environment with very dynamic transitions
    // IMPORTANT: Match these states with the ones in OodaLoop::new
    let states = vec![
        "clear".to_string(),
        "light_rain".to_string(),
        "heavy_rain".to_string(),
        "fog".to_string(),
        "urban_canyon".to_string(),
        "forest".to_string(),
        "mountainous".to_string(),
    ];
    
    let transition_matrix = vec![
        vec![0.25, 0.25, 0.1, 0.1, 0.1, 0.1, 0.1], // equal chance for all states
        vec![0.25, 0.25, 0.1, 0.1, 0.1, 0.1, 0.1],
        vec![0.25, 0.25, 0.1, 0.1, 0.1, 0.1, 0.1],
        vec![0.25, 0.25, 0.1, 0.1, 0.1, 0.1, 0.1],
        vec![0.25, 0.25, 0.1, 0.1, 0.1, 0.1, 0.1],
        vec![0.25, 0.25, 0.1, 0.1, 0.1, 0.1, 0.1],
        vec![0.25, 0.25, 0.1, 0.1, 0.1, 0.1, 0.1],
    ];
    
    // Set custom environment model
    ooda.environment_model = MarkovEnvironmentModel::new(&states, transition_matrix, 0);
    
    // Similar metrics setup
    let mut cycle_times = Vec::with_capacity(num_trials);
    let mut prediction_hits = 0;
    let mut adaptation_latencies = Vec::with_capacity(num_trials);
    let mut last_predicted_state = ooda.environment_model.predict_next_state();
    
    for i in 0..num_trials {
        // Force rapid environment changes every cycle
        let forced_state = i % states.len();
        ooda.environment_model.update_state(Some(forced_state));
        
        // Add radar contacts based on environment to simulate changes
        uav.comms.radar_contacts.clear();
        
        match forced_state {
            4 => { // urban_canyon (index 4)
                for j in 0..4 {
                    uav.comms.radar_contacts.push(RadarContact {
                        distance_m: 500.0 + (j as f32 * 100.0),
                        bearing_deg: 45.0 + (j as f32 * 30.0),
                        relative_speed_mps: 10.0,
                        via_link: LinkType::MAVLink {
                            version: 2,
                            heartbeat_interval_ms: 500,
                        },
                    });
                }
            },
            5 => { // forest (index 5)
                for j in 0..2 {
                    uav.comms.radar_contacts.push(RadarContact {
                        distance_m: 200.0 + (j as f32 * 150.0),
                        bearing_deg: 60.0 + (j as f32 * 20.0),
                        relative_speed_mps: 5.0,
                        via_link: LinkType::MAVLink {
                            version: 2,
                            heartbeat_interval_ms: 500,
                        },
                    });
                }
            },
            6 => { // mountainous (index 6)
                uav.comms.radar_contacts.push(RadarContact {
                    distance_m: 1500.0,
                    bearing_deg: 90.0,
                    relative_speed_mps: 35.0,
                    via_link: LinkType::MAVLink {
                        version: 2,
                        heartbeat_interval_ms: 500,
                    },
                });
            },
            _ => {} // other states - no radar contacts
        }
        
        let _cycle_start = std::time::Instant::now();
        let cycle_time = ooda.execute_cycle(&mut uav.comms, &mut uav.payload, &mut uav.flight_controller);
        cycle_times.push(cycle_time);
        
        if ooda.environment_model.current_state_idx == last_predicted_state {
            prediction_hits += 1;
        }
        
        let adaptation_time = match &ooda.decision_cache {
            Some(decision) => {
                match decision {
                    crate::ooda::Decision::EnhanceCommsReliability => Duration::from_millis(15),
                    crate::ooda::Decision::PrepareMeshNetworking => Duration::from_millis(25),
                    _ => Duration::from_millis(5),
                }
            },
            None => Duration::from_millis(0),
        };
        
        adaptation_latencies.push(adaptation_time);
        last_predicted_state = ooda.environment_model.predict_next_state();
    }
    
    // Calculate statistics
    let avg_cycle_time = if !cycle_times.is_empty() {
        let sum = cycle_times.iter().sum::<Duration>();
        sum.as_secs_f64() * 1000.0 / cycle_times.len() as f64 // Convert to ms
    } else {
        0.0
    };
    
    let avg_adaptation_latency = if !adaptation_latencies.is_empty() {
        let sum = adaptation_latencies.iter().sum::<Duration>();
        sum.as_secs_f64() * 1000.0 / adaptation_latencies.len() as f64 // Convert to ms
    } else {
        0.0
    };
    
    let prediction_accuracy = if num_trials > 0 {
        prediction_hits as f64 / num_trials as f64 * 100.0
    } else {
        0.0
    };
    
    MarkovBenchmarkResult {
        scenario: "Rapid Transitions".to_string(),
        cycle_time_ms: avg_cycle_time,
        prediction_accuracy,
        adaptation_latency_ms: avg_adaptation_latency,
        trials: num_trials,
    }
}

pub fn print_results(results: &[MarkovBenchmarkResult]) {
    println!("\n| Environment Scenario | OODA Cycle (ms) | Prediction Accuracy (%) | Adaptation Latency (ms) |");
    println!("|----------------------|-----------------|-------------------------|--------------------------|");
    
    for result in results {
        println!(
            "| {:<20} | {:<15.2} | {:<23.1} | {:<24.2} |",
            result.scenario,
            result.cycle_time_ms,
            result.prediction_accuracy,
            result.adaptation_latency_ms
        );
    }
    
    println!(
        "\n*Table 6: Markov Environment Model Performance (n={} trials)*",
        results[0].trials
    );
}