use criterion::{criterion_group, criterion_main, Criterion};

fn bench_memory_overhead(c: &mut Criterion) {
    c.bench_function("memory_overhead_telemetry_collect", |b| {
        let collector = ai_runtime::telemetry::TelemetryCollector::new().unwrap();
        b.iter(|| collector.collect().unwrap())
    });
}

criterion_group!(benches, bench_memory_overhead);
criterion_main!(benches);
