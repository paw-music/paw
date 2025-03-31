use crate::{midi::event::MidiEventListener, osc::clock::Clock, sample::Frame};
use channel_rack::ChannelRack;
use mixer::Mixer;

pub mod channel_rack;
pub mod mixer;

pub enum ClockSource {
    Internal,
    External(u32),
}

pub struct Daw<const CHANNEL_RACK_SIZE: usize, const MIXER_SIZE: usize, const FX_SLOTS: usize> {
    rack: ChannelRack<CHANNEL_RACK_SIZE>,
    mixer: Mixer<MIXER_SIZE, FX_SLOTS>,
    clock: Clock,
}

#[cfg(feature = "egui")]
impl<const CHANNEL_RACK_SIZE: usize, const MIXER_SIZE: usize, const FX_SLOTS: usize>
    crate::param::ui::EguiComponent<()> for Daw<CHANNEL_RACK_SIZE, MIXER_SIZE, FX_SLOTS>
{
    fn egui(&mut self, ui: &mut ::egui::Ui, _: ()) {
        let params = crate::param::ui::DefaultUiParams { clock: self.clock };

        egui::SidePanel::left("Channel rack")
            .resizable(false)
            .show_inside(ui, |ui| {
                self.rack.egui(ui, (MIXER_SIZE, params.clock));
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.mixer.egui(ui, params);
        });

        // ui.horizontal(|ui| {
        //     ui.allocate_ui(
        //         egui::vec2(ui.available_width() / 3.0, ui.available_height()),
        //         |ui| {
        //             ui.set_width(ui.available_width());
        //             self.rack.egui(ui, (MIXER_SIZE, params.clock));
        //         },
        //     );

        //     ui.separator();

        //     self.mixer.egui(ui, params);
        // });
    }
}

impl<const CHANNEL_RACK_SIZE: usize, const MIXER_SIZE: usize, const FX_SLOTS: usize>
    Daw<CHANNEL_RACK_SIZE, MIXER_SIZE, FX_SLOTS>
{
    pub fn note_on(
        &mut self,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.rack.note_on(&self.clock, note, velocity);
        self.mixer.note_on(&self.clock, note, velocity);
    }

    pub fn note_off(
        &mut self,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.rack.note_off(&self.clock, note, velocity);
        self.mixer.note_off(&self.clock, note, velocity);
    }

    pub fn clock(&self) -> Clock {
        self.clock
    }
}

impl<const CHANNEL_RACK_SIZE: usize, const MIXER_SIZE: usize, const FX_SLOTS: usize>
    Daw<CHANNEL_RACK_SIZE, MIXER_SIZE, FX_SLOTS>
{
    pub fn new(sample_rate: u32) -> Self {
        Self {
            rack: ChannelRack::new(),
            mixer: Mixer::new(),
            clock: Clock::zero(sample_rate),
        }
    }

    pub fn rack_mut(&mut self) -> &mut ChannelRack<CHANNEL_RACK_SIZE> {
        &mut self.rack
    }

    pub fn mixer_mut(&mut self) -> &mut Mixer<MIXER_SIZE, FX_SLOTS> {
        &mut self.mixer
    }

    pub fn tick_internal(&mut self) -> Frame {
        let output = self.tick_inner();
        self.clock.tick();
        output
    }

    pub fn tick_external(&mut self, tick: u32) -> Frame {
        let output = self.tick_inner();
        self.clock.set(tick);
        output
    }

    fn tick_inner(&mut self) -> Frame {
        let output = self.rack.tick_active(&self.clock);
        // TODO: Sequencer mode instead of just playing active or merge active channel mode and sequencer mode.

        let mixed = self.mixer.mix(&self.clock, output);

        mixed
    }
}
