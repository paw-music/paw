use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use paw::{
    daw::{channel_rack::Instrument, Daw},
    midi::event::MidiEventListener,
    osc::clock::Clock,
    param::f32::UnitInterval,
    sample::Frame,
    wavetable::{
        synth::{create_basic_wavetable_synth, WavetableSynth},
        Wavetable, WavetableProps,
    },
};
use pprof::criterion::PProfProfiler;
use std::f32::consts::TAU;

const SAMPLE_RATE: u32 = 48_000;

fn create_playing_wt_synth<
    const VOICES: usize,
    const LFOS: usize,
    const ENVS: usize,
    const OSCS: usize,
>(
    clock: &Clock,
) -> WavetableSynth<4, 1024, VOICES, LFOS, ENVS, OSCS> {
    let mut synth = create_basic_wavetable_synth::<VOICES, LFOS, ENVS, OSCS>(SAMPLE_RATE);

    synth.note_on(&clock, paw::midi::note::Note::A4, UnitInterval::MAX);

    synth
}

macro_rules! wavetable_synth_1s_bench {
    ($group: ident: [
        $($voices: expr, $lfos: expr, $envs: expr, $oscs: expr),*
        $(,)?
    ]) => {
        $(
            let preset = ($voices, $lfos, $envs, $oscs);
            $group.bench_with_input(format!("Sample-by-sample {preset:?}"), &preset, |b, _| {
                let mut clock = Clock::zero(SAMPLE_RATE);
                let mut synth = create_playing_wt_synth::<$voices, $lfos, $envs, $oscs>(&clock);

                b.iter(|| {
                    for _ in 0..SAMPLE_RATE as usize {
                        synth.tick(&clock);
                        clock.tick();
                    }
                });
            });

            $group.bench_with_input(format!("Buffer processing {preset:?}"), &preset, |b, _| {
                let clock = Clock::zero(SAMPLE_RATE);
                let mut synth = create_playing_wt_synth::<$voices, $lfos, $envs, $oscs>(&clock);
                let mut buffer = [Frame::zero(); SAMPLE_RATE as usize];

                b.iter(|| {
                    synth.process_buffer(&clock, &mut buffer);
                });
            });
        )*
    };
}

fn bench(c: &mut Criterion) {
    const WT_LENGTH: usize = 1024;

    let wt = Wavetable::<1, WT_LENGTH>::new(|_, phase| (TAU * phase).sin());

    let props = WavetableProps::new(0, &wt);

    {
        let mut group = c.benchmark_group("WavetableProps");
        let group = group.sample_size(10_000);

        group.bench_function("lerp", |b| {
            b.iter(|| props.lerp(black_box(0.1276438512323)))
        });
    }

    {
        let mut group = c.benchmark_group("1s WT synth playback");
        let group = group.sample_size(10_000);

        wavetable_synth_1s_bench!(
            group: [
                1, 0, 0, 0,
                1, 1, 1, 1,
                8, 1, 1, 1,
                16, 1, 1, 1,
                1, 8, 1, 1,
                1, 16, 1, 1,
                1, 1, 8, 1,
                1, 1, 16, 1,
                1, 1, 1, 8,
                1, 1, 1, 16,
            ]
        );
    }

    {
        let mut group = c.benchmark_group("Synth vs DAW overhead");

        const VOICES: usize = 16;
        const LFOS: usize = 4;
        const ENVS: usize = 4;
        const OSCS: usize = 4;
        const BUFFER_SIZE: usize = 4096;

        let mut clock = Clock::zero(SAMPLE_RATE);
        let mut synth = create_playing_wt_synth::<VOICES, LFOS, ENVS, OSCS>(&clock);

        group
            .bench_function("Wavetable Synth direct usage", |b| {
                b.iter(|| {
                    for _ in 0..BUFFER_SIZE {
                        synth.tick(&clock);
                        clock.tick();
                    }
                });
            })
            .bench_function("Wavetable Synth direct usage (buffer)", |b| {
                let mut buffer = [Frame::zero(); BUFFER_SIZE];
                b.iter(|| synth.process_buffer(&clock, &mut buffer));
            });

        let mut daw = Daw::<1, 1, 0>::new(SAMPLE_RATE);
        daw.rack_mut()
            .push_instrument(Box::new(create_basic_wavetable_synth::<
                VOICES,
                LFOS,
                ENVS,
                OSCS,
            >(SAMPLE_RATE)))
            .unwrap();

        daw.note_on(paw::midi::note::Note::A0, UnitInterval::MAX);

        group
            .bench_function("DAW abstraction", |b| {
                b.iter(|| {
                    for _ in 0..BUFFER_SIZE {
                        daw.tick_internal();
                    }
                });
            })
            .bench_function("DAW abstraction (buffer)", |b| {
                let mut buffer = [Frame::zero(); BUFFER_SIZE];
                b.iter(|| {
                    daw.process_buffer(&mut buffer);
                });
            });
    }

    {
        let mut group = c.benchmark_group("Buffer processing vs sample-by-sample");

        const VOICES: usize = 16;
        const LFOS: usize = 4;
        const ENVS: usize = 4;
        const OSCS: usize = 4;

        let mut daw = Daw::<1, 1, 0>::new(SAMPLE_RATE);

        daw.rack_mut()
            .push_instrument(Box::new(create_basic_wavetable_synth::<
                VOICES,
                LFOS,
                ENVS,
                OSCS,
            >(SAMPLE_RATE)))
            .unwrap();

        daw.note_on(paw::midi::note::Note::A0, UnitInterval::MAX);

        for p in 7..=12 {
            let size = 2usize.pow(p);

            group
                .throughput(criterion::Throughput::Elements(size as u64))
                .bench_with_input(
                    BenchmarkId::new("Buffer processing", size),
                    &size,
                    |b, &size| {
                        let mut buffer = vec![Frame::zero(); size];
                        b.iter(|| {
                            daw.process_buffer(&mut buffer);
                        });
                    },
                )
                .bench_with_input(
                    BenchmarkId::new("Sample-by-sample", size),
                    &size,
                    |b, &size| {
                        b.iter(|| {
                            for _ in 0..size {
                                daw.tick_internal();
                            }
                        });
                    },
                );
        }
    }
}

criterion_group! {
    name = wt;
    config = Criterion::default()
    .with_profiler(PProfProfiler::new(50_000, pprof::criterion::Output::Flamegraph(None)));
    targets = bench
}
criterion_main!(wt);
