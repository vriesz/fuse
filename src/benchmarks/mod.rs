// src/benchmarks/mod.rs

pub mod communication;
pub mod environmental;
pub mod mission;
pub mod mocks;
pub mod physical_benchmarks;
pub mod integrated;
pub mod markov;

pub fn run_all_benchmarks() {
    println!("\n===== RUNNING BENCHMARKS FROM src/benchmarks =====");
    // integrated::integrated_benchmarks();

    println!("\n----- Mission Benchmarks -----");
    let mission_results = mission::run_benchmarks(50);
    mission::print_results(&mission_results);

    println!("\n----- Environmental Benchmarks -----");
    let (weather_results, terrain_results) = environmental::run_benchmarks(30, 30);
    environmental::print_weather_results(&weather_results);
    environmental::print_terrain_results(&terrain_results);

    println!("\n----- Communication Benchmarks -----");
    let comm_results = communication::run_architectural_comparison(50);
    communication::print_results(&comm_results);

    println!("\n----- Physical Layout Benchmarks -----");
    let physical_results = physical_benchmarks::run_physical_benchmarks(30);
    physical_benchmarks::print_results(&physical_results);

    println!("\n----- Markov Environment Model Benchmarks -----");
    let markov_results = markov::run_markov_benchmarks(50);
    markov::print_results(&markov_results);

    println!("\n===== BENCHMARKS COMPLETE =====");
}
