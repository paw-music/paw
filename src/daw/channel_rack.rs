use super::mixer::{TrackOutput, UnmixedOutput};
use crate::{
    midi::event::MidiEventListener, osc::clock::Clock, sample::Frame,
    wavetable::synth::create_wavetable_synth,
};
use alloc::boxed::Box;

pub trait Instrument: MidiEventListener + Send {
    fn tick(&mut self, clock: &Clock) -> Frame;
    fn name(&self) -> &str;

    #[cfg(feature = "egui")]
    fn egui(&mut self, ui: &mut egui::Ui, params: (Clock,));
}

pub(super) struct Channel {
    // TODO: Volume and Panning
    mixer_track: usize,
    instrument: Box<dyn Instrument>,
}

#[cfg(feature = "egui")]
impl crate::param::ui::EguiComponent<(usize, bool, bool, usize, Clock), bool> for Channel {
    #[must_use]
    fn egui(
        &mut self,
        ui: &mut egui::Ui,
        (index, active, playing, mixer_size, clock): (usize, bool, bool, usize, Clock),
    ) -> bool {
        use egui::Widget as _;

        let mut toggle_active = false;

        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut self.mixer_track).clamp_range(0..=mixer_size));

            toggle_active = egui::Button::new(self.instrument.name())
                .fill(if playing {
                    egui::Color32::from_black_alpha(255)
                } else {
                    egui::Color32::from_black_alpha(0)
                })
                .ui(ui)
                .clicked();

            if active {
                egui::Window::new(format!("{}[{index}]", self.instrument.name()))
                    .auto_sized()
                    .show(ui.ctx(), |ui| {
                        self.instrument.egui(ui, (clock,));
                    });
            }
        });

        toggle_active
    }
}

impl MidiEventListener for Channel {
    fn note_on(
        &mut self,
        clock: &Clock,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.instrument.note_on(clock, note, velocity);
    }

    fn note_off(
        &mut self,
        clock: &Clock,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.instrument.note_off(clock, note, velocity);
    }
}

impl Channel {
    fn new(instrument: Box<dyn Instrument>) -> Self {
        Self {
            mixer_track: 0,
            instrument,
        }
    }

    pub fn instrument_mut(&mut self) -> &mut dyn Instrument {
        self.instrument.as_mut()
    }

    pub fn tick(&mut self, clock: &Clock) -> TrackOutput {
        TrackOutput::new(self.mixer_track, self.instrument.tick(clock))
    }
}

pub struct ChannelRack<const SIZE: usize> {
    channels: [Option<Channel>; SIZE],
    active: Option<usize>,
    is_active_playing: bool,
}

#[cfg(feature = "egui")]
impl<const SIZE: usize> crate::param::ui::EguiComponent<(usize, Clock)> for ChannelRack<SIZE> {
    fn egui(&mut self, ui: &mut egui::Ui, (mixer_size, clock): (usize, Clock)) {
        ui.vertical(|ui| {
            let active = self.active;
            let is_active_playing = self.is_active_playing;
            let new_active = self
                .iter_channels_mut()
                .enumerate()
                .fold(None, |new_active, (index, channel)| {
                    let is_active = active == Some(index);
                    let toggle_active =
                        channel.egui(ui, (index, is_active, is_active_playing, mixer_size, clock));

                    new_active.or(if toggle_active {
                        if is_active {
                            None
                        } else {
                            Some(index)
                        }
                    } else {
                        None
                    })
                })
                .or(self.active);

            self.active = new_active;

            // TODO: Plugin manager
            if !self.is_full() {
                if ui.button("Add instrument").clicked() {
                    self.push_instrument(Box::new(create_wavetable_synth(clock.sample_rate)))
                        .unwrap();
                }
            }
        });
    }
}

impl<const SIZE: usize> MidiEventListener for ChannelRack<SIZE> {
    fn note_on(
        &mut self,
        clock: &Clock,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.iter_channels_mut()
            .for_each(|channel| channel.note_on(clock, note, velocity));
    }

    fn note_off(
        &mut self,
        clock: &Clock,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.iter_channels_mut()
            .for_each(|channel| channel.note_off(clock, note, velocity));
    }
}

impl<const SIZE: usize> ChannelRack<SIZE> {
    pub fn new() -> Self {
        Self {
            channels: [const { None }; SIZE],
            active: None,
            is_active_playing: false,
        }
    }

    pub fn is_full(&self) -> bool {
        self.channels.iter().all(Option::is_some)
    }

    pub fn push_instrument(&mut self, instrument: Box<dyn Instrument>) -> Result<(), ()> {
        if let Some(next) = self.channels.iter_mut().find(|channel| channel.is_none()) {
            next.replace(Channel::new(instrument));
            Ok(())
        } else {
            Err(())
        }
    }

    // TODO: This should not tick but process active channel or played in sequencer
    // pub fn tick<const MIXER_SIZE: usize>(&mut self, clock: &Clock) -> UnmixedOutput<MIXER_SIZE> {
    //     self.channels
    //         .iter_mut()
    //         .filter_map(|inst| inst.as_mut().map(|inst| inst.tick(clock)))
    //         .fold(UnmixedOutput::zero(), |output, channel| output + channel)
    // }

    fn iter_channels_mut(&mut self) -> impl Iterator<Item = &mut Channel> {
        self.channels
            .iter_mut()
            .filter_map(|channel| channel.as_mut())
    }

    pub fn tick_active<const MIXER_SIZE: usize>(
        &mut self,
        clock: &Clock,
    ) -> UnmixedOutput<MIXER_SIZE> {
        if let Some(channel) = self
            .active
            .map(|active| self.channels[active].as_mut())
            .flatten()
        {
            self.is_active_playing = true;
            channel.tick(clock).into()
        } else {
            self.is_active_playing = false;
            UnmixedOutput::zero()
        }
    }
}
