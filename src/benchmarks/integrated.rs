// src/benchmarks/integrated.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use crate::engine::generate_architecture;
use crate::models::UavConstraints;
use crate::flight_control::PIDController;
use crate::comms::{LinkType, CommunicationHub, CommsPriority};
use crate::comms::dds::DDSQoSProfile;
use crate::comms::tta::TTACycle;
use crate::comms::fog::FogComputingManager;
use crate::models::architecture::UavSystems;
use crate::models::constraints::MissionType;
use crate::ooda::OodaLoop;
use std::time::{Duration, Instant};

// Architecture generation benchmark
pub fn arch_gen_benchmark(c: &mut Criterion) {
    let constraints = UavConstraints::default();
    
    c.benchmark_group("Architecture Generation")
        .bench_function("generate_architecture", |b| {
            b.iter(|| generate_architecture(&constraints))
        });
}

// PID controller benchmark
pub fn pid_controller_benchmark(c: &mut Criterion) {
    let mut pid = PIDController::new(0.1, 0.01, 0.05);
    
    c.benchmark_group("PID Controller")
        .bench_function("update", |b| {
            b.iter(|| {
                pid.update(black_box(10.0), black_box(0.1));
            })
        });
}

// Integration with existing benchmark functionality
pub fn integrated_benchmarks() {
    // Run original communication benchmarks
    let mut criterion = Criterion::default();
    arch_gen_benchmark(&mut criterion);
    pid_controller_benchmark(&mut criterion);
    
    // Run the existing benchmarks
    super::communication::communication_benchmarks(&mut criterion);
    let mission_results = super::mission::run_benchmarks(100);
    super::mission::print_results(&mission_results);
    
    let (weather_results, terrain_results) = super::environmental::run_benchmarks(50, 50);
    super::environmental::print_weather_results(&weather_results);
    super::environmental::print_terrain_results(&terrain_results);
}

// For standalone execution
#[cfg(feature = "bench")]
pub fn run_benchmarks() {
    integrated_benchmarks();
}