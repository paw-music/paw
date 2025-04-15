use criterion::{criterion_group, criterion_main, Criterion};
use paw::{
    midi::event::MidiEventListener,
    modx::{
        env::{Env, EnvProps},
        lfo::{Lfo, LfoProps, LfoWaveform},
    },
    osc::clock::{Clock, Freq},
    param::f32::UnitInterval,
    sample::time::SampleCount,
};

const SAMPLE_RATE: u32 = 48_000;

fn bench(c: &mut Criterion) {
    {
        let mut group = c.benchmark_group("Lfo 1s ticking");

        for waveform in LfoWaveform::each(UnitInterval::EQUILIBRIUM) {
            let mut lfo = Lfo::new();

            let mut props = LfoProps::new(0);
            props.enabled = true;
            props.freq = Freq::Hz(100);
            props.waveform = waveform;
            let mut clock = Clock::zero(SAMPLE_RATE);

            // Note: Lfo does not produce anything when note isn't sent
            lfo.note_on(&clock, paw::midi::note::Note::A4, UnitInterval::MAX);

            group.bench_function(waveform.to_string(), |b| {
                b.iter(|| {
                    for _ in 0..=SAMPLE_RATE {
                        lfo.tick(&clock, core::hint::black_box(&props));
                        clock.tick();
                    }
                });
            });
        }
    }

    {
        let mut group = c.benchmark_group("Env 1s ticking");

        for (name, delay, attack, hold, decay, sustain, release) in [
            (
                "All zero",
                /* Delay */ SampleCount::zero(),
                /* Attack */ SampleCount::zero(),
                /* Hold */ SampleCount::zero(),
                /* Decay */ SampleCount::zero(),
                /* Sustain */ UnitInterval::MAX,
                /* Release */ SampleCount::zero(),
            ),
            (
                "Long Delay",
                /* Delay */ SampleCount::from_secs(1, SAMPLE_RATE),
                /* Attack */ SampleCount::zero(),
                /* Hold */ SampleCount::zero(),
                /* Decay */ SampleCount::zero(),
                /* Sustain */ UnitInterval::MAX,
                /* Release */ SampleCount::zero(),
            ),
            (
                "Long Attack",
                /* Delay */ SampleCount::zero(),
                /* Attack */ SampleCount::from_secs(1, SAMPLE_RATE),
                /* Hold */ SampleCount::zero(),
                /* Decay */ SampleCount::zero(),
                /* Sustain */ UnitInterval::MAX,
                /* Release */ SampleCount::zero(),
            ),
            (
                "Long Hold",
                /* Delay */ SampleCount::zero(),
                /* Attack */ SampleCount::zero(),
                /* Hold */ SampleCount::from_secs(1, SAMPLE_RATE),
                /* Decay */ SampleCount::zero(),
                /* Sustain */ UnitInterval::MAX,
                /* Release */ SampleCount::zero(),
            ),
            (
                "Long Decay",
                /* Delay */ SampleCount::zero(),
                /* Attack */ SampleCount::zero(),
                /* Hold */ SampleCount::zero(),
                /* Decay */ SampleCount::from_secs(1, SAMPLE_RATE),
                /* Sustain */ UnitInterval::MAX,
                /* Release */ SampleCount::zero(),
            ),
            (
                "Long Release",
                /* Delay */ SampleCount::zero(),
                /* Attack */ SampleCount::zero(),
                /* Hold */ SampleCount::zero(),
                /* Decay */ SampleCount::zero(),
                /* Sustain */ UnitInterval::MAX,
                /* Release */ SampleCount::from_secs(1, SAMPLE_RATE),
            ),
        ] {
            let props = EnvProps {
                index: 0,
                enabled: true,
                amount: UnitInterval::MAX,
                target: paw::modx::mod_pack::ModTarget::default(),
                delay,
                attack,
                hold,
                decay,
                sustain,
                release,
            };

            let mut env = Env::new();

            group.bench_function(
                format!(
                    "{name} (D={delay}, A={attack}, H={hold}, D={decay}, S={sustain}, R={release})"
                ),
                |b| {
                    b.iter(|| {
                        let mut clock = core::hint::black_box(Clock::zero(SAMPLE_RATE));

                        env.note_on(&clock, paw::midi::note::Note::A4, UnitInterval::MAX);

                        // Before sustain
                        for length in [props.delay, props.attack, props.hold, props.decay] {
                            for _ in 0..length.inner() {
                                env.tick(&clock, core::hint::black_box(&props));
                                clock.tick();
                            }
                        }

                        // Sustain for 0.1s
                        for _ in 0..SAMPLE_RATE / 10 {
                            env.tick(&clock, core::hint::black_box(&props));
                            clock.tick();
                        }

                        env.note_off(&clock, paw::midi::note::Note::A4, UnitInterval::MAX);

                        // Release 0.25s
                        for _ in 0..props.release.inner() {
                            env.tick(&clock, core::hint::black_box(&props));
                            clock.tick();
                        }
                    });
                },
            );
        }
    }
}

criterion_group! {
    name = modx;
    config = Criterion::default();
    targets = bench
}
criterion_main!(modx);
