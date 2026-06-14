use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_inference(c: &mut Criterion) {
    use ai_runtime::inference::InferenceEngine;
    use ai_runtime::telemetry::TelemetrySnapshot;

    let engine = InferenceEngine::new().unwrap();
    let snap = TelemetrySnapshot {
        timestamp_ms: 0,
        cpu_avg: 0.5,
        cpu_per_core: vec![0.5; 8],
        memory_used_bytes: 1_000_000,
        memory_total_bytes: 8_000_000,
        process_count: 100,
    };

    c.bench_function("inference_latency", |b| {
        b.iter(|| engine.suggest_priorities(black_box(&snap)).unwrap())
    });
}

criterion_group!(benches, bench_inference);
criterion_main!(benches);
