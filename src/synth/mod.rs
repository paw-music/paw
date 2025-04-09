use crate::{
    daw::channel_rack::Instrument,
    midi::event::MidiEventListener,
    modulation::{env::EnvProps, lfo::LfoProps, mod_pack::ModPack, Modulate as _},
    osc::{
        clock::Clock,
        OpProps, Osc, OscParams,
    },
    sample::Frame,
    voice::{controller::VoicesController, Voice, VoiceParams},
};

#[derive(Clone)]
pub struct Synth<
    O: Osc,
    const VOICES: usize,
    const LFOS: usize,
    const ENVS: usize,
    const OSCS: usize,
> {
    lfo_props: [LfoProps; LFOS],
    env_props: [EnvProps; ENVS],

    /// Global ADSRs (envelopes) and LFOs
    mods: ModPack<LFOS, ENVS, OSCS>,

    osc_props: [OpProps<'static, O, OSCS>; OSCS],

    voices: VoicesController<O, VOICES, LFOS, ENVS, OSCS>,
}

impl<
        O: Osc + 'static,
        const VOICES: usize,
        const LFOS: usize,
        const ENVS: usize,
        const OSCS: usize,
    > Instrument for Synth<O, VOICES, LFOS, ENVS, OSCS>
{
    #[inline]
    fn tick(&mut self, clock: &Clock) -> Frame {
        self.tick(clock)
    }

    #[inline]
    fn name(&self) -> &str {
        "Synth"
    }

    #[cfg(feature = "egui")]
    fn egui(&mut self, ui: &mut egui::Ui, params: (Clock,)) {
        use crate::param::ui::{DefaultUiParams, EguiComponent as _};

        let params = DefaultUiParams { clock: params.0 };

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                self.voices.egui(ui, params);
                self.osc_props
                    .iter_mut()
                    .for_each(|props| props.egui(ui, params));
            });

            ui.horizontal(|ui| {
                self.lfo_props.iter_mut().for_each(|lfo| {
                    lfo.egui(ui, params);
                });

                self.env_props.iter_mut().for_each(|env| {
                    env.egui(ui, params);
                });
            });
        });
    }
}

impl<
        O: Osc + 'static,
        const VOICES: usize,
        const LFOS: usize,
        const ENVS: usize,
        const OSCS: usize,
    > MidiEventListener for Synth<O, VOICES, LFOS, ENVS, OSCS>
{
    #[inline]
    fn note_on(
        &mut self,
        clock: &Clock,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.mods.note_on(clock, note, velocity);
        self.voices.note_on(clock, note, velocity);
    }

    #[inline]
    fn note_off(
        &mut self,
        clock: &Clock,
        note: crate::midi::note::Note,
        velocity: crate::param::f32::UnitInterval,
    ) {
        self.mods.note_off(clock, note, velocity);
        self.voices.note_off(clock, note, velocity);
    }
}

impl<
        O: Osc + 'static,
        const VOICES: usize,
        const LFOS: usize,
        const ENVS: usize,
        const OSCS: usize,
    > Synth<O, VOICES, LFOS, ENVS, OSCS>
{
    pub fn new(sample_rate: u32, osc_props: impl Fn(usize) -> O::Props<'static>) -> Self {
        Self {
            lfo_props: core::array::from_fn(|index| LfoProps::new(index)),
            env_props: core::array::from_fn(|index| EnvProps::new(index, sample_rate)),
            mods: ModPack::new(),
            osc_props: core::array::from_fn(|index| OpProps::new(index, osc_props(index))),
            voices: VoicesController::new(|_| Voice::new(|_| O::default())),
        }
    }

    #[inline]
    pub fn props_mut(&mut self) -> &mut [OpProps<'static, O, OSCS>] {
        &mut self.osc_props
    }

    pub fn tick(&mut self, clock: &Clock) -> Frame {
        // Note: Need array allocation because we cannot pass slice (params are modulated) and don't want a vector
        let osc_params = core::array::from_fn(|index| OscParams {
            props: self.osc_props[index].modulated(|target| {
                self.mods
                    .tick(clock, target, &self.lfo_props, &self.env_props)
            }),
            pitch_mod: self.mods.tick(
                clock,
                crate::modulation::mod_pack::ModTarget::GlobalPitch,
                &self.lfo_props,
                &self.env_props,
            ),
        });

        let amp_mod = self.mods.tick(
            clock,
            crate::modulation::mod_pack::ModTarget::GlobalLevel,
            &self.lfo_props,
            &self.env_props,
        );

        let frame = self.voices.tick(
            clock,
            VoiceParams {
                env_params: &self.env_props,
                lfo_params: &self.lfo_props,
                osc_params,
                amp_mod,
            },
        );

        frame
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        midi::event::MidiEventListener, osc::clock::Clock, param::f32::UnitInterval,
        wavetable::synth::create_basic_wavetable_synth,
    };

    #[test]
    fn high_value_tick_precision() {
        let clock = Clock {
            sample_rate: 44_000,
            tick: 0,
        };

        let mut synth = create_basic_wavetable_synth::<1, 1, 1, 1>(clock.sample_rate);

        let note = crate::midi::note::Note::A4;
        synth.note_on(&clock, note, UnitInterval::MAX);
        let note_cycle = (clock.sample_rate as f32 / note.freq().inner()) as u32;

        assert_eq!(synth.tick(&clock), synth.tick(&clock.with_tick(note_cycle)));

        // assert_eq!(
        //     synth.tick(&clock),
        //     synth.tick(&clock.with_tick(clock.sample_rate * 11587))
        // );
    }
}
