use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use paw::{
    midi::note::Note,
    osc::clock::{Clock, Freq},
};
use std::time::Duration;

const SAMPLE_RATE: u32 = 48_000;

fn bench(c: &mut Criterion) {
    {
        let mut group = c.benchmark_group("Clock::phase vs Clock::phase_fast");

        let clock = Clock::zero(SAMPLE_RATE).with_tick(1_000);
        let mut last_sync = 0;
        let mut last_freq = Freq::ZERO;
        let mut phase_step = 0.0;
        for note in Note::each() {
            let freq = note.freq();
            group
                .bench_with_input(BenchmarkId::new("phase", freq), &freq, |b, freq| {
                    b.iter(|| {
                        clock.phase(black_box(*freq), black_box(&mut last_sync));
                    });
                })
                .bench_with_input(BenchmarkId::new("phase_fast", freq), &freq, |b, freq| {
                    let phase_step = freq.inner() / SAMPLE_RATE as f32;
                    b.iter(|| {
                        clock.phase_fast(black_box(phase_step), black_box(&mut last_sync));
                    });
                })
                .bench_with_input(
                    BenchmarkId::new("phase_fast (include step recalculation)", freq),
                    &freq,
                    |b, &freq| {
                        b.iter(|| {
                            if last_freq != freq {
                                last_freq = freq;
                                phase_step = freq.inner() / clock.sample_rate as f32;
                            }
                            clock.phase_fast(black_box(phase_step), black_box(&mut last_sync));
                        });
                    },
                );
        }
    }
}

criterion_group! {
    name = common;
    config = Criterion::default().measurement_time(Duration::from_secs(5));
    targets = bench
}
criterion_main!(common);
