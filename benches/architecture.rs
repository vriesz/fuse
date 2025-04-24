use criterion::{criterion_group, criterion_main, Criterion};
use uav_arch_gen::engine::generate_architecture;
use uav_arch_gen::models::UavConstraints;

fn bench_arch_gen(c: &mut Criterion) {
    let constraints = UavConstraints::default();
    
    c.bench_function("architecture_generation", |b| {
        b.iter(|| generate_architecture(&constraints))
    });
}

criterion_group!{
    name = benches;
    config = Criterion::default();
    targets = bench_arch_gen
}

criterion_main!(benches);