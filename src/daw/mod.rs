use crate::{
    midi::event::MidiEventListener,
    osc::clock::{Clock, Tick},
    sample::Frame,
};
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
    #[inline]
    pub fn note_on(
        &mut self,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.rack.note_on(&self.clock, note, velocity);
        self.mixer.note_on(&self.clock, note, velocity);
    }

    #[inline]
    pub fn note_off(
        &mut self,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.rack.note_off(&self.clock, note, velocity);
        self.mixer.note_off(&self.clock, note, velocity);
    }

    #[inline(always)]
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

    #[inline(always)]
    pub fn rack_mut(&mut self) -> &mut ChannelRack<CHANNEL_RACK_SIZE> {
        &mut self.rack
    }

    #[inline(always)]
    pub fn mixer_mut(&mut self) -> &mut Mixer<MIXER_SIZE, FX_SLOTS> {
        &mut self.mixer
    }

    #[inline]
    pub fn tick_internal(&mut self) -> Frame {
        let output = self.tick_inner();
        self.clock.tick();
        output
    }

    #[inline]
    pub fn tick_external(&mut self, tick: u32) -> Frame {
        let output = self.tick_inner();
        self.clock.set(tick);
        output
    }

    #[inline]
    fn tick_inner(&mut self) -> Frame {
        let output = self.rack.tick_active(&self.clock);
        // TODO: Sequencer mode instead of just playing active or merge active channel mode and sequencer mode.

        let mixed = self.mixer.mix(&self.clock, output);

        mixed
    }

    /// Recommended instead of ticking. Processes a buffer at a time. This reduces overhead of `Box<dyn Instrument>` and `Box<dyn Fx>` as well as other values referencing during sample-by-sample processing with `tick*` methods. The usage of `process_buffer` does not guarantee that each DAW component will not use sample-by-sample method but avoids expensive values referencing while opening the door for compiler optimizations and caching.
    #[inline]
    pub fn process_buffer(&mut self, buffer: &mut [Frame]) {
        let track = self
            .rack
            .process_buffer_active::<MIXER_SIZE>(&self.clock, buffer);

        track.map(|track| self.mixer.mix_channel_buffer(&self.clock, track, buffer));

        self.clock.tick_for_buffer(buffer.len() as Tick);
    }
}

#[cfg(test)]
mod tests {
    use crate::{daw::Daw, osc::clock::Tick, sample::Frame};

    #[test]
    fn process_buffer_tick_equal() {
        const SAMPLE_RATE: u32 = 48_000;

        let mut buffer = [Frame::zero(); 1024];

        let mut daw = Daw::<1, 1, 0>::new(SAMPLE_RATE);

        daw.process_buffer(&mut buffer);

        assert!(buffer
            .iter()
            .enumerate()
            .all(|(index, sample)| { *sample == daw.tick_external(index as Tick) }));
    }
}
