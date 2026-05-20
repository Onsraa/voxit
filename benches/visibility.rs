#[path = "common.rs"]
mod common;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use voxit::visibility::VisibilityMask;

fn bench_compute(c: &mut Criterion) {
    let mut group = c.benchmark_group("VisibilityMask::recompute");
    for &(raw_side, density, amp, label) in &[
        (128_u32, 30.0_f32, 2500.0_f32, "132_cubed"),
        (256, 30.0, 2500.0, "264_cubed"),
        (512, 30.0, 15800.0, "528_cubed"),
    ] {
        let (grid, settings, _mask) = common::fixture(raw_side, density, amp);
        let total = grid.dims[0] * grid.dims[1] * grid.dims[2];
        group.throughput(criterion::Throughput::Elements(total as u64));
        // Warm the storage once so we measure the steady-state in-place
        // recompute, matching how live slider drags hit it.
        let mut mask = VisibilityMask::new_empty();
        mask.recompute(&grid, &settings);
        group.bench_with_input(BenchmarkId::from_parameter(label), &(), |b, _| {
            b.iter(|| mask.recompute(&grid, &settings));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_compute);
criterion_main!(benches);
