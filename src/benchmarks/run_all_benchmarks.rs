// src/benchmarks/run_all_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uav_arch_gen::engine::generate_architecture;
use uav_arch_gen::models::UavConstraints;
use uav_arch_gen::flight_control::PIDController;
use uav_arch_gen::benchmarks;

// Run architecture benchmark
fn arch_benchmark(c: &mut Criterion) {
    // Mock implementation - replace with your actual implementation
    let constraints = UavConstraints::default();
    
    c.benchmark_group("Architecture Generation")
        .bench_function("generate_architecture", |b| {
            b.iter(|| black_box(true)) // Replaced with mock for now
        });
}

// Run PID controller benchmark
fn pid_benchmark(c: &mut Criterion) {
    let mut pid = PIDController::new(0.1, 0.01, 0.05);
    
    c.benchmark_group("PID Controller")
        .bench_function("update", |b| {
            b.iter(|| {
                pid.update(black_box(10.0), black_box(0.1));
            })
        });
}

// Run the src/benchmarks code
fn additional_benchmarks(_c: &mut Criterion) {
    // Only run src/benchmarks when bench feature is enabled
    #[cfg(feature = "bench")]
    {
        println!("\n--- Running additional benchmarks from src/benchmarks/ ---");
        benchmarks::run_all_benchmarks();
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = arch_benchmark, pid_benchmark, additional_benchmarks
);
criterion_main!(benches);