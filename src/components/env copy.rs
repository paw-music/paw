use crate::{
    midi::{event::MidiEventListener, note::Note},
    osc::clock::Clock,
    param::{f32::UnitInterval, ui::UiComponent},
    sample::time::SampleCount,
};

// const BASE_SAMPLE_RATE: u32 = 48_000;

// /// Minimum attack possible to set
// const MIN_ATTACK: SampleCount<BASE_SAMPLE_RATE> = SampleCount::from_millis(1);
// const MIN_RELEASE: SampleCount<BASE_SAMPLE_RATE> = SampleCount::from_millis(1);

// TODO: Non-linear envelopes
/// ADSR parameters in samples
#[derive(Debug)]
pub struct EnvParams<const OSCS: usize> {
    pub enabled: bool,
    pub amount: UnitInterval,
    pub target: EnvTarget,
    pub delay: SampleCount,
    pub attack: SampleCount,
    pub hold: SampleCount,
    pub decay: SampleCount,
    pub sustain: UnitInterval,
    pub release: SampleCount,
}

impl<const OSCS: usize> EnvParams<OSCS> {
    pub fn stage(&self, phase: u32) -> EnvStage {
        let stage_phase = phase;
        let stage_end = self.delay.inner();

        if phase <= stage_end {
            return EnvStage::Delay(UnitInterval::new_checked(
                stage_phase as f32 / self.delay.inner() as f32,
            ));
        }

        let stage_phase = phase - stage_end;
        let stage_end = stage_end + self.attack.inner();

        if phase <= stage_end {
            return EnvStage::Attack(UnitInterval::new(
                stage_phase as f32 / self.attack.inner() as f32,
            ));
        }

        let stage_phase = phase - stage_end;
        let stage_end = stage_end + self.hold.inner();

        if phase <= stage_end {
            return EnvStage::Hold(UnitInterval::new(
                stage_phase as f32 / self.hold.inner() as f32,
            ));
        }

        let stage_phase = phase - stage_end;
        let stage_end = stage_end + self.decay.inner();

        if phase <= stage_end {
            return EnvStage::Decay(UnitInterval::new(
                stage_phase as f32 / self.decay.inner() as f32,
            ));
        }
    }

    // pub fn before_sustain(&self) -> SampleCount {
    //     self.delay + self.attack + self.hold + self.decay
    // }
}

impl<const OSCS: usize> UiComponent for EnvParams<OSCS> {
    fn ui(&mut self, ui: &mut impl crate::param::ui::ParamUi, params: &crate::param::ui::UiParams) {
        ui.v_stack(|ui| {
            ui.checkbox("Env enabled", &mut self.enabled);

            if !self.enabled {
                return;
            }

            let time_clamp = (
                SampleCount::from_millis(1, &params.clock),
                SampleCount::from_seconds(10, &params.clock),
            );

            ui.sample_count("Delay", &mut self.delay, Some(time_clamp), &params.clock);
            ui.sample_count("Attack", &mut self.attack, Some(time_clamp), &params.clock);
            ui.sample_count("Hold", &mut self.hold, Some(time_clamp), &params.clock);
            ui.sample_count("Decay", &mut self.decay, Some(time_clamp), &params.clock);
            ui.unit_interval("Sustain", &mut self.sustain);
            ui.sample_count(
                "Release",
                &mut self.release,
                Some(time_clamp),
                &params.clock,
            );

            ui.select(
                "Target",
                &mut self.target,
                [
                    ("Pitch", EnvTarget::SynthPitch),
                    ("Level", EnvTarget::SynthLevel),
                ]
                .into_iter()
                .chain(
                    (0..OSCS).map(|osc_index| ("Wavetable position", EnvTarget::WtPos(osc_index))),
                ),
            );
        });
    }
}

impl<const OSCS: usize> Default for EnvParams<OSCS> {
    fn default() -> Self {
        Self {
            enabled: false,
            amount: UnitInterval::MAX,
            target: Default::default(),
            delay: SampleCount::ZERO,
            attack: SampleCount::new(50),
            hold: SampleCount::ZERO,
            decay: SampleCount::ZERO,
            sustain: UnitInterval::MAX,
            release: SampleCount::new(50),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum EnvStage {
    Idle,
    Delay(UnitInterval),
    Attack(UnitInterval),
    Hold(UnitInterval),
    Decay(UnitInterval),
    Sustain,
    Release(UnitInterval),
}

// /// Current ADSR stage and count of samples before next stage
// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// enum EnvStage {
//     Idle,
//     Delay(u32),
//     Attack(u32),
//     Hold(u32),
//     Decay(u32),
//     Sustain,
//     Release(u32),
// }

// impl EnvStage {
//     fn note_on(&mut self) {
//         *self = Self::Delay(0);
//     }

//     fn note_off(&mut self) {
//         // TODO: Smart note_off with declicking. Not going right to the release but ending current stage softly.
//         *self = Self::Release(0);
//     }

//     /// Increment sample index, returns true if ADSR stage is changed
//     fn maybe_advance<const OSCS: usize>(&mut self, params: &EnvParams<OSCS>) -> bool {
//         match self {
//             Self::Idle => false,
//             Self::Delay(sample) => {
//                 if params.delay <= *sample {
//                     *self = Self::Attack(0);
//                     true
//                 } else {
//                     *sample += 1;
//                     false
//                 }
//             }
//             Self::Attack(sample) => {
//                 if params.attack <= *sample {
//                     *self = Self::Hold(0);
//                     true
//                 } else {
//                     *sample += 1;
//                     false
//                 }
//             }
//             Self::Hold(sample) => {
//                 if params.hold <= *sample {
//                     *self = Self::Decay(0);
//                     true
//                 } else {
//                     *sample += 1;
//                     false
//                 }
//             }
//             Self::Decay(sample) => {
//                 if params.decay <= *sample {
//                     *self = Self::Sustain;
//                     true
//                 } else {
//                     *sample += 1;
//                     false
//                 }
//             }
//             Self::Sustain => false,
//             Self::Release(sample) => {
//                 if params.release <= *sample {
//                     *self = Self::Idle;
//                     true
//                 } else {
//                     *sample += 1;
//                     false
//                 }
//             }
//         }
//     }
// }

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnvTarget {
    #[default]
    SynthLevel,
    SynthPitch,
    WtPos(usize),
}

pub struct Env<const OSCS: usize> {
    stage: EnvStage,
    /// The attack peak (triggered note velocity)
    velocity: f32,
    start: u32,
}

impl<const OSCS: usize> MidiEventListener for Env<OSCS> {
    fn note_on(&mut self, clock: &Clock, note: Note, velocity: UnitInterval) {
        let _ = note;
        self.start = clock.tick;
        self.velocity = velocity.inner();
        self.stage.note_on();
    }

    fn note_off(&mut self, clock: &Clock, note: Note, velocity: UnitInterval) {
        let _ = clock;
        let _ = velocity;
        let _ = note;
        self.stage.note_off();
    }
}

impl<const OSCS: usize> Env<OSCS> {
    pub fn new() -> Self {
        Self {
            stage: EnvStage::Idle,
            velocity: 0.0,
            start: 0,
        }
    }

    fn attack_endpoint(&self, params: &EnvParams<OSCS>) -> f32 {
        // TODO: Check this. It will avoid usage of peak (note velocity) in case of zero decay. This is done to avoid high-frequency amplitude jump, but may be incorrect for user needs.
        if params.decay > SampleCount::ZERO {
            self.velocity
        } else {
            params.sustain.inner()
        }
    }

    /// [0.0; 1.0]
    pub fn tick(&mut self, params: &EnvParams<OSCS>) -> Option<UnitInterval> {
        if !params.enabled {
            return Some(UnitInterval::MAX);
        }

        while self.stage.maybe_advance(&params) {}

        let value = match self.stage {
            EnvStage::Idle => return None,
            EnvStage::Delay(_) => 0.0,
            EnvStage::Attack(sample) => {
                sample as f32 / params.attack.inner() as f32 * self.attack_endpoint(params)
            }
            EnvStage::Hold(_) => self.velocity,
            EnvStage::Decay(sample) => {
                1.0 - sample as f32 / params.decay.inner() as f32
                    * ((self.attack_endpoint(params) - params.sustain.inner()).abs())
            }
            EnvStage::Sustain => params.sustain.inner(),
            EnvStage::Release(sample) => {
                params.sustain.inner()
                    - sample as f32 / params.release.inner() as f32 * params.sustain.inner()
            }
        };

        Some(UnitInterval::new_checked(value))
    }
}

pub struct EnvPack<const SIZE: usize, const OSCS: usize> {
    envs: [Env<OSCS>; SIZE],
}

impl<const SIZE: usize, const OSCS: usize> MidiEventListener for EnvPack<SIZE, OSCS> {
    fn note_on(&mut self, clock: &Clock, note: Note, velocity: UnitInterval) {
        self.envs
            .iter_mut()
            .for_each(|env| env.note_on(clock, note, velocity));
    }

    fn note_off(&mut self, clock: &Clock, note: Note, velocity: UnitInterval) {
        self.envs
            .iter_mut()
            .for_each(|env| env.note_off(clock, note, velocity));
    }
}

impl<const SIZE: usize, const OSCS: usize> EnvPack<SIZE, OSCS> {
    pub fn new() -> Self {
        Self {
            envs: core::array::from_fn(|_| Env::new()),
        }
    }

    pub fn tick(&mut self, target: EnvTarget, params: &[EnvParams<OSCS>]) -> Option<UnitInterval> {
        debug_assert_eq!(params.len(), self.envs.len());

        params
            .iter()
            .zip(self.envs.iter_mut())
            .filter_map(|(params, env)| {
                if params.target == target {
                    Some(env.tick(params).unwrap_or(UnitInterval::MIN))
                } else {
                    None
                }
            })
            .next()
    }
}
