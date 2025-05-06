// benches/benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uav_arch_gen::engine::generate_architecture;
use uav_arch_gen::models::UavConstraints;
use uav_arch_gen::flight_control::PIDController;

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

// Run the benchmarks
criterion_group!(benches, arch_gen_benchmark, pid_controller_benchmark);
criterion_main!(benches);