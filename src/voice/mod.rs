use crate::{
    components::{
        env::{EnvPack, EnvParams, EnvTarget},
        lfo::{LfoPack, LfoParams, LfoTarget},
    },
    midi::event::MidiEventListener,
    osc::{clock::Clock, Osc, OscPack, OscProps},
    param::f32::{SignedUnitInterval, UnitInterval},
    value::freq::modulate_freq,
};

pub mod controller;

#[derive(Clone, Copy)]
pub struct VoiceParams<'a, O: Osc, const OSCS: usize> {
    pub osc_props: &'a [OscProps<'static, O>],
    pub env_params: &'a [EnvParams<OSCS>],
    pub lfo_params: &'a [LfoParams<OSCS>],
}

pub struct Voice<O: Osc, const LFOS: usize, const ENVS: usize, const OSCS: usize> {
    oscs: OscPack<O, OSCS>,
    root_freq: f32,
    detune: SignedUnitInterval,
    blend: UnitInterval,
    envs: EnvPack<ENVS, OSCS>,
    lfos: LfoPack<LFOS, OSCS>,
}

impl<O: Osc + 'static, const LFOS: usize, const ENVS: usize, const OSCS: usize> MidiEventListener
    for Voice<O, LFOS, ENVS, OSCS>
{
    fn note_on(&mut self, note: crate::midi::note::Note, velocity: UnitInterval) {
        self.root_freq = note.freq();

        // self.oscs.note_on(self.root_freq);
        self.envs.note_on(note, velocity);
        self.lfos.note_on(note, velocity);
    }

    fn note_off(&mut self, note: crate::midi::note::Note, velocity: UnitInterval) {
        self.envs.note_off(note, velocity);
        self.lfos.note_off(note, velocity);
    }
}

impl<O: Osc + 'static, const LFOS: usize, const ENVS: usize, const OSCS: usize>
    Voice<O, LFOS, ENVS, OSCS>
{
    pub fn new(osc: impl Fn(usize) -> O) -> Self {
        Self {
            oscs: OscPack::new(osc),
            root_freq: 0.0,
            detune: SignedUnitInterval::EQUILIBRIUM,
            blend: UnitInterval::MAX,
            envs: EnvPack::new(),
            lfos: LfoPack::new(),
        }
    }

    pub fn set_detune(&mut self, blend: UnitInterval, detune: SignedUnitInterval) {
        self.blend = blend;
        self.detune = detune;
    }

    pub fn tick<'a>(
        &mut self,
        clock: &Clock,
        params: &VoiceParams<'a, O, OSCS>,
    ) -> Option<SignedUnitInterval> {
        let pitch_mod = self
            .envs
            .tick(EnvTarget::SynthPitch, &params.env_params)
            .map(|pitch_mod| pitch_mod.remap_into_signed())
            .or_else(|| {
                self.lfos
                    .tick(clock, LfoTarget::GlobalPitch, params.lfo_params)
            })
            .unwrap_or(SignedUnitInterval::EQUILIBRIUM);

        // FIXME: Should be clamped by SignedUnitInterval or can exceed [-1.0; 1.0] range?
        let pitch_mod = pitch_mod.inner() + self.detune.inner();

        let freq = modulate_freq(self.root_freq, pitch_mod);

        let amp_mod = self
            .envs
            .tick(EnvTarget::SynthLevel, &params.env_params)
            .or_else(|| {
                self.lfos
                    .tick(clock, LfoTarget::GlobalLevel, params.lfo_params)
                    .map(|amp_mod| amp_mod.remap_into_unsigned())
            });

        self.oscs.tick(clock, freq, params.osc_props).map(|sample| {
            let sample = if let Some(amp_mod) = amp_mod {
                sample * amp_mod
            } else {
                sample
            };

            sample * self.blend
        })
    }
}

// impl<O: Osc, const SIZE: usize, const SAMPLE_RATE: u32> Iterator
//     for VoicesController<O, SIZE, SAMPLE_RATE>
// where
//     O::Item: Sample,
// {
//     type Item = O::Item;

//     fn next(&mut self) -> Option<Self::Item> {
//         // Note: Check if this logic with Some(....sum()) is right. Maybe if all voices are off it should return None

//         Some(
//         )
//     }
// }

// impl<O: Osc, const SIZE: usize, const SAMPLE_RATE: u32> Source
//     for VoicesController<O, SIZE, SAMPLE_RATE>
// where
//     <O as Iterator>::Item: Sample,
// {
// }
