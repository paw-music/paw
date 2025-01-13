use crate::{
    adsr::{Adsr, AdsrParams},
    midi::note::Note,
    osc::Osc,
    sample::Sample,
    source::Source,
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
pub fn voices_detune(count: usize, detune: f32, blend: f32) -> impl Iterator<Item = (f32, f32)> {
    // Note: Detune is allowed to be zero, it is checked below and treated as "no detune"
    debug_assert!(detune >= 0.0 && detune <= 0.5, "Malformed detune {detune}");
    debug_assert!(blend >= 0.0 && blend <= 1.0, "Malformed blend {blend}");

    let half_detuned_voices = (count as f32 - 1.0) / 2.0;
    let center_voices = 2.0 - count as f32 % 1.0;
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

#[derive(Clone)]
struct MonoVoice {
    note: Option<Note>,
}

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
        voices_notes: [MonoVoice; MAX_VOICES],
    },
    Unison {
        /// Unison voices count per voice group (one group per played note)
        unison: u8,

        /// Voices detune
        detune: u8,

        /// Amount of blend between less and more detuned voices
        blend: f32,

        /// Voice groups. Each triggered note uses one group.
        groups: [Option<VoiceGroup<MAX_VOICES>>; MAX_VOICES],
    },
}

impl<const MAX_VOICES: usize> Polyphony<MAX_VOICES> {}

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
    polyphony: Polyphony<VOICE_COUNT>,
    adsr: AdsrParams<SAMPLE_RATE>,
}

impl<O: Osc, const VOICE_COUNT: usize, const SAMPLE_RATE: u32>
    VoicesController<O, VOICE_COUNT, SAMPLE_RATE>
where
    <O as Iterator>::Item: Sample,
{
    pub fn voice_n(&self, index: usize) -> &Voice<O, SAMPLE_RATE> {
        &self.voices[index]
    }

    pub fn has_note(&self, note: Note) -> bool {
        match &self.polyphony {
            Polyphony::Poly {
                last_voice_index,
                voices_notes,
            } => voices_notes.iter().any(|voice| voice.note == Some(note)),
            Polyphony::Unison {
                unison,
                detune,
                blend,
                groups,
            } => {
                todo!()
            }
        }
    }

    pub fn stop_all(&mut self) {
        match &mut self.polyphony {
            Polyphony::Poly {
                last_voice_index,
                voices_notes,
            } => voices_notes
                .iter_mut()
                .enumerate()
                .for_each(|(index, voice)| {
                    if voice.note.take().is_some() {
                        self.voices[index].note_off();
                    }
                }),
            Polyphony::Unison {
                unison,
                detune,
                blend,
                groups,
            } => todo!(),
        }
    }

    pub fn note_on(&mut self, note: Note, velocity: f32) {
        // TODO: What to do if there're no available voices? Queue? Ignore?
        // TODO: Priority

        // TODO: Is this always right?
        if self.has_note(note) {
            return;
        }

        match &mut self.polyphony {
            Polyphony::Poly {
                last_voice_index,
                voices_notes,
            } => {
                // let next_index = (*last_voice_index as usize + 1);
                // let index = (next_index..VOICE_COUNT + next_index).find(|index| {
                //     let index = index % VOICE_COUNT;
                //     if
                // });

                // Note: This is [`NotePriority::Last`]

                let oldest_index = (*last_voice_index as usize + 1) % VOICE_COUNT;

                // TODO: This will find next free voice, but only free as "not triggered" while it can still be in release ADSR stage
                // Find next free voice, if no free voice found, fallback to the oldest one and overwrite it
                let index = (oldest_index..oldest_index + VOICE_COUNT)
                    .find(|index| voices_notes[index % VOICE_COUNT].note.is_none())
                    .unwrap_or(oldest_index)
                    % VOICE_COUNT;

                self.voices[index].osc.set_freq(note.freq());
                self.voices[index].note_on(velocity);
                voices_notes[index].note = Some(note);

                *last_voice_index = index as u8;
            }
            Polyphony::Unison {
                unison,
                detune,
                blend,
                groups,
            } => {
                // Note: This is [`NotePriority::Last`]

                // let free_voices = (0..VOICE_COUNT).filter_map(|index| groups.iter().filter_map(|group| group.as_ref()).find(|group| group))
                todo!()
            }
        }
    }

    pub fn note_off(&mut self, note: Note) {
        match &mut self.polyphony {
            Polyphony::Poly {
                last_voice_index: _,
                voices_notes,
            } => {
                if let Some(index) = voices_notes
                    .iter()
                    .position(|voice| voice.note == Some(note))
                {
                    self.voices[index].note_off();
                    voices_notes[index].note = None;
                }
            }
            Polyphony::Unison {
                unison,
                detune,
                blend,
                groups,
            } => {}
        }
    }

    pub fn adsr(&self) -> &AdsrParams<SAMPLE_RATE> {
        &self.adsr
    }

    pub fn adsr_mut(&mut self) -> &mut AdsrParams<SAMPLE_RATE> {
        &mut self.adsr
    }

    // pub fn iter_all_voices_mut(&mut self) -> impl Iterator<Item = &mut Voice<O>> {
    //     self.voices.iter_mut()
    // }

    // pub fn iter_active_voices_mut(&mut self) -> impl Iterator<Item = &mut Voice<O>> {
    //     self.voices.iter_mut().take(self.unison)
    // }

    // pub fn iter_active_voices(&self) -> impl Iterator<Item = &Voice<O>> {
    //     self.voices.iter().take(self.unison)
    // }

    // pub fn voices_detune(&self) -> impl Iterator<Item = (f32, f32)> {
    //     voices_detune(self.unison(), self.detune, self.blend)
    // }

    // fn distribute_freq(&mut self) {
    //     let center_freq = self.center_freq;
    //     let detunes = voices_detune(self.unison(), self.detune, self.blend);

    //     self.iter_active_voices_mut()
    //         .zip(detunes)
    //         .for_each(|(voice, (detune, amp))| {
    //             // debug_assert!(amp >= 0.0 && amp <= 1.0, "Malformed amp {amp}");

    //             let voice_freq = center_freq * detune;

    //             voice.osc.set_freq(voice_freq);
    //             voice.amp = amp;
    //         });
    // }
}

// impl<O: Osc, const SIZE: usize> Osc for VoicesController<O, SIZE>
// where
//     <O as Iterator>::Item: Sample,
// {
//     fn set_freq(&mut self, freq: f32) -> &mut Self {
//         debug_assert!(
//             freq >= 0.0 && freq.is_finite(),
//             "Malformed frequency {freq}"
//         );

//         self.center_freq = freq;
//         self.distribute_freq();
//         self
//     }

//     fn freq(&self) -> f32 {
//         self.center_freq
//     }

//     fn reset(&mut self) -> &mut Self {
//         self.iter_active_voices_mut().for_each(|voice| {
//             voice.osc.reset();
//         });
//         self
//     }
// }

impl<O: Osc, const SIZE: usize, const SAMPLE_RATE: u32> VoicesController<O, SIZE, SAMPLE_RATE>
where
    <O as Iterator>::Item: Sample,
{
    pub fn new(f: impl Fn(usize) -> Voice<O, SAMPLE_RATE>) -> Self {
        Self {
            voices: core::array::from_fn(f),
            polyphony: Polyphony::Poly {
                last_voice_index: 0,
                voices_notes: core::array::from_fn(|_| MonoVoice { note: None }),
            },
            adsr: AdsrParams::default(),
        }
    }
}

//     /// Clamps active to the size of the voice stack and a minimum is a single voice
//     pub fn set_unison(&mut self, unison: usize) -> &mut Self {
//         self.unison = unison.clamp(1, SIZE);
//         self.distribute_freq();
//         self
//     }

//     /// Count of active voices
//     pub fn unison(&self) -> usize {
//         self.unison
//     }

//     pub fn set_detune(&mut self, detune: f32) -> &mut Self {
//         let detune = detune.clamp(0.0, 1.0);
//         self.detune = detune;
//         self.distribute_freq();
//         self
//     }

//     pub fn detune(&self) -> f32 {
//         self.detune
//     }

//     pub fn set_blend(&mut self, blend: f32) -> &mut Self {
//         let blend = blend.clamp(0.0, 1.0);
//         self.blend = blend;
//         self.distribute_freq();
//         self
//     }

//     pub fn blend(&self) -> f32 {
//         self.blend
//     }
// }

impl<O: Osc, const SIZE: usize, const SAMPLE_RATE: u32> Iterator
    for VoicesController<O, SIZE, SAMPLE_RATE>
where
    O::Item: Sample,
{
    type Item = O::Item;

    fn next(&mut self) -> Option<Self::Item> {
        // let attenuation = (self.active_count() as f32).sqrt();
        // let voice_count = self.unison();
        // let mix = self
        //     .iter_active_voices_mut()
        //     .map(|voice| (voice.osc.next().unwrap(), voice.amp))
        //     .enumerate()
        //     .fold(O::Item::zero(), |mix, (index, (sample, amp))| {
        //         // sample.amp(amp).fold_mean(mix, index)
        //         mix + sample.amp(amp / voice_count as f32)
        //     });

        // Some(mix)

        // Note: Check if this logic with Some(....sum()) is right. Maybe if all voices are off it should return None

        match &self.polyphony {
            Polyphony::Poly {
                last_voice_index,
                voices_notes,
            } => Some(
                (0..SIZE)
                    .map(|index| {
                        self.voices[index]
                            .tick(&self.adsr)
                            .map(|sample| sample.amp(1.0 / SIZE as f32))
                            .unwrap_or(Self::Item::zero())
                    })
                    .sum(),
            ),
            Polyphony::Unison {
                unison,
                detune,
                blend,
                groups,
            } => todo!(),
        }
    }
}

impl<O: Osc, const SIZE: usize, const SAMPLE_RATE: u32> Source
    for VoicesController<O, SIZE, SAMPLE_RATE>
where
    <O as Iterator>::Item: Sample,
{
}
