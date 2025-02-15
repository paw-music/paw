use super::{Voice, VoiceParams};
use crate::{
    midi::{event::MidiEventListener, note::Note},
    osc::{clock::Clock, Osc},
    param::{
        f32::{HalfUnitInterval, SignedUnitInterval, UnitInterval},
        ui::UiComponent,
    },
};

// TODO: Blend mode. Like Center vs Detuned, Linear (more detuned voices blend less)
/// Distribute detune by voices, returns iterator of (detune factor, voice amp determined by blend)
/// This is a distinct function to be used both for synthesizer and UI to draw unison parameters
pub fn voices_detune(
    count: usize,
    detune: UnitInterval,
    blend: HalfUnitInterval,
) -> impl Iterator<Item = (SignedUnitInterval, UnitInterval)> {
    let half_detuned_voices = count as f32 / 2.0;
    let center_area = 1.0 / count as f32;

    (0..count).map(move |index| {
        if detune > 0.0 && half_detuned_voices > 0.0 {
            let center_offset = (index as f32 + 0.5) / half_detuned_voices - 1.0;

            (
                SignedUnitInterval::new(center_offset * detune.inner()),
                // This equation means that centered voices are one or two nearest to center. For odd number of voices, there's a center voice with `center_offset` being 0.0, while for even number of voices there're two with distance dependent on count of voices.
                UnitInterval::new(
                    if center_offset.abs() - center_area <= f32::EPSILON {
                        1.0 - blend.inner()
                    } else {
                        blend.inner()
                    }
                    .sqrt(),
                ),
            )
        } else {
            (SignedUnitInterval::EQUILIBRIUM, UnitInterval::MAX)
        }
    })
}

// #[derive(Clone)]
// struct VoiceGroup<const MAX_UNISON: usize> {
//     // root_freq: f32,
//     root_note: Note,
//     // TODO: Optimize Option<usize> to usize with reserved None value or event better use Option<u8> optimized to u7 + opt bit
//     voices_indices: [Option<usize>; MAX_UNISON],
// }

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

pub struct VoicesController<
    O: Osc,
    const VOICES: usize,
    const LFOS: usize,
    const ENVS: usize,
    const OSCS: usize,
> {
    voices: [Voice<O, LFOS, ENVS, OSCS>; VOICES],
    /// Voices root notes.
    voices_notes: [MonoVoice; VOICES],
    polyphony: Polyphony<VOICES>,
}

impl<O: Osc, const VOICES: usize, const LFOS: usize, const ENVS: usize, const OSCS: usize>
    UiComponent for VoicesController<O, VOICES, LFOS, ENVS, OSCS>
{
    fn ui(
        &mut self,
        ui: &mut impl crate::param::ui::ParamUi,
        _ui_params: &crate::param::ui::UiParams,
    ) {
        ui.v_stack(|ui| {
            // TODO: Move from/to polyphony state
            ui.select(
                "Polyphony",
                &mut self.polyphony,
                [
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
                ]
                .into_iter(),
            );

            if let Polyphony::Unison {
                unison,
                detune,
                blend,
            } = &mut self.polyphony
            {
                ui.lines(
                    voices_detune(*unison, *detune, *blend).map(|(detune, blend)| {
                        let x = 1.0 - detune.inner();
                        // let is_center = detune == 1.0;

                        ((x, 1.0), (x, -blend.inner()))
                    }),
                );

                ui.count("Unison", unison, (1, VOICES));
                ui.unit_interval("Detune", detune);
                ui.half_unit_interval("Blend", blend);
            }
        });
    }
}

impl<
        O: Osc + 'static,
        const VOICES: usize,
        const LFOS: usize,
        const ENVS: usize,
        const OSCS: usize,
    > MidiEventListener for VoicesController<O, VOICES, LFOS, ENVS, OSCS>
{
    fn note_on(&mut self, note: Note, velocity: UnitInterval) {
        // TODO: What to do if there're no available voices? Queue? Ignore?
        // TODO: Priority

        // TODO: Is this always right?
        if self.has_note(note) {
            return;
        }

        let mut note_on_voice = |index: usize,
                                 note: Note,
                                 velocity: UnitInterval,
                                 detune: SignedUnitInterval,
                                 blend: UnitInterval| {
            // FIXME: Voice amp is not right for blend, use separate state
            self.voices[index].set_detune(blend, detune);
            self.voices[index].note_on(note, velocity);
        };

        match &mut self.polyphony {
            Polyphony::Poly { last_voice_index } => {
                let oldest_index = (*last_voice_index as usize + 1) % VOICES;

                // TODO: This will find next free voice, but only free as "not triggered" while it can still be in release ADSR stage
                // Find next free voice, if no free voice found, fallback to the oldest one and overwrite it
                let index = (oldest_index..oldest_index + VOICES)
                    .find(|index| self.voices_notes[index % VOICES].note.is_none())
                    .unwrap_or(oldest_index)
                    % VOICES;

                note_on_voice(
                    index,
                    note,
                    velocity,
                    SignedUnitInterval::EQUILIBRIUM,
                    UnitInterval::MAX,
                );
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
                        note_on_voice(voice_index, note, velocity, detune, blend);
                        voice.note = Some(note);
                    });
            }
        }
    }

    fn note_off(&mut self, note: Note, velocity: UnitInterval) {
        let _ = velocity;

        self.voices_notes
            .iter_mut()
            .enumerate()
            .for_each(|(index, voice)| {
                if voice.note == Some(note) {
                    self.voices[index].note_off(note, velocity);
                    voice.note = None;
                }
            });
    }
}

impl<
        O: Osc + 'static,
        const VOICES: usize,
        const LFOS: usize,
        const ENVS: usize,
        const OSCS: usize,
    > VoicesController<O, VOICES, LFOS, ENVS, OSCS>
{
    pub fn new(f: impl Fn(usize) -> Voice<O, LFOS, ENVS, OSCS>) -> Self {
        Self {
            voices: core::array::from_fn(f),
            polyphony: Polyphony::Poly {
                last_voice_index: 0,
            },
            voices_notes: core::array::from_fn(|_| MonoVoice { note: None }),
        }
    }

    pub fn voice_n(&self, index: usize) -> &Voice<O, LFOS, ENVS, OSCS> {
        &self.voices[index]
    }

    pub fn voices_detune(
        &self,
    ) -> Option<impl Iterator<Item = (SignedUnitInterval, UnitInterval)>> {
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

    pub fn iter_voices_mut(&mut self) -> impl Iterator<Item = &mut Voice<O, LFOS, ENVS, OSCS>> {
        self.voices.iter_mut()
    }

    pub fn has_note(&self, note: Note) -> bool {
        self.voices_notes
            .iter()
            .any(|voice| voice.note == Some(note))
    }

    // pub fn stop_all(&mut self) {
    //     self.voices_notes
    //         .iter_mut()
    //         .enumerate()
    //         .for_each(|(index, voice)| {
    //             if voice.note.take().is_some() {
    //                 self.voices[index].note_off();
    //             }
    //         })
    // }

    pub fn tick<'a>(
        &mut self,
        clock: &Clock,
        voice_params: &VoiceParams<'a, O, OSCS>,
    ) -> Option<SignedUnitInterval> {
        let sample = (0..VOICES)
            .map(|index| {
                self.voices[index]
                    .tick(clock, voice_params)
                    .map(|sample| sample / VOICES as f32)
                    .unwrap_or(SignedUnitInterval::EQUILIBRIUM)
            })
            .sum();

        Some(sample)
    }
}
