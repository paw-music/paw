use crate::{
    param::{
        f32::{SignedUnitInterval, UnitInterval},
        ui::UiComponent,
    },
    value::freq::modulate_freq,
};
use clock::Clock;

pub mod clock;

pub trait Osc: Sized {
    type Props<'a>;

    fn tick<'a>(&mut self, phase: f32, params: &Self::Props<'a>) -> SignedUnitInterval;
}

#[derive(Debug, Clone, Copy)]
pub enum OscOutput {
    Direct,
    FMNext,
    AMNext,
    RMNext,
    // TODO: PM? Phase modulation
}

#[derive(Debug, Clone, Copy)]
pub enum OscMod {
    Direct(SignedUnitInterval),
    FM(SignedUnitInterval),
    AM(UnitInterval),
    RM(SignedUnitInterval),
}

/// The properties of oscillator component. Global for all oscillator instances.
pub struct OscProps<'a, O: Osc> {
    osc: O::Props<'a>,
    output: OscOutput,
}

impl<'a, O: Osc> OscProps<'a, O> {
    pub fn new(osc: O::Props<'a>) -> Self {
        Self {
            osc,
            output: OscOutput::Direct,
        }
    }
}

impl<'a, O: Osc> UiComponent for OscProps<'a, O>
where
    O::Props<'a>: UiComponent,
{
    fn ui(
        &mut self,
        ui: &mut impl crate::param::ui::ParamUi,
        ui_params: &crate::param::ui::UiParams,
    ) {
        self.osc.ui(ui, ui_params);
        // TODO: Output
    }
}

// /// Parameters single oscillator instance receives
// pub struct OscParams {
//     pub freq: f32,
// }

// impl OscParams {
//     pub fn with_freq(&self, freq: f32) -> Self {
//         Self { freq, ..*self }
//     }
// }

// The state of the oscillator
#[derive(Debug, Clone, Copy)]
pub struct OscState {
    last_cycle: u32,
    // freq: f32,
}

impl OscState {
    // fn modulate_freq(&mut self, f: impl Fn(f32) -> f32) {
    //     self.freq = f(self.freq);
    // }

    // fn phase(&mut self, clock: &Clock) -> f32 {
    //     clock.phase(self.freq, &mut self.last_tick)
    // }
}

// TODO: Pitch + Fine pitch, Syncing between oscillators (now synced through reset call), osc mix
pub struct OscPack<O: Osc, const OSCS: usize> {
    oscs: [O; OSCS],
    states: [OscState; OSCS],
}

impl<O: Osc + 'static, const OSCS: usize> OscPack<O, OSCS> {
    pub fn new(osc: impl Fn(usize) -> O) -> Self {
        Self {
            oscs: core::array::from_fn(osc),
            states: core::array::from_fn(|_| OscState {
                last_cycle: 0,
                // freq: 0.0,
            }),
        }
    }

    // fn reset(&mut self) {
    //     // TODO: Sync modes
    //     self.states.iter_mut().for_each(|state| {
    //         // TODO: Is this right or we should reset to current tick?
    //         state.last_tick = 0;
    //     });
    // }

    // fn set_freq(&mut self, freq: f32) {
    //     self.states.iter_mut().for_each(|state| {
    //         state.freq = freq;
    //     });
    // }

    // pub fn note_on(&mut self, freq: f32) {
    //     self.reset();
    //     self.set_freq(freq);
    // }

    pub fn tick<'a>(
        &mut self,
        clock: &Clock,
        freq: f32,
        props: &[OscProps<'a, O>],
    ) -> Option<SignedUnitInterval> {
        let (_, output) = self
            .oscs
            .iter_mut()
            .zip(props)
            .zip(self.states.iter_mut())
            .fold(
                (
                    OscMod::Direct(SignedUnitInterval::EQUILIBRIUM),
                    SignedUnitInterval::EQUILIBRIUM,
                ),
                |(modulation, prev_output), ((osc, props), state)| {
                    let freq = match modulation {
                        OscMod::FM(fm) => modulate_freq(freq, fm.inner()),
                        _ => freq,
                    };

                    let phase = clock.phase(freq, &mut state.last_cycle);

                    let output: SignedUnitInterval = osc.tick(phase, &props.osc);

                    let output = match modulation {
                        OscMod::AM(am) => output * am,
                        OscMod::RM(rm) => output * rm,
                        _ => output,
                    };

                    (
                        match props.output {
                            OscOutput::Direct => OscMod::Direct(output),
                            OscOutput::FMNext => OscMod::FM(output),
                            OscOutput::AMNext => OscMod::AM(output.remap_into_unsigned()),
                            OscOutput::RMNext => OscMod::RM(output),
                        },
                        prev_output + output,
                    )
                },
            );

        Some(output)
    }
}
