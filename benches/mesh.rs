use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use voxit::source::mesh::MeshData;
use voxit::volume::{build_from_mesh, MeshColorMode};

fn heightmap_mesh(side: u32) -> MeshData {
    let pitch = side + 1;
    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity((pitch * pitch) as usize);
    for z in 0..=side {
        for x in 0..=side {
            let fx = x as f32 / side as f32;
            let fz = z as f32 / side as f32;
            let y = ((fx * 8.0).sin() * (fz * 6.0).cos() * 0.3 + 0.3).max(0.0);
            vertices.push([fx, y, fz]);
        }
    }

    let mut indices: Vec<u32> = Vec::with_capacity((side * side * 6) as usize);
    for z in 0..side {
        for x in 0..side {
            let i0 = z * pitch + x;
            let i1 = i0 + 1;
            let i2 = (z + 1) * pitch + x;
            let i3 = i2 + 1;
            indices.push(i0);
            indices.push(i2);
            indices.push(i1);
            indices.push(i1);
            indices.push(i2);
            indices.push(i3);
        }
    }

    let (aabb_min, aabb_max) = aabb(&vertices);

    MeshData {
        vertices,
        normals: Vec::new(),
        uvs: None,
        colors: None,
        indices,
        texture: None,
        aabb_min,
        aabb_max,
    }
}

fn aabb(verts: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut lo = [f32::INFINITY; 3];
    let mut hi = [f32::NEG_INFINITY; 3];
    for v in verts {
        for a in 0..3 {
            if v[a] < lo[a] {
                lo[a] = v[a];
            }
            if v[a] > hi[a] {
                hi[a] = v[a];
            }
        }
    }
    (lo, hi)
}

fn bench_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_from_mesh");

    for &(side, voxels_per_axis, label) in &[
        (64u32, 64u32, "8k_tris_v64"),
        (128, 128, "32k_tris_v128"),
        (256, 256, "131k_tris_v256"),
    ] {
        let mesh = heightmap_mesh(side);
        let longest = mesh
            .aabb_max
            .iter()
            .zip(mesh.aabb_min.iter())
            .map(|(h, l)| h - l)
            .fold(0.0_f32, f32::max);
        let voxel_size = longest / voxels_per_axis as f32;
        let tris = (mesh.indices.len() / 3) as u64;
        group.throughput(criterion::Throughput::Elements(tris));
        group.bench_with_input(BenchmarkId::from_parameter(label), &(), |b, _| {
            b.iter(|| build_from_mesh(&mesh, voxel_size, MeshColorMode::Auto, 0, 0));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_build);
criterion_main!(benches);
