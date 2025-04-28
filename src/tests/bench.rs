// src/tests/bench.rs

#[cfg(feature = "bench")]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use crate::engine::generate_architecture;
use crate::models::constraints::UavConstraints;
use crate::ooda::OodaLoop;
use crate::models::architecture::UavSystems;
use crate::models::constraints::MissionType;
use crate::benchmarks;


#[cfg(feature = "bench")]
pub fn bench_architecture_generation(c: &mut Criterion) {
    let constraints = UavConstraints::default();
    
    c.bench_function("generate_default_architecture", |b| {
        b.iter(|| generate_architecture(black_box(&constraints)))
    });
    
    let mut ai_constraints = constraints.clone();
    ai_constraints.requires_ai = true;
    ai_constraints.secure_comms = true;
    
    c.bench_function("generate_complex_architecture", |b| {
        b.iter(|| generate_architecture(black_box(&ai_constraints)))
    });
}

#[cfg(feature = "bench")]
pub fn bench_ooda_cycle(c: &mut Criterion) {
    let mut uav = UavSystems::new(MissionType::Surveillance);
    let mut ooda = OodaLoop::new();
    
    // Add some radar contacts
    uav.scan_surroundings();
    
    c.bench_function("ooda_cycle_execution", |b| {
        b.iter(|| {
            black_box(ooda.execute_cycle(
                black_box(&mut uav.comms),
                black_box(&mut uav.payload),
                black_box(&mut uav.flight_controller)
            ))
        })
    });
}

#[cfg(feature = "bench")]
pub fn bench_performance_tables(c: &mut Criterion) {
    c.bench_function("generate_performance_tables", |b| {
        b.iter(|| {
            benchmarks::run_all_benchmarks();
        })
    });
}


#[cfg(feature = "bench")]
criterion_group!(benches, bench_architecture_generation, bench_ooda_cycle, bench_performance_tables);
#[cfg(feature = "bench")]
criterion_main!(benches);