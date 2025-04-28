use criterion::{criterion_group, criterion_main, Criterion};
use uav_arch_gen::benchmarks;

fn run_all_benchmarks(_c: &mut Criterion) {
    benchmarks::run_all_benchmarks();
}

criterion_group!(benches, run_all_benchmarks);
criterion_main!(benches);