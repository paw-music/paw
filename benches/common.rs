use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench(c: &mut Criterion) {
    // c.bench_function("WavetableProps::lerp", |b| {
    //     b.iter(|| props.lerp(black_box(0.1276438512323)))
    // });
}

criterion_group! {
    name = common;
    config = Criterion::default().sample_size(1_000);
    targets = bench
}
criterion_main!(common);
