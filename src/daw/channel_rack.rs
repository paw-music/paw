use super::mixer::{TrackOutput, UnmixedOutput};
use crate::{
    midi::event::MidiEventListener,
    osc::clock::{Clock, Tick},
    sample::Frame,
};
use alloc::boxed::Box;

pub trait Instrument: MidiEventListener + Send {
    fn tick(&mut self, clock: &Clock) -> Frame;
    fn name(&self) -> &str;

    #[inline]
    fn process_buffer(&mut self, clock: &Clock, buffer: &mut [Frame]) {
        clock
            .for_buffer(buffer.len())
            .zip(buffer.iter_mut())
            .for_each(|(clock, frame)| *frame = self.tick(&clock));
    }

    #[cfg(feature = "egui")]
    fn egui(&mut self, ui: &mut egui::Ui, params: (Clock,));
}

pub struct RackChannel {
    // TODO: Volume and Panning
    mixer_track: usize,
    instrument: Box<dyn Instrument>,
}

#[cfg(feature = "egui")]
impl crate::param::ui::EguiComponent<(usize, bool, bool, usize, Clock), bool> for RackChannel {
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

impl MidiEventListener for RackChannel {
    #[inline]
    fn note_on(
        &mut self,
        clock: &Clock,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.instrument.note_on(clock, note, velocity);
    }

    #[inline]
    fn note_off(
        &mut self,
        clock: &Clock,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.instrument.note_off(clock, note, velocity);
    }
}

impl RackChannel {
    fn new(instrument: Box<dyn Instrument>) -> Self {
        Self {
            mixer_track: 0,
            instrument,
        }
    }

    #[inline]
    pub fn instrument_mut(&mut self) -> &mut dyn Instrument {
        self.instrument.as_mut()
    }

    #[inline]
    pub fn tick(&mut self, clock: &Clock) -> TrackOutput {
        TrackOutput::new(self.mixer_track, self.instrument.tick(clock))
    }

    #[inline]
    pub fn process_buffer(&mut self, clock: &Clock, buffer: &mut [Frame]) {
        self.instrument.process_buffer(clock, buffer);
    }
}

pub struct ChannelRack<const SIZE: usize> {
    channels: [Option<RackChannel>; SIZE],
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
                    todo!()
                    // self.push_instrument(Box::new(create_basic_wavetable_synth(clock.sample_rate)))
                    //     .unwrap();
                }
            }
        });
    }
}

impl<const SIZE: usize> MidiEventListener for ChannelRack<SIZE> {
    #[inline]
    fn note_on(
        &mut self,
        clock: &Clock,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.iter_channels_mut()
            .for_each(|channel| channel.note_on(clock, note, velocity));
    }

    #[inline]
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

    #[inline]
    pub fn is_full(&self) -> bool {
        self.channels.iter().all(Option::is_some)
    }

    pub fn push_instrument(&mut self, instrument: Box<dyn Instrument>) -> Result<usize, ()> {
        if let Some(id) = self
            .channels
            .iter_mut()
            .position(|channel| channel.is_none())
        {
            self.channels[id].replace(RackChannel::new(instrument));

            Ok(id)
        } else {
            Err(())
        }
    }

    #[inline]
    pub fn set_active(&mut self, active: usize) {
        self.active = Some(active);
    }

    // TODO: This should not tick but process active channel or played in sequencer
    // pub fn tick<const MIXER_SIZE: usize>(&mut self, clock: &Clock) -> UnmixedOutput<MIXER_SIZE> {
    //     self.channels
    //         .iter_mut()
    //         .filter_map(|inst| inst.as_mut().map(|inst| inst.tick(clock)))
    //         .fold(UnmixedOutput::zero(), |output, channel| output + channel)
    // }

    #[inline]
    fn iter_channels_mut(&mut self) -> impl Iterator<Item = &mut RackChannel> {
        self.channels
            .iter_mut()
            .filter_map(|channel| channel.as_mut())
    }

    #[inline]
    pub fn tick_active<const MIXER_SIZE: usize>(
        &mut self,
        clock: &Clock,
    ) -> UnmixedOutput<MIXER_SIZE> {
        self.active
            .and_then(|active| {
                self.channels[active].as_mut().map(|channel| {
                    self.is_active_playing = true;
                    channel.tick(clock).into()
                })
            })
            .unwrap_or_else(|| {
                self.is_active_playing = false;
                UnmixedOutput::zero()
            })
    }

    #[inline]
    pub fn process_buffer_active<const MIXER_SIZE: usize>(
        &mut self,
        clock: &Clock,
        buffer: &mut [Frame],
    ) -> Option<usize> {
        self.active.and_then(|active| {
            if let Some(channel) = self.channels[active].as_mut() {
                self.is_active_playing = true;
                channel.process_buffer(clock, buffer);
                Some(channel.mixer_track)
            } else {
                self.is_active_playing = false;
                None
            }
        })
    }
}
