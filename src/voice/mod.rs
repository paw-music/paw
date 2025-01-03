use crate::{osc::Osc, sample::Sample, source::Source};

#[derive(Clone)]
pub struct Voice<O: Osc>
where
    <O as Iterator>::Item: Sample,
{
    osc: O,
    amp: f32,
}

impl<O: Osc> Voice<O>
where
    <O as Iterator>::Item: Sample,
{
    pub fn new(osc: O) -> Self {
        Self { osc, amp: 1.0 }
    }

    pub fn osc_mut(&mut self) -> &mut O {
        &mut self.osc
    }

    pub fn osc(&self) -> &O {
        &self.osc
    }
}

// TODO: Blend mode. Like Center vs Detuned, Linear (more detuned voices blend less)
// TODO: Maybe for even number of voices there should be no center voices at all and all voices will be detuned?
/// Distribute detune by voices, returns iterator of (detune factor, voice amp determined by blend)
pub fn voices_detune(count: usize, detune: f32, blend: f32) -> impl Iterator<Item = (f32, f32)> {
    // Note: Detune is allowed to be zero, it is checked below and treated as "no detune"
    debug_assert!(detune >= 0.0 && detune <= 0.5, "Malformed detune {detune}");
    debug_assert!(blend >= 0.0 && blend <= 1.0, "Malformed blend {blend}");

    let half_detuned_voices = (count as f32 - 1.0) / 2.0;
    // let attenuation = 1.0 / (count as f32).sqrt();
    // Center blend
    // let blend = blend - 0.5;

    (0..count).map(move |index| {
        if detune > 0.0 && half_detuned_voices > 0.0 {
            let center_offset = (index as f32 - half_detuned_voices).trunc() / half_detuned_voices;

            (
                1.0 + center_offset * detune,
                if center_offset == 0.0 {
                    1.0 - blend
                } else {
                    blend
                },
            )
        } else {
            (1.0, 1.0)
        }
    })
}

#[derive(Clone)]
pub struct VoiceStack<O: Osc, const SIZE: usize>
where
    <O as Iterator>::Item: Sample,
{
    voices: [Voice<O>; SIZE],
    active: usize,
    center_freq: f32,
    blend: f32,
    detune: f32,
}

impl<O: Osc, const SIZE: usize> VoiceStack<O, SIZE>
where
    <O as Iterator>::Item: Sample,
{
    pub fn iter_all_voices_mut(&mut self) -> impl Iterator<Item = &mut Voice<O>> {
        self.voices.iter_mut()
    }

    pub fn iter_active_voices_mut(&mut self) -> impl Iterator<Item = &mut Voice<O>> {
        self.voices.iter_mut().take(self.active)
    }

    pub fn iter_active_voices(&self) -> impl Iterator<Item = &Voice<O>> {
        self.voices.iter().take(self.active)
    }

    pub fn voices_detune(&self) -> impl Iterator<Item = (f32, f32)> {
        voices_detune(self.active_count(), self.detune, self.blend)
    }

    fn distribute_freq(&mut self) {
        let center_freq = self.center_freq;
        let detunes = voices_detune(self.active_count(), self.detune, self.blend);

        self.iter_active_voices_mut()
            .zip(detunes)
            .for_each(|(voice, (detune, amp))| {
                // debug_assert!(amp >= 0.0 && amp <= 1.0, "Malformed amp {amp}");

                let voice_freq = center_freq * detune;

                voice.osc.set_freq(voice_freq);
                voice.amp = amp;
            });
    }
}

impl<O: Osc, const SIZE: usize> Osc for VoiceStack<O, SIZE>
where
    <O as Iterator>::Item: Sample,
{
    fn set_freq(&mut self, freq: f32) -> &mut Self {
        debug_assert!(
            freq >= 0.0 && freq.is_finite(),
            "Malformed frequency {freq}"
        );

        self.center_freq = freq;
        self.distribute_freq();
        self
    }

    fn freq(&self) -> f32 {
        self.center_freq
    }

    fn reset(&mut self) -> &mut Self {
        self.iter_active_voices_mut().for_each(|voice| {
            voice.osc.reset();
        });
        self
    }
}

impl<O: Osc, const SIZE: usize> VoiceStack<O, SIZE>
where
    <O as Iterator>::Item: Sample,
{
    pub fn new(f: impl Fn(usize) -> Voice<O>) -> Self {
        Self {
            voices: core::array::from_fn(f),
            active: 1,
            center_freq: 0.0,
            detune: 0.0,
            blend: 0.5,
        }
    }

    /// Clamps active to the size of the voice stack and a minimum is a single voice
    pub fn set_active_voices(&mut self, active: usize) -> &mut Self {
        self.active = active.clamp(1, SIZE);
        self.distribute_freq();
        self
    }

    /// Count of active voices
    pub fn active_count(&self) -> usize {
        self.active
    }

    pub fn set_detune(&mut self, detune: f32) -> &mut Self {
        let detune = detune.clamp(0.0, 1.0);
        self.detune = detune;
        self.distribute_freq();
        self
    }

    pub fn detune(&self) -> f32 {
        self.detune
    }

    pub fn set_blend(&mut self, blend: f32) -> &mut Self {
        let blend = blend.clamp(0.0, 1.0);
        self.blend = blend;
        self.distribute_freq();
        self
    }

    pub fn blend(&self) -> f32 {
        self.blend
    }
}

impl<O: Osc, const SIZE: usize> Iterator for VoiceStack<O, SIZE>
where
    O::Item: Sample,
{
    type Item = O::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let mean = self
            .iter_active_voices_mut()
            .map(|voice| (voice.osc.next().unwrap(), voice.amp))
            .enumerate()
            .fold(O::Item::zero(), |mix, (index, (sample, amp))| {
                sample.amp(amp).fold_mean(mix, index)
            });

        Some(mean)
    }
}

impl<O: Osc, const SIZE: usize> Source for VoiceStack<O, SIZE> where <O as Iterator>::Item: Sample {}
