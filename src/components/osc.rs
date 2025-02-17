use crate::{
    midi::event::MidiEventListener,
    osc::Osc,
    param::ui::UiComponent,
    voice::{controller::VoicesController, Voice, VoiceParams},
};

pub struct OscInstance<
    O: Osc,
    const VOICES: usize,
    const SAMPLE_RATE: u32,
    const LFOS: usize,
    const ENVS: usize,
    const OSCS: usize,
> {
    voices: VoicesController<O, VOICES, SAMPLE_RATE, LFOS, ENVS, OSCS>,
}

impl<
        O: Osc + 'static,
        const VOICES: usize,
        const SAMPLE_RATE: u32,
        const LFOS: usize,
        const ENVS: usize,
        const OSCS: usize,
    > MidiEventListener for OscInstance<O, VOICES, SAMPLE_RATE, LFOS, ENVS, OSCS>
{
    fn note_on(
        &mut self,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.voices.note_on(clock, note, velocity);
    }

    fn note_off(
        &mut self,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.voices.note_off(clock, note, velocity);
    }
}

impl<
        O: Osc,
        const VOICES: usize,
        const SAMPLE_RATE: u32,
        const LFOS: usize,
        const ENVS: usize,
        const OSCS: usize,
    > UiComponent for OscInstance<O, VOICES, SAMPLE_RATE, LFOS, ENVS, OSCS>
{
    fn ui(&mut self, ui: &mut impl crate::param::ui::ParamUi) {
        self.voices.ui(ui);
    }
}

impl<
        O: Osc + 'static,
        const VOICES: usize,
        const SAMPLE_RATE: u32,
        const LFOS: usize,
        const ENVS: usize,
        const OSCS: usize,
    > OscInstance<O, VOICES, SAMPLE_RATE, LFOS, ENVS, OSCS>
{
    pub fn new(osc: impl Fn(usize) -> O + Clone) -> Self {
        Self {
            voices: VoicesController::new(|_| Voice::new(osc.clone())),
        }
    }

    pub fn tick<'a>(
        &mut self,
        params: &VoiceParams<'a, O, SAMPLE_RATE, OSCS>,
    ) -> Option<O::Output> {
        self.voices.tick(params)
    }
}
