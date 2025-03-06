use crate::{
    midi::event::MidiEventListener,
    modulation::{env::EnvProps, fm, lfo::LfoProps, mod_pack::ModPack, ModValue},
    osc::{
        clock::{Clock, Freq},
        Osc, OscPack, OscParams,
    },
    param::f32::{SignedUnitInterval, UnitInterval},
    sample::Frame,
};

pub mod controller;

// TODO: Non-static osc props
pub struct VoiceParams<'a, O: Osc, const OSCS: usize> {
    pub osc_params: [OscParams<'static, O, OSCS>; OSCS],
    pub env_params: &'a [EnvProps],
    pub lfo_params: &'a [LfoProps],
    pub amp_mod: ModValue,
}

// FIXME: Env changes how FM sounds with two oscs

pub struct Voice<O: Osc, const LFOS: usize, const ENVS: usize, const OSCS: usize> {
    oscs: OscPack<O, OSCS>,
    root_freq: Freq,
    detune: SignedUnitInterval,
    blend: UnitInterval,
    stereo_balance: UnitInterval,
    mods: ModPack<LFOS, ENVS, OSCS>,
    velocity: UnitInterval,
}

impl<O: Osc + 'static, const LFOS: usize, const ENVS: usize, const OSCS: usize> MidiEventListener
    for Voice<O, LFOS, ENVS, OSCS>
{
    fn note_on(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        self.root_freq = note.freq();
        self.velocity = velocity;

        self.mods.note_on(clock, note, velocity);
        self.oscs.note_on(clock, note, velocity);
    }

    fn note_off(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        self.velocity = UnitInterval::MIN;
        self.mods.note_off(clock, note, velocity);
        self.oscs.note_off(clock, note, velocity);
    }
}

impl<O: Osc + 'static, const LFOS: usize, const ENVS: usize, const OSCS: usize>
    Voice<O, LFOS, ENVS, OSCS>
{
    pub fn new(osc: impl Fn(usize) -> O) -> Self {
        Self {
            oscs: OscPack::new(osc),
            root_freq: Freq::ZERO,
            detune: SignedUnitInterval::EQUILIBRIUM,
            blend: UnitInterval::MAX,
            stereo_balance: UnitInterval::EQUILIBRIUM,
            mods: ModPack::new(),
            velocity: UnitInterval::MIN,
        }
    }

    pub fn set_detune(&mut self, blend: UnitInterval, detune: SignedUnitInterval) {
        self.blend = blend;
        self.detune = detune;
    }

    pub fn set_stereo_balance(&mut self, stereo_balance: UnitInterval) {
        self.stereo_balance = stereo_balance;
    }

    pub fn tick<'a>(&mut self, clock: &Clock, params: &VoiceParams<'a, O, OSCS>) -> Option<Frame> {
        let freq = fm(self.root_freq, self.detune.inner());

        // TODO: Should blend and velocity be passed to osc or is it a post-modulation?

        let amp = self.blend
            * match params.amp_mod {
                // Use raw velocity without modulation
                ModValue::None => self.velocity,
                // Envelope depends on velocity (attack goes to velocity), so env is just setting the amp.
                ModValue::Env(env) => env,
                // TODO: use `am` instead of mul?
                // Lfo modulates amp with max of given velocity
                ModValue::Lfo(lfo) => lfo.remap_into_ui() * self.velocity,
            };

        let sample = self
            .oscs
            .tick(clock, freq, &params.osc_params)
            .map(|sample| sample * amp.inner());

        sample.map(|sample| Frame::equal(sample).stereo_balanced(self.stereo_balance))
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
