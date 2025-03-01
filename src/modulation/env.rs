use super::mod_pack::ModTarget;
use crate::{
    midi::event::MidiEventListener,
    osc::clock::Clock,
    param::{f32::UnitInterval, ui::UiComponent},
    sample::time::SampleCount,
};

// #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// pub enum EnvTarget {
//     #[default]
//     SynthLevel,
//     SynthPitch,
//     WtPos(usize),
// }

#[derive(Debug)]
pub struct EnvProps {
    pub index: usize,
    pub enabled: bool,
    pub amount: UnitInterval,
    pub target: ModTarget,

    // Stages //
    pub delay: SampleCount,
    pub attack: SampleCount,
    pub hold: SampleCount,
    pub decay: SampleCount,
    pub sustain: UnitInterval,
    pub release: SampleCount,
}

impl UiComponent for EnvProps {
    fn ui(&mut self, ui: &mut impl crate::param::ui::ParamUi, params: &crate::param::ui::UiParams) {
        ui.v_stack(|ui| {
            ui.checkbox(&format!("Env{} enabled", self.index), &mut self.enabled);

            if !self.enabled {
                return;
            }

            let time_clamp = (
                SampleCount::from_millis(1, params.clock.sample_rate),
                SampleCount::from_seconds(10, params.clock.sample_rate),
            );

            ui.sample_count("Delay", &mut self.delay, Some(time_clamp), &params.clock);
            ui.sample_count("Attack", &mut self.attack, Some(time_clamp), &params.clock);
            ui.sample_count("Hold", &mut self.hold, Some(time_clamp), &params.clock);
            ui.sample_count("Decay", &mut self.decay, Some(time_clamp), &params.clock);
            ui.unit_interval("Sustain", &mut self.sustain);
            ui.sample_count(
                "Release",
                &mut self.release,
                Some(time_clamp),
                &params.clock,
            );

            // TODO
            // ui.select(
            //     "Target",
            //     &mut self.target,
            //     [
            //         ("Pitch", EnvTarget::SynthPitch),
            //         ("Level", EnvTarget::SynthLevel),
            //     ]
            //     .into_iter()
            //     .chain(
            //         (0..OSCS).map(|osc_index| ("Wavetable position", EnvTarget::WtPos(osc_index))),
            //     ),
            // );
        });
    }
}

impl EnvProps {
    // fn before_sustain(&self, position: u32) -> Option<f32> {
    //     [(self.delay, 0.0), (self.attack, ), self.hold, self.decay]
    //         .iter()
    //         .try_fold(
    //             (0, self.delay.inner()),
    //             |(stage_pos, stage_end), stage_len| {
    //                 if position <= stage_end {
    //                     Err(stage_pos as f32 / stage_len.inner() as f32)
    //                 } else {
    //                     Ok((position - stage_end, stage_end + stage_len.inner()))
    //                 }
    //             },
    //         )
    //         .err()
    // }

    pub fn new(index: usize, sample_rate: u32) -> Self {
        Self {
            index,
            enabled: false,
            amount: UnitInterval::MAX,
            target: Default::default(),
            delay: SampleCount::zero(),
            attack: SampleCount::from_millis(1, sample_rate),
            hold: SampleCount::zero(),
            decay: SampleCount::zero(),
            sustain: UnitInterval::MAX,
            release: SampleCount::from_millis(1, sample_rate),
        }
    }

    fn attack_endpoint(&self, velocity: f32) -> f32 {
        if !self.decay.is_zero() {
            velocity
        } else {
            self.sustain.inner()
        }
    }

    fn before_sustain(&self, pos: u32, velocity: f32) -> Option<f32> {
        let stage_end = self.delay.inner();

        if pos <= stage_end {
            return Some(0.0);
        }

        let stage_phase = pos - stage_end;
        let stage_end = stage_end + self.attack.inner();

        if pos <= stage_end {
            return Some(
                stage_phase as f32 / self.attack.inner() as f32 * self.attack_endpoint(velocity),
            );
        }

        let stage_end = stage_end + self.hold.inner();

        if pos <= stage_end {
            return Some(velocity);
        }

        let stage_phase = pos - stage_end;
        let stage_end = stage_end + self.decay.inner();

        if pos <= stage_end {
            return Some(
                1.0 - stage_phase as f32 / self.decay.inner() as f32
                    * (self.attack_endpoint(velocity) - self.sustain.inner()).abs(),
            );
        }

        None
    }

    fn after_sustain(&self, pos: u32) -> Option<f32> {
        if pos <= self.release.inner() {
            Some((1.0 - pos as f32 / self.release.inner() as f32) * self.sustain.inner())
        } else {
            None
        }
    }
}

enum EnvState {
    Idle,
    NoteOn {
        velocity: UnitInterval,
        at_tick: u32,
    },
    NoteOff {
        at_tick: u32,
    },
}

pub struct Env {
    state: EnvState,
}

impl MidiEventListener for Env {
    fn note_on(&mut self, clock: &Clock, _note: crate::midi::note::Note, velocity: UnitInterval) {
        self.state = EnvState::NoteOn {
            velocity,
            at_tick: clock.tick,
        }
    }

    fn note_off(&mut self, clock: &Clock, _note: crate::midi::note::Note, _velocity: UnitInterval) {
        self.state = EnvState::NoteOff {
            at_tick: clock.tick,
        }
    }
}

impl Env {
    pub fn new() -> Self {
        Self {
            state: EnvState::Idle,
        }
    }

    pub fn tick(&mut self, clock: &Clock, params: &EnvProps) -> Option<UnitInterval> {
        if !params.enabled {
            return None;
        }

        match self.state {
            EnvState::Idle => None,
            EnvState::NoteOn { velocity, at_tick } => {
                if let Some(before_sustain) =
                    params.before_sustain(clock.tick - at_tick, velocity.inner())
                {
                    Some(UnitInterval::new(before_sustain))
                } else {
                    Some(params.sustain)
                }
            }
            EnvState::NoteOff { at_tick } => params
                .after_sustain(clock.tick - at_tick)
                .map(UnitInterval::new),
        }
    }
}

pub struct EnvPack<const SIZE: usize> {
    envs: [Env; SIZE],
}

impl<const SIZE: usize> MidiEventListener for EnvPack<SIZE> {
    fn note_on(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        self.envs
            .iter_mut()
            .for_each(|env| env.note_on(clock, note, velocity));
    }

    fn note_off(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        self.envs
            .iter_mut()
            .for_each(|env| env.note_off(clock, note, velocity));
    }
}

impl<const SIZE: usize> EnvPack<SIZE> {
    pub fn new() -> Self {
        Self {
            envs: core::array::from_fn(|_| Env::new()),
        }
    }

    pub fn tick(
        &mut self,
        clock: &Clock,
        target: ModTarget,
        params: &[EnvProps],
    ) -> Option<UnitInterval> {
        debug_assert_eq!(params.len(), self.envs.len());

        params
            .iter()
            .zip(self.envs.iter_mut())
            .filter_map(|(params, env)| {
                if params.target == target {
                    env.tick(clock, params)
                } else {
                    None
                }
            })
            .next()
    }
}
