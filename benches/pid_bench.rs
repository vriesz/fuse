use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uav_arch_gen::flight_control::PIDController;

fn pid_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("PID Controller");
    let mut pid = PIDController::new(0.1, 0.01, 0.05);
    
    group.bench_function("update", |b| {
        b.iter(|| {
            pid.update(black_box(10.0), black_box(0.1));
        })
    });
    
    group.finish();
}

criterion_group!(benches, pid_benchmark);
criterion_main!(benches);