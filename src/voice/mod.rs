use fixed::traits::ToFixed;

use crate::{
    components::adsr::{Adsr, AdsrParams},
    midi::note::Note,
    osc::Osc,
    param::{
        f32::{HalfUnitInterval, UnitInterval},
        ui::UiComponent,
    },
    sample::Sample,
    source::Source,
    value::freq::Freq,
};

pub struct Voice<O: Osc, const SAMPLE_RATE: u32>
where
    <O as Iterator>::Item: Sample,
{
    osc: O,
    amp: f32,
    adsr: Adsr<SAMPLE_RATE>,
}

impl<O: Osc, const SAMPLE_RATE: u32> Voice<O, SAMPLE_RATE>
where
    <O as Iterator>::Item: Sample,
{
    pub fn new(osc: O) -> Self {
        Self {
            osc,
            amp: 1.0,
            adsr: Adsr::new(),
        }
    }

    pub fn note_on(&mut self, velocity: f32) {
        debug_assert!(velocity >= 0.0 && velocity <= 1.0);
        self.adsr.note_on(velocity);
    }

    pub fn note_off(&mut self) {
        self.adsr.note_off();
    }

    pub fn osc_mut(&mut self) -> &mut O {
        &mut self.osc
    }

    pub fn osc(&self) -> &O {
        &self.osc
    }

    pub fn tick(&mut self, adsr: &AdsrParams<SAMPLE_RATE>) -> Option<O::Item> {
        self.osc.next().and_then(|sample| {
            self.adsr.tick(adsr).map(|env| {
                debug_assert!(env.is_finite());

                sample.amp(self.amp * env)
            })
        })
    }
}

// TODO: Blend mode. Like Center vs Detuned, Linear (more detuned voices blend less)
// TODO: Maybe for even number of voices there should be no center voices at all and all voices will be detuned? - Yes, fix this
/// Distribute detune by voices, returns iterator of (detune factor, voice amp determined by blend)
/// This is a distinct function to be used both for synthesizer and UI to draw unison parameters
pub fn voices_detune(
    count: usize,
    detune: UnitInterval,
    blend: HalfUnitInterval,
) -> impl Iterator<Item = (f32, f32)> {
    let half_detuned_voices = count as f32 / 2.0;

    (0..count).map(move |index| {
        if detune > 0.0 && half_detuned_voices > 0.0 {
            let center_offset = (index as f32 + 0.5) / half_detuned_voices - 1.0;

            (
                1.0 + center_offset * detune.inner(),
                if center_offset == 0.0 {
                    1.0 - blend.inner()
                } else {
                    blend.inner()
                }
                .sqrt(),
            )
        } else {
            (1.0, 1.0)
        }
    })
}

#[derive(Clone)]
struct VoiceGroup<const MAX_UNISON: usize> {
    // root_freq: f32,
    root_note: Note,
    // TODO: Optimize Option<usize> to usize with reserved None value or event better use Option<u8> optimized to u7 + opt bit
    voices_indices: [Option<usize>; MAX_UNISON],
}

#[derive(Clone, Default)]
struct MonoVoice {
    note: Option<Note>,
}

// TODO: Maybe we should merge `voices_notes` and place them separately
/// The polyphony mode
#[derive(Clone)]
enum Polyphony<const MAX_VOICES: usize> {
    // TODO: Add mono? Restricting to a specific mono voices count
    Poly {
        /// The index of the voice played last. Depending on NotePriority next triggered note will replace currently played one if there're no available voices.
        // TODO: This can be replaced with `next_note_index` that is picked by NotePriority.
        //  - For last it is the oldest triggered
        //  - For Highest and lowest the frequency comparison is done to store `next_note_index` with lowest/highest note
        last_voice_index: u8,
    },
    Unison {
        /// Unison voices count per voice group (one group per played note)
        unison: usize,

        /// Voices detune
        detune: UnitInterval,

        /// Amount of blend between less and more detuned voices where 0.0 is no detuned voices at all and 0.5 is the maximum value where detuned voices and center voice are equal in amplification
        blend: HalfUnitInterval,
    },
}

// FIXME: Not right
impl<const MAX_VOICES: usize> PartialEq for Polyphony<MAX_VOICES> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Poly { .. }, Self::Poly { .. }) | (Self::Unison { .. }, Self::Unison { .. }) => {
                true
            }
            _ => false,
        }
    }
}

// TODO: Use
/// The order by which newly triggered notes take precedence over currently playing notes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotePriority {
    /// Lower-frequency notes take precedence over higher-frequency notes when there's not enough free voices to play triggered note
    Lowest,
    /// Higher-frequency notes take precedence over lower-frequency notes
    Highest,
    /// If there's not enough free voices, the oldest triggered note is replaced with a new one
    Last,
    // TODO: In case gliding/portamento added, should notes be stacked in a queue so when voice is freed it glides to new note?
    /// The note is only played if there're enough free voices
    OnlyFree,
}

pub struct VoicesController<O: Osc, const VOICE_COUNT: usize, const SAMPLE_RATE: u32>
where
    <O as Iterator>::Item: Sample,
{
    voices: [Voice<O, SAMPLE_RATE>; VOICE_COUNT],
    /// Voices root notes.
    voices_notes: [MonoVoice; VOICE_COUNT],
    polyphony: Polyphony<VOICE_COUNT>,
    adsr: AdsrParams<SAMPLE_RATE>,
}

impl<O: Osc, const VOICE_COUNT: usize, const SAMPLE_RATE: u32> UiComponent
    for VoicesController<O, VOICE_COUNT, SAMPLE_RATE>
where
    <O as Iterator>::Item: Sample,
{
    fn ui(&mut self, ui: &mut impl crate::param::ui::ParamUi) {
        // TODO: Move from/to polyphony state
        ui.select(
            "Polyphony",
            &mut self.polyphony,
            &[
                (
                    "Poly",
                    Polyphony::Poly {
                        last_voice_index: 0,
                    },
                ),
                (
                    "Unison",
                    Polyphony::Unison {
                        unison: 1,
                        detune: UnitInterval::new(0.5),
                        blend: HalfUnitInterval::new(0.5),
                    },
                ),
            ],
        );

        if let Polyphony::Unison {
            unison,
            detune,
            blend,
        } = &mut self.polyphony
        {
            ui.count("Unison", unison, (1, VOICE_COUNT));
            ui.unit_interval("Detune", detune);
            ui.half_unit_interval("Blend", blend);
        }

        self.adsr.ui(ui);
    }
}

impl<O: Osc, const VOICE_COUNT: usize, const SAMPLE_RATE: u32>
    VoicesController<O, VOICE_COUNT, SAMPLE_RATE>
where
    <O as Iterator>::Item: Sample,
{
    pub fn voice_n(&self, index: usize) -> &Voice<O, SAMPLE_RATE> {
        &self.voices[index]
    }

    pub fn voices_detune(&self) -> Option<impl Iterator<Item = (f32, f32)>> {
        match &self.polyphony {
            Polyphony::Poly { .. } => None,
            &Polyphony::Unison {
                unison,
                detune,
                blend,
                ..
            } => Some(voices_detune(unison, detune, blend)),
        }
    }

    pub fn iter_voices_mut(&mut self) -> impl Iterator<Item = &mut Voice<O, SAMPLE_RATE>> {
        self.voices.iter_mut()
    }

    pub fn has_note(&self, note: Note) -> bool {
        self.voices_notes
            .iter()
            .any(|voice| voice.note == Some(note))
    }

    pub fn stop_all(&mut self) {
        self.voices_notes
            .iter_mut()
            .enumerate()
            .for_each(|(index, voice)| {
                if voice.note.take().is_some() {
                    self.voices[index].note_off();
                }
            })
    }

    pub fn note_on(&mut self, note: Note, velocity: f32) {
        // TODO: What to do if there're no available voices? Queue? Ignore?
        // TODO: Priority

        // TODO: Is this always right?
        if self.has_note(note) {
            return;
        }

        let mut note_on_voice = |index: usize, freq: Freq, velocity: f32, blend: f32| {
            self.voices[index].osc.set_freq(freq);
            self.voices[index].amp = blend;
            self.voices[index].note_on(velocity);
        };

        match &mut self.polyphony {
            Polyphony::Poly { last_voice_index } => {
                let oldest_index = (*last_voice_index as usize + 1) % VOICE_COUNT;

                // TODO: This will find next free voice, but only free as "not triggered" while it can still be in release ADSR stage
                // Find next free voice, if no free voice found, fallback to the oldest one and overwrite it
                let index = (oldest_index..oldest_index + VOICE_COUNT)
                    .find(|index| self.voices_notes[index % VOICE_COUNT].note.is_none())
                    .unwrap_or(oldest_index)
                    % VOICE_COUNT;

                note_on_voice(index, note.freq(), velocity, 1.0);
                self.voices_notes[index].note = Some(note);

                *last_voice_index = index as u8;
            }
            Polyphony::Unison {
                unison,
                detune,
                blend,
            } => {
                let unison = *unison;
                let detune = *detune;
                let blend = *blend;

                // TODO: NotePriority
                if self
                    .voices_notes
                    .iter()
                    .filter(|voice| voice.note.is_none())
                    .count()
                    < unison
                {
                    return;
                }

                self.voices_notes
                    .iter_mut()
                    .enumerate()
                    .filter_map(|(index, voice)| {
                        if voice.note.is_none() {
                            Some((index, voice))
                        } else {
                            None
                        }
                    })
                    .take(unison)
                    .zip(voices_detune(unison, detune, blend))
                    .for_each(|((voice_index, voice), (detune, blend))| {
                        note_on_voice(
                            voice_index,
                            Freq::from_num(note.freq().saturating_mul(detune.to_fixed())),
                            velocity,
                            blend,
                        );
                        voice.note = Some(note);
                    });
            }
        }
    }

    pub fn note_off(&mut self, note: Note) {
        self.voices_notes
            .iter_mut()
            .enumerate()
            .for_each(|(index, voice)| {
                if voice.note == Some(note) {
                    self.voices[index].note_off();
                    voice.note = None;
                }
            });
    }

    pub fn adsr(&self) -> &AdsrParams<SAMPLE_RATE> {
        &self.adsr
    }

    pub fn adsr_mut(&mut self) -> &mut AdsrParams<SAMPLE_RATE> {
        &mut self.adsr
    }
}

impl<O: Osc, const SIZE: usize, const SAMPLE_RATE: u32> VoicesController<O, SIZE, SAMPLE_RATE>
where
    <O as Iterator>::Item: Sample,
{
    pub fn new(f: impl Fn(usize) -> Voice<O, SAMPLE_RATE>) -> Self {
        Self {
            voices: core::array::from_fn(f),
            polyphony: Polyphony::Poly {
                last_voice_index: 0,
            },
            adsr: AdsrParams::default(),
            voices_notes: core::array::from_fn(|_| MonoVoice { note: None }),
        }
    }
}

impl<O: Osc, const SIZE: usize, const SAMPLE_RATE: u32> Iterator
    for VoicesController<O, SIZE, SAMPLE_RATE>
where
    O::Item: Sample,
{
    type Item = O::Item;

    fn next(&mut self) -> Option<Self::Item> {
        // Note: Check if this logic with Some(....sum()) is right. Maybe if all voices are off it should return None

        Some(
            (0..SIZE)
                .map(|index| {
                    self.voices[index]
                        .tick(&self.adsr)
                        .map(|sample| sample.amp(1.0 / SIZE as f32))
                        .unwrap_or(Self::Item::zero())
                })
                .sum(),
        )
    }
}

impl<O: Osc, const SIZE: usize, const SAMPLE_RATE: u32> Source
    for VoicesController<O, SIZE, SAMPLE_RATE>
where
    <O as Iterator>::Item: Sample,
{
}
