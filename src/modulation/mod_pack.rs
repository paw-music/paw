use core::fmt::Display;

use super::{
    env::{EnvPack, EnvProps},
    lfo::{LfoPack, LfoProps},
    ModValue,
};
use crate::{midi::event::MidiEventListener, osc::clock::Clock};

// #[derive(Debug, Clone, Copy)]
// pub enum ModSource {
//     Lfo(usize),
//     Env(usize),
// }

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ModTarget {
    // Global modulations //
    #[default]
    GlobalLevel,
    GlobalPitch,

    // Wavetable modulations //
    OscWtPos(usize),
}

impl Display for ModTarget {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ModTarget::GlobalLevel => write!(f, "Synth level"),
            ModTarget::GlobalPitch => write!(f, "Synth pitch"),
            // ModTarget::OscPitch(osc) => write!(f, "OSC{osc} pitch"),
            // ModTarget::OscLevel(osc) => write!(f, "OSC{osc} level"),
            ModTarget::OscWtPos(osc) => write!(f, "OSC{osc} WT position"),
        }
    }
}

impl ModTarget {
    #[inline]
    pub fn each<const OSCS: usize>() -> impl Iterator<Item = Self> {
        [Self::GlobalLevel, Self::GlobalPitch]
            .into_iter()
            // .chain((0..OSCS).map(|osc| Self::OscPitch(osc)))
            // .chain((0..OSCS).map(|osc| Self::OscLevel(osc)))
            .chain((0..OSCS).map(|osc| Self::OscWtPos(osc)))
    }
}

#[derive(Clone)]
pub struct ModPack<const LFOS: usize, const ENVS: usize, const OSCS: usize> {
    lfos: LfoPack<LFOS>,
    envs: EnvPack<ENVS>,
}

impl<const LFOS: usize, const ENVS: usize, const OSCS: usize> MidiEventListener
    for ModPack<LFOS, ENVS, OSCS>
{
    #[inline]
    fn note_on(
        &mut self,
        clock: &crate::osc::clock::Clock,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.lfos.note_on(clock, note, velocity);
        self.envs.note_on(clock, note, velocity);
    }

    #[inline]
    fn note_off(
        &mut self,
        clock: &crate::osc::clock::Clock,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.lfos.note_off(clock, note, velocity);
        self.envs.note_off(clock, note, velocity);
    }
}

impl<const LFOS: usize, const ENVS: usize, const OSCS: usize> ModPack<LFOS, ENVS, OSCS> {
    pub fn new() -> Self {
        Self {
            lfos: LfoPack::new(),
            envs: EnvPack::new(),
        }
    }

    #[inline]
    pub fn tick(
        &mut self,
        clock: &Clock,
        target: ModTarget,
        lfo_props: &[LfoProps],
        env_props: &[EnvProps],
    ) -> ModValue {
        self.lfos
            .tick(clock, target, lfo_props)
            .map(|lfo_mod| ModValue::Lfo(lfo_mod))
            .or_else(|| {
                self.envs
                    .tick(clock, target, env_props)
                    .map(|env_mod| ModValue::Env(env_mod))
            })
            .unwrap_or_default()
    }
}
