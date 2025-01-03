// use core::{ marker::PhantomData, ops::Index };

// use crate::{ sample::Sample, value::freq::Freq };

// /// Collection of global consts and types
// pub trait Ctx: Sized + 'static {
//     const SAMPLE_RATE: usize;
//     const WAVETABLE_DEPTH: usize;
//     const WAVETABLE_LENGTH: usize;

//     type Sample: Sample;

//     // Derived types to use generics in const expression //
//     type WavetableRowInner: Index<usize, Output = Self::Sample>;
//     type WavetableInner: for<'a> Index<usize, Output = (Freq, Self::WavetableRowInner)>;
// }

// pub struct CtxInstance<
//     S: Sample,
//     const SAMPLE_RATE: usize,
//     const WAVETABLE_DEPTH: usize,
//     const WAVETABLE_LENGTH: usize
// > {
//     sample: PhantomData<S>,
// }

// impl<
//     S: Sample,
//     const SAMPLE_RATE: usize,
//     const WAVETABLE_DEPTH: usize,
//     const WAVETABLE_LENGTH: usize
// > CtxInstance<S, SAMPLE_RATE, WAVETABLE_DEPTH, WAVETABLE_LENGTH> {
//     pub fn new() -> Self {
//         Self {
//             sample: PhantomData,
//         }
//     }

//     pub fn sample<NewSample: Sample>() -> CtxInstance<
//         NewSample,
//         SAMPLE_RATE,
//         WAVETABLE_DEPTH,
//         WAVETABLE_LENGTH
//     > {
//         CtxInstance { sample: PhantomData }
//     }

//     pub fn sample_rate<const NEW_SAMPLE_RATE: usize>(
//         self
//     ) -> CtxInstance<S, NEW_SAMPLE_RATE, WAVETABLE_DEPTH, WAVETABLE_LENGTH> {
//         CtxInstance {
//             sample: PhantomData,
//         }
//     }

//     pub fn wavetable_depth<const NEW_WAVETABLE_DEPTH: usize>(
//         self
//     ) -> CtxInstance<S, SAMPLE_RATE, NEW_WAVETABLE_DEPTH, WAVETABLE_LENGTH> {
//         CtxInstance {
//             sample: PhantomData,
//         }
//     }

//     pub fn wavetable_length<const NEW_WAVETABLE_LENGTH: usize>(
//         self
//     ) -> CtxInstance<S, SAMPLE_RATE, WAVETABLE_DEPTH, NEW_WAVETABLE_LENGTH> {
//         CtxInstance {
//             sample: PhantomData,
//         }
//     }
// }

// impl<
//     S: Sample + 'static,
//     const SAMPLE_RATE: usize,
//     const WAVETABLE_DEPTH: usize,
//     const WAVETABLE_LENGTH: usize
// > Ctx for CtxInstance<S, SAMPLE_RATE, WAVETABLE_DEPTH, WAVETABLE_LENGTH> {
//     type Sample = S;

//     const SAMPLE_RATE: usize = SAMPLE_RATE;
//     const WAVETABLE_DEPTH: usize = WAVETABLE_DEPTH;
//     const WAVETABLE_LENGTH: usize = WAVETABLE_LENGTH;

//     type WavetableRowInner = [Self::Sample; WAVETABLE_LENGTH];
//     type WavetableInner = [(Freq, Self::WavetableRowInner); WAVETABLE_DEPTH];
// }
