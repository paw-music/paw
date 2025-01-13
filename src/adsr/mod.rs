use crate::sample::time::SampleCount;
use num::clamp;

// const BASE_SAMPLE_RATE: u32 = 48_000;

// /// Minimum attack possible to set
// const MIN_ATTACK: SampleCount<BASE_SAMPLE_RATE> = SampleCount::from_millis(1);
// const MIN_RELEASE: SampleCount<BASE_SAMPLE_RATE> = SampleCount::from_millis(1);

// TODO: Delay and hold
// TODO: Complex envelopes
/// ADSR parameters in samples
pub struct AdsrParams<const SAMPLE_RATE: u32> {
    attack: SampleCount<SAMPLE_RATE>,
    decay: SampleCount<SAMPLE_RATE>,
    sustain: f32,
    release: SampleCount<SAMPLE_RATE>,
}

impl<const SAMPLE_RATE: u32> AdsrParams<SAMPLE_RATE> {
    // Note: Useless, as sustain is infinite
    // pub fn length(&self) -> SampleCount<SAMPLE_RATE> {
    //     self.attack + self.decay + self.release
    // }

    pub fn attack(&self) -> SampleCount<SAMPLE_RATE> {
        self.attack
    }

    pub fn decay(&self) -> SampleCount<SAMPLE_RATE> {
        self.decay
    }

    pub fn sustain(&self) -> f32 {
        self.sustain
    }

    pub fn release(&self) -> SampleCount<SAMPLE_RATE> {
        self.release
    }

    pub fn set_attack(&mut self, attack: SampleCount<SAMPLE_RATE>) -> &mut Self {
        self.attack = clamp(attack, SampleCount::from_millis(1), SampleCount::MAX);
        self
    }

    pub fn set_decay(&mut self, decay: SampleCount<SAMPLE_RATE>) -> &mut Self {
        self.decay = decay;
        self
    }

    pub fn set_sustain(&mut self, sustain: f32) -> &mut Self {
        self.sustain = sustain.clamp(0.0, 1.0);
        self
    }

    pub fn set_release(&mut self, release: SampleCount<SAMPLE_RATE>) -> &mut Self {
        self.release = clamp(release, SampleCount::from_millis(1), SampleCount::MAX);
        self
    }
}

impl<const SAMPLE_RATE: u32> Default for AdsrParams<SAMPLE_RATE> {
    fn default() -> Self {
        Self {
            attack: SampleCount::from_millis(1),
            decay: SampleCount::ZERO,
            sustain: 1.0,
            release: SampleCount::from_millis(1),
        }
    }
}

/// Current ADSR stage and state
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AdsrStage {
    None,
    Attack(u32),
    Decay(u32),
    Sustain,
    Release(u32),
}

impl AdsrStage {
    fn note_on(&mut self) {
        *self = Self::Attack(0);
    }

    fn note_off(&mut self) {
        *self = Self::Release(0);
    }

    /// Increment sample index, returns true if ADSR stage is changed
    fn maybe_advance<const SAMPLE_RATE: u32>(&mut self, params: &AdsrParams<SAMPLE_RATE>) -> bool {
        match self {
            Self::None => false,
            Self::Attack(sample) => {
                if params.attack <= *sample {
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
            params.sustain
        }
    }

    pub fn tick(&mut self, params: &AdsrParams<SAMPLE_RATE>) -> Option<f32> {
        while self.stage.maybe_advance(&params) {}

        match self.stage {
            AdsrStage::None => None,
            AdsrStage::Attack(sample) => {
                Some(sample as f32 / params.attack.inner() as f32 * self.attack_endpoint(params))
            }
            AdsrStage::Decay(sample) => Some(
                1.0 - sample as f32 / params.decay.inner() as f32
                    * ((self.attack_endpoint(params) - params.sustain).abs()),
            ),
            AdsrStage::Sustain => Some(params.sustain),
            AdsrStage::Release(sample) => {
                Some(params.sustain - sample as f32 / params.release.inner() as f32 * params.sustain)
            }
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
