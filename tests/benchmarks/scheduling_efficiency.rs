use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_scheduling_efficiency(c: &mut Criterion) {
    use kernel_interface::KernelInterface;

    let ki = kernel_interface::platform();

    c.bench_function("list_processes", |b| {
        b.iter(|| black_box(ki.list_processes().unwrap()))
    });
}

criterion_group!(benches, bench_scheduling_efficiency);
criterion_main!(benches);
