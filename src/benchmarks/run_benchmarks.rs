// src/benchmarks/run_benchmarks.rs

use criterion::{criterion_group, criterion_main, Criterion};
use uav_arch_gen::benchmarks::integrated::{arch_gen_benchmark, pid_controller_benchmark};

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = arch_gen_benchmark, pid_controller_benchmark
);
criterion_main!(benches);