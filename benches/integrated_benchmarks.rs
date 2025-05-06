// benches/integrated_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uav_arch_gen::engine::generate_architecture;
use uav_arch_gen::flight_control::PIDController;
use uav_arch_gen::models::UavConstraints;
// use uav_arch_gen::benchmarks::integrated::{
//     arch_gen_benchmark,
//     pid_controller_benchmark,
//     integrated_benchmarks
// };

use uav_arch_gen::benchmarks;

pub fn bench_arch_gen(c: &mut Criterion) {
    let constraints = UavConstraints::default();

    c.benchmark_group("Architecture Generation")
        .bench_function("generate_architecture", |b| {
            b.iter(|| generate_architecture(&constraints))
        });
}

pub fn bench_pid_controller(c: &mut Criterion) {
    let mut pid = PIDController::new(0.1, 0.01, 0.05);

    c.benchmark_group("PID Controller")
        .bench_function("update", |b| {
            b.iter(|| {
                pid.update(black_box(10.0), black_box(0.1));
            })
        });
}

pub fn bench_uav_systems(_c: &mut Criterion) {
    // Run the custom benchmarks from the src/benchmarks directory
    // This will print formatted tables with the results
    benchmarks::run_all_benchmarks();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(50);
    targets = bench_arch_gen, bench_pid_controller, bench_uav_systems
);
criterion_main!(benches);
