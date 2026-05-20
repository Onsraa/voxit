#[path = "common.rs"]
mod common;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use voxit::volume::build_from_geotiff;

fn bench_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_from_geotiff");
    let raw_side = 128_u32;
    let amp = 2500.0_f32;
    let (raw, thresholds) = common::raw_for(raw_side, amp);
    for &(density, label) in &[(30.0_f32, "d30m"), (10.0, "d10m"), (5.0, "d5m")] {
        let probe = build_from_geotiff(&raw, density, 1.0, &thresholds);
        let total = probe.dims[0] * probe.dims[1] * probe.dims[2];
        group.throughput(criterion::Throughput::Elements(total as u64));
        group.bench_with_input(BenchmarkId::from_parameter(label), &(), |b, _| {
            b.iter(|| build_from_geotiff(&raw, density, 1.0, &thresholds));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_build);
criterion_main!(benches);
