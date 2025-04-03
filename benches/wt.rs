use std::f32::consts::TAU;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use paw::{
    midi::event::MidiEventListener,
    osc::clock::Clock,
    param::f32::UnitInterval,
    wavetable::{synth::create_basic_wavetable_synth, Wavetable, WavetableProps},
};

const SAMPLE_RATE: u32 = 48_000;

fn wavetable_synth_1s<
    const VOICES: usize,
    const LFOS: usize,
    const ENVS: usize,
    const OSCS: usize,
>() {
    let mut synth = create_basic_wavetable_synth::<VOICES, LFOS, ENVS, OSCS>(SAMPLE_RATE);

    let mut clock = Clock::zero(SAMPLE_RATE);

    synth.note_on(&clock, paw::midi::note::Note::A4, UnitInterval::MAX);

    for _ in 0..SAMPLE_RATE {
        synth.tick(&clock);
        clock.tick();
    }
}

fn bench(c: &mut Criterion) {
    const WT_LENGTH: usize = 1024;

    let wt = Wavetable::<1, WT_LENGTH>::new(|_, phase| (TAU * phase).sin());

    let props = WavetableProps::new(0, &wt);

    c.bench_function("WavetableProps::lerp", |b| {
        b.iter(|| props.lerp(black_box(0.1276438512323)))
    });

    c.bench_function("wavetable_synth_1s<VOICES=1,LFOS=0,ENVS=0,OSCS=0>", |b| {
        b.iter(wavetable_synth_1s::<1, 0, 0, 0>);
    });

    c.bench_function("wavetable_synth_1s<VOICES=1,LFOS=1,ENVS=1,OSCS=1>", |b| {
        b.iter(wavetable_synth_1s::<1, 1, 1, 1>);
    });

    c.bench_function("wavetable_synth_1s<VOICES=8,LFOS=1,ENVS=1,OSCS=1>", |b| {
        b.iter(wavetable_synth_1s::<8, 1, 1, 1>);
    });

    c.bench_function("wavetable_synth_1s<VOICES=16,LFOS=1,ENVS=1,OSCS=1>", |b| {
        b.iter(wavetable_synth_1s::<16, 1, 1, 1>);
    });

    c.bench_function("wavetable_synth_1s<VOICES=1,LFOS=8,ENVS=1,OSCS=1>", |b| {
        b.iter(wavetable_synth_1s::<1, 8, 1, 1>);
    });

    c.bench_function("wavetable_synth_1s<VOICES=1,LFOS=16,ENVS=1,OSCS=1>", |b| {
        b.iter(wavetable_synth_1s::<1, 16, 1, 1>);
    });

    c.bench_function("wavetable_synth_1s<VOICES=1,LFOS=1,ENVS=8,OSCS=1>", |b| {
        b.iter(wavetable_synth_1s::<1, 1, 8, 1>);
    });

    c.bench_function("wavetable_synth_1s<VOICES=1,LFOS=1,ENVS=16,OSCS=1>", |b| {
        b.iter(wavetable_synth_1s::<1, 1, 16, 1>);
    });

    c.bench_function("wavetable_synth_1s<VOICES=1,LFOS=1,ENVS=1,OSCS=8>", |b| {
        b.iter(wavetable_synth_1s::<1, 1, 1, 8>);
    });

    c.bench_function("wavetable_synth_1s<VOICES=1,LFOS=1,ENVS=1,OSCS=16>", |b| {
        b.iter(wavetable_synth_1s::<1, 1, 1, 16>);
    });
}

criterion_group! {
    name = wt;
    config = Criterion::default().sample_size(1_000);
    targets = bench
}
criterion_main!(wt);
