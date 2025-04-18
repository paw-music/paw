use crate::{
    midi::event::MidiEventListener,
    modx::{env::EnvProps, fm, lfo::LfoProps, mod_pack::ModPack, ModValue},
    osc::{
        clock::{Clock, Freq},
        OpParams, OperatorPack, Osc,
    },
    param::f32::{SignedUnitInterval, UnitInterval},
    sample::Frame,
};

pub mod controller;

// TODO: Non-static osc props
pub struct VoiceParams<'a, const OSCS: usize> {
    pub env_params: &'a [EnvProps],
    pub lfo_params: &'a [LfoProps],
    pub amp_mod: Option<ModValue>,
}

// FIXME: Env changes how FM sounds with two oscs

#[derive(Clone)]
pub struct Voice<O: Osc, const LFOS: usize, const ENVS: usize, const OSCS: usize> {
    ops: OperatorPack<O, OSCS>,
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
    #[inline]
    fn note_on(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        self.root_freq = note.freq();
        self.velocity = velocity;

        self.mods.note_on(clock, note, velocity);
        self.ops.note_on(clock, note, velocity);
    }

    #[inline]
    fn note_off(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        self.velocity = UnitInterval::MIN;
        self.mods.note_off(clock, note, velocity);
        self.ops.note_off(clock, note, velocity);
    }
}

impl<O: Osc + 'static, const LFOS: usize, const ENVS: usize, const OSCS: usize>
    Voice<O, LFOS, ENVS, OSCS>
{
    pub fn new(osc: impl Fn(usize) -> O) -> Self {
        Self {
            ops: OperatorPack::new(osc),
            root_freq: Freq::ZERO,
            detune: SignedUnitInterval::EQUILIBRIUM,
            blend: UnitInterval::MAX,
            stereo_balance: UnitInterval::EQUILIBRIUM,
            mods: ModPack::new(),
            velocity: UnitInterval::MIN,
        }
    }

    #[inline]
    pub fn set_detune(&mut self, blend: UnitInterval, detune: SignedUnitInterval) {
        self.blend = blend;
        self.detune = detune;
    }

    #[inline]
    pub fn set_stereo_balance(&mut self, stereo_balance: UnitInterval) {
        self.stereo_balance = stereo_balance;
    }

    #[inline(always)]
    pub fn tick<'a>(
        &mut self,
        clock: &Clock,
        params: &VoiceParams<'a, OSCS>,
        op_params: &[OpParams<'static, O, OSCS>; OSCS],
    ) -> Frame {
        let freq = fm(self.root_freq, self.detune.inner());

        let amp = self.blend
            * params
                .amp_mod
                .map(|amp_mod| {
                    match amp_mod {
                        // // Use raw velocity without modulation
                        // ModValue::None => self.velocity,
                        // Envelope depends on velocity (attack goes to velocity), so env is just setting the amp.
                        ModValue::Env(env) => env,
                        // TODO: use `am` instead of mul?
                        // Lfo modulates amp with max of given velocity
                        ModValue::Lfo(lfo) => lfo.remap_into_ui() * self.velocity,
                    }
                })
                .unwrap_or(UnitInterval::MAX);

        let sample = self.ops.tick(clock, freq, op_params) * amp.inner();

        Frame::mono(sample).stereo_balanced(self.stereo_balance)
    }
}
