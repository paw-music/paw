use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::{
    f32::consts::{PI, TAU},
    time::Duration,
};

macro_rules! bench_fns {
    ($group: ident ($input: expr): {
        $($f: ident),* $(,)?
    }) => {
        $($group.bench_with_input(
            BenchmarkId::new(format!("std::f32::{}", stringify!($f)), $input),
            &$input,
            |b, &input| {
                b.iter(|| f32::$f(input));
            },
        );

        $group.bench_with_input(
            BenchmarkId::new(format!("micromath::F32Ext::{}", stringify!($f)), $input),
            &$input,
            |b, &input| {
                b.iter(|| micromath::F32Ext::$f(input));
            },
        );)*
    };
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Native FPU (this CPU) vs micromath");

    // 0.0f32.abs();

    for input in [
        0.0,
        1.0,
        -1.0,
        0.5,
        PI,
        TAU,
        -PI,
        -TAU,
        0.4376487234832,
        -1.3475648723,
    ] {
        // for (name, f) in [
        //     ("FPU asin", f32::asin as fn(f32) -> f32),
        //     ("micromath asin", F32Ext::asin),
        //     ("FPU acos", f32::acos),
        //     ("micromath acos", F32Ext::acos),
        //     ("FPU sin", f32::sin),
        //     ("micromath sin", F32Ext::sin),
        //     ("FPU cos", f32::cos),
        //     ("micromath cos", F32Ext::cos),
        //     ("FPU cos", f32::cos),
        //     ("micromath cos", F32Ext::cos),
        // ] {
        //     group.bench_with_input(BenchmarkId::new(name, input), &input, |b, &input| {
        //         b.iter(|| f(input));
        //     });
        // }

        bench_fns!(group (input): {
            abs,
            asin,
            acos,
            atan,
            ceil,
            cos,
            exp,
            floor,
            fract,
            ln,
            log2,
            log10,
            recip,
            round,
            sin,
            sin_cos,
            sqrt,
            tan,
            trunc,
        });
    }

    group.finish();
}

criterion_group! {
    name = fpu_vs_micromath;
    config = Criterion::default().measurement_time(Duration::from_secs(1)).sample_size(100);
    // .with_profiler(PProfProfiler::new(50_000, pprof::criterion::Output::Flamegraph(None)));
    targets = bench
}
criterion_main!(fpu_vs_micromath);
