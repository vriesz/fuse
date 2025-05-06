// src/benchmarks/integrated.rs

// Integration with existing benchmark functionality
pub fn integrated_benchmarks() {
    // Run the existing benchmarks
    super::communication::run_architectural_comparison(50);
    let mission_results = super::mission::run_benchmarks(100);
    super::mission::print_results(&mission_results);
    
    let (weather_results, terrain_results) = super::environmental::run_benchmarks(50, 50);
    super::environmental::print_weather_results(&weather_results);
    super::environmental::print_terrain_results(&terrain_results);
    
    // Run physical benchmarks as well
    let physical_results = super::physical_benchmarks::run_physical_benchmarks(30);
    super::physical_benchmarks::print_results(&physical_results);
}

// For standalone execution
#[cfg(feature = "bench")]
pub fn run_benchmarks() {
    integrated_benchmarks();
}