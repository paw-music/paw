use core::{iter::Sum, ops::Add};

use alloc::boxed::Box;

use crate::{
    fx::Fx, midi::event::MidiEventListener, osc::clock::Clock, param::f32::UnitInterval,
    sample::Frame,
};

/// Single track output to be applied to mixer output
pub struct TrackOutput {
    track: usize,
    output: Frame,
}

impl TrackOutput {
    pub fn new(track: usize, output: Frame) -> Self {
        Self { track, output }
    }
}

pub struct UnmixedOutput<const SIZE: usize> {
    tracks: [Frame; SIZE],
}

impl<const SIZE: usize> From<TrackOutput> for UnmixedOutput<SIZE> {
    fn from(value: TrackOutput) -> Self {
        Self {
            tracks: core::array::from_fn(|index| {
                if index == value.track {
                    value.output
                } else {
                    Frame::zero()
                }
            }),
        }
    }
}

impl<const SIZE: usize> Add<TrackOutput> for UnmixedOutput<SIZE> {
    type Output = Self;

    fn add(mut self, rhs: TrackOutput) -> Self::Output {
        self.tracks[rhs.track] += rhs.output;
        self
    }
}

impl<const SIZE: usize> Add for UnmixedOutput<SIZE> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.zip(rhs, |lhs, rhs| lhs + rhs)
    }
}

impl<const SIZE: usize> Sum for UnmixedOutput<SIZE> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |sum, mo| sum + mo)
    }
}

impl<const SIZE: usize> UnmixedOutput<SIZE> {
    pub fn zero() -> Self {
        Self {
            tracks: [Frame::zero(); SIZE],
        }
    }

    // pub fn single(track: usize, output: f32) -> Self {
    //     Self {
    //         tracks: core::array::from_fn(|index| if index == track { output } else { Frame::zero() }),
    //     }
    // }

    pub fn from_fn(f: impl FnMut(usize) -> Frame) -> Self {
        Self {
            tracks: core::array::from_fn(f),
        }
    }

    pub fn zip(
        &self,
        other: UnmixedOutput<SIZE>,
        f: impl Fn(Frame, Frame) -> Frame,
    ) -> UnmixedOutput<SIZE> {
        UnmixedOutput::from_fn(|index| f(self.tracks[index], other.tracks[index]))
    }
}

pub(super) struct MixerTrack<const FX_SLOTS: usize> {
    // TODO: Disable fx
    // TODO: Mute
    // TODO: Panning
    // TODO!: Decibel level
    pub(super) level: UnitInterval,
    pub(super) effects: [Option<Box<dyn Fx>>; FX_SLOTS],
}

#[cfg(feature = "egui")]
impl<const FX_SLOTS: usize> crate::param::ui::EguiComponent<(usize, Clock)>
    for MixerTrack<FX_SLOTS>
{
    fn egui(&mut self, ui: &mut egui::Ui, (index, clock): (usize, Clock)) {
        ui.vertical(|ui| {
            ui.set_max_width(50.0);
            ui.vertical_centered(|ui| {
                ui.label(format!("{index}"));
            });

            ui.vertical_centered(|ui| {
                ui.add(self.level.widget().vertical());
            });

            // TODO: Render only focused effect
            self.iter_effects_mut().for_each(|fx| {
                fx.egui(ui, (clock,));
            });
        });
    }
}

impl<const FX_SLOTS: usize> MidiEventListener for MixerTrack<FX_SLOTS> {
    fn note_on(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        self.iter_effects_mut()
            .for_each(|fx| fx.note_on(clock, note, velocity));
    }

    fn note_off(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        self.iter_effects_mut()
            .for_each(|fx| fx.note_off(clock, note, velocity));
    }
}

impl<const FX_SLOTS: usize> MixerTrack<FX_SLOTS> {
    const fn new() -> Self {
        Self {
            level: UnitInterval::MAX,
            effects: [const { None }; FX_SLOTS],
        }
    }

    pub fn iter_effects_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn Fx>> {
        self.effects.iter_mut().filter_map(|fx| fx.as_mut())
    }

    fn mix(&mut self, clock: &Clock, input: Frame) -> Frame {
        let output = self.effects.iter_mut().fold(input, |input, fx| {
            fx.as_mut().map(|fx| fx.tick(clock, input)).unwrap_or(input)
        });

        let output = output * self.level.inner();

        output
    }
}

pub struct Mixer<const SIZE: usize, const FX_SLOTS: usize> {
    pub(super) tracks: [MixerTrack<FX_SLOTS>; SIZE],
}

#[cfg(feature = "egui")]
impl<const SIZE: usize, const FX_SLOTS: usize> crate::param::ui::EguiComponent
    for Mixer<SIZE, FX_SLOTS>
{
    fn egui(&mut self, ui: &mut egui::Ui, params: crate::param::ui::DefaultUiParams) {
        ui.horizontal(|ui| {
            self.tracks
                .iter_mut()
                .enumerate()
                .for_each(|(index, track)| {
                    if index > 0 {
                        ui.separator();
                    }
                    track.egui(ui, (index, params.clock))
                });
        });
    }
}

impl<const SIZE: usize, const FX_SLOTS: usize> MidiEventListener for Mixer<SIZE, FX_SLOTS> {
    fn note_on(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        self.tracks
            .iter_mut()
            .for_each(|track| track.note_on(clock, note, velocity));
    }

    fn note_off(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        self.tracks
            .iter_mut()
            .for_each(|track| track.note_off(clock, note, velocity));
    }
}

impl<const SIZE: usize, const FX_SLOTS: usize> Mixer<SIZE, FX_SLOTS> {
    pub const fn new() -> Self {
        Self {
            tracks: [const { MixerTrack::new() }; SIZE],
        }
    }

    pub fn mix(&mut self, clock: &Clock, input: UnmixedOutput<SIZE>) -> Frame {
        self.tracks
            .iter_mut()
            .zip(input.tracks)
            .fold(Frame::zero(), |mix, (track, input)| {
                mix + track.mix(clock, input)
            })
    }
}
