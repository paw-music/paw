use crate::{
    param::{f32::UnitInterval, ui::UiComponent},
    sample::time::SampleCount,
};

// const BASE_SAMPLE_RATE: u32 = 48_000;

// /// Minimum attack possible to set
// const MIN_ATTACK: SampleCount<BASE_SAMPLE_RATE> = SampleCount::from_millis(1);
// const MIN_RELEASE: SampleCount<BASE_SAMPLE_RATE> = SampleCount::from_millis(1);

// TODO: Non-linear envelopes
/// ADSR parameters in samples
pub struct AdsrParams<const SAMPLE_RATE: u32> {
    pub delay: SampleCount<SAMPLE_RATE>,
    pub attack: SampleCount<SAMPLE_RATE>,
    pub hold: SampleCount<SAMPLE_RATE>,
    pub decay: SampleCount<SAMPLE_RATE>,
    pub sustain: UnitInterval,
    pub release: SampleCount<SAMPLE_RATE>,
}

impl<const SAMPLE_RATE: u32> UiComponent for AdsrParams<SAMPLE_RATE> {
    fn ui(&mut self, ui: &mut impl crate::param::ui::ParamUi) {
        let time_clamp = (SampleCount::from_millis(1), SampleCount::from_seconds(10));

        ui.sample_count("Delay", &mut self.delay, Some(time_clamp));
        ui.sample_count("Attack", &mut self.attack, Some(time_clamp));
        ui.sample_count("Hold", &mut self.hold, Some(time_clamp));
        ui.sample_count("Decay", &mut self.decay, Some(time_clamp));
        ui.unit_interval("Sustain", &mut self.sustain);
        ui.sample_count("Release", &mut self.release, Some(time_clamp));
    }
}

impl<const SAMPLE_RATE: u32> AdsrParams<SAMPLE_RATE> {
    // pub fn with_params(&mut self, mut f: impl FnMut(Param)) {
    //     f(Param::new("Delay", &mut self.delay).clamped(
    //         SampleCount::<SAMPLE_RATE>::ZERO,
    //         SampleCount::<SAMPLE_RATE>::from_millis(10_000),
    //     ));
    //     f(Param::new("Attack", &mut self.attack).clamped(
    //         SampleCount::<SAMPLE_RATE>::ZERO,
    //         SampleCount::<SAMPLE_RATE>::from_millis(10_000),
    //     ));
    //     f(Param::new("Hold", &mut self.hold).clamped(
    //         SampleCount::<SAMPLE_RATE>::ZERO,
    //         SampleCount::<SAMPLE_RATE>::from_millis(10_000),
    //     ));
    //     f(Param::new("Decay", &mut self.decay).clamped(
    //         SampleCount::<SAMPLE_RATE>::ZERO,
    //         SampleCount::<SAMPLE_RATE>::from_millis(10_000),
    //     ));
    //     f(Param::new("Sustain", &mut self.sustain));
    //     f(Param::new("Release", &mut self.release).clamped(
    //         SampleCount::<SAMPLE_RATE>::ZERO,
    //         SampleCount::<SAMPLE_RATE>::from_millis(10_000),
    //     ));
    // }

    pub fn before_sustain(&self) -> SampleCount<SAMPLE_RATE> {
        self.delay + self.attack + self.hold + self.decay
    }

    // pub fn delay(&self) -> SampleCount<SAMPLE_RATE> {
    //     self.delay
    // }

    // pub fn attack(&self) -> SampleCount<SAMPLE_RATE> {
    //     self.attack
    // }

    // pub fn hold(&self) -> SampleCount<SAMPLE_RATE> {
    //     self.hold
    // }

    // pub fn decay(&self) -> SampleCount<SAMPLE_RATE> {
    //     self.decay
    // }

    // pub fn sustain(&self) -> UnitInterval {
    //     self.sustain
    // }

    // pub fn release(&self) -> SampleCount<SAMPLE_RATE> {
    //     self.release
    // }

    // pub fn set_delay(&mut self, delay: SampleCount<SAMPLE_RATE>) -> &mut Self {
    //     self.delay = delay;
    //     self
    // }

    // pub fn set_attack(&mut self, attack: SampleCount<SAMPLE_RATE>) -> &mut Self {
    //     self.attack = clamp(attack, SampleCount::from_millis(1), SampleCount::MAX);
    //     self
    // }

    // pub fn set_hold(&mut self, hold: SampleCount<SAMPLE_RATE>) -> &mut Self {
    //     self.hold = hold;
    //     self
    // }

    // pub fn set_decay(&mut self, decay: SampleCount<SAMPLE_RATE>) -> &mut Self {
    //     self.decay = decay;
    //     self
    // }

    // pub fn set_sustain(&mut self, sustain: f32) -> &mut Self {
    //     self.sustain = sustain.clamp(0.0, 1.0);
    //     self
    // }

    // pub fn set_release(&mut self, release: SampleCount<SAMPLE_RATE>) -> &mut Self {
    //     self.release = clamp(release, SampleCount::from_millis(1), SampleCount::MAX);
    //     self
    // }
}

impl<const SAMPLE_RATE: u32> Default for AdsrParams<SAMPLE_RATE> {
    fn default() -> Self {
        Self {
            delay: SampleCount::ZERO,
            attack: SampleCount::from_millis(1),
            hold: SampleCount::ZERO,
            decay: SampleCount::ZERO,
            sustain: UnitInterval::new(1.0),
            release: SampleCount::from_millis(1),
        }
    }
}

/// Current ADSR stage and state
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AdsrStage {
    None,
    Delay(u32),
    Attack(u32),
    Hold(u32),
    Decay(u32),
    Sustain,
    Release(u32),
}

impl AdsrStage {
    fn note_on(&mut self) {
        *self = Self::Delay(0);
    }

    fn note_off(&mut self) {
        *self = Self::Release(0);
    }

    /// Increment sample index, returns true if ADSR stage is changed
    fn maybe_advance<const SAMPLE_RATE: u32>(&mut self, params: &AdsrParams<SAMPLE_RATE>) -> bool {
        match self {
            Self::None => false,
            Self::Delay(sample) => {
                if params.delay <= *sample {
                    *self = Self::Attack(0);
                    true
                } else {
                    *sample += 1;
                    false
                }
            }
            Self::Attack(sample) => {
                if params.attack <= *sample {
                    *self = Self::Hold(0);
                    true
                } else {
                    *sample += 1;
                    false
                }
            }
            Self::Hold(sample) => {
                if params.hold <= *sample {
                    *self = Self::Decay(0);
                    true
                } else {
                    *sample += 1;
                    false
                }
            }
            Self::Decay(sample) => {
                if params.decay <= *sample {
                    *self = Self::Sustain;
                    true
                } else {
                    *sample += 1;
                    false
                }
            }
            Self::Sustain => false,
            Self::Release(sample) => {
                if params.release <= *sample {
                    *self = Self::None;
                    true
                } else {
                    *sample += 1;
                    false
                }
            }
        }
    }
}

pub struct Adsr<const SAMPLE_RATE: u32> {
    stage: AdsrStage,
    /// The attack peak (triggered note velocity)
    velocity: f32,
}

impl<const SAMPLE_RATE: u32> Adsr<SAMPLE_RATE> {
    pub fn new() -> Self {
        Self {
            stage: AdsrStage::None,
            velocity: 0.0,
        }
    }

    pub fn note_on(&mut self, velocity: f32) {
        self.velocity = velocity;
        self.stage.note_on();
    }

    pub fn note_off(&mut self) {
        self.stage.note_off();
    }

    fn attack_endpoint(&self, params: &AdsrParams<SAMPLE_RATE>) -> f32 {
        // TODO: Check this. It will avoid usage of peak (note velocity) in case of zero decay. This is done to avoid high-frequency amplitude jump, but may be incorrect for user needs.
        if params.decay > SampleCount::ZERO {
            self.velocity
        } else {
            params.sustain.inner()
        }
    }

    pub fn tick(&mut self, params: &AdsrParams<SAMPLE_RATE>) -> Option<f32> {
        while self.stage.maybe_advance(&params) {}

        match self.stage {
            AdsrStage::None => None,
            AdsrStage::Delay(_) => Some(0.0),
            AdsrStage::Attack(sample) => {
                Some(sample as f32 / params.attack.inner() as f32 * self.attack_endpoint(params))
            }
            AdsrStage::Hold(_) => Some(self.velocity),
            AdsrStage::Decay(sample) => Some(
                1.0 - sample as f32 / params.decay.inner() as f32
                    * ((self.attack_endpoint(params) - params.sustain.inner()).abs()),
            ),
            AdsrStage::Sustain => Some(params.sustain.inner()),
            AdsrStage::Release(sample) => Some(
                params.sustain.inner()
                    - sample as f32 / params.release.inner() as f32 * params.sustain.inner(),
            ),
        }
    }

    // pub fn into_iter(self, params: &AdsrParams<SAMPLE_RATE>) -> AdsrIterator<'_, SAMPLE_RATE> {
    //     AdsrIterator { params, adsr: self }
    // }
}

// pub struct AdsrIterator<'a, const SAMPLE_RATE: u32> {
//     params: &'a AdsrParams<SAMPLE_RATE>,
//     adsr: Adsr<SAMPLE_RATE>,
// }

// impl<'a, const SAMPLE_RATE: u32> Iterator for AdsrIterator<'a, SAMPLE_RATE> {
//     type Item = f32;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.adsr.tick(&self.params)
//     }
// }
