use alloc::{
    format,
    string::{String, ToString},
};
use core::fmt::Display;

use crate::value::freq::{Freq, Millihertz};

pub mod f32;
pub mod int;
pub mod select;
pub mod ui;

// pub enum ParamValueKind {
//     /// f32 from 0.0 to 1.0 inclusive
//     UnitInterval,
//     /// f32 from 0.0 to 0.5 inclusive
//     HalfUnitInterval,
//     /// All u8 values ranging from 0 to 255
//     U8,
//     /// All i8 values ranging from -128 to 127
//     I8,
// }

// pub trait ParamType {
//     const MIN: Self;
//     const MAX: Self;

//     // fn new(value: Self::Inner) -> Self;

//     fn offset(&self, offset: i32) -> Self;

//     // TODO: Logarithmic?
// }

// pub trait ParamType {
//     fn as_value(&self) -> ParamValue;
//     fn set_value(&mut self, value: ParamValue);

//     fn format(&self) -> ParamFormat {
//         ParamFormat::Auto
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
// pub enum ParamValue {
//     UnitInterval { value: f32 },
//     HalfUnitInterval { value: f32 },
//     U8 { value: u8 },
//     U32 { value: u32 },
//     Freq(Millihertz),
// }

// impl Display for ParamValue {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         match self {
//             ParamValue::UnitInterval { value } => write!(f, "{value}"),
//             ParamValue::HalfUnitInterval { value } => write!(f, "{value}"),
//             ParamValue::U8 { value } => write!(f, "{value}"),
//             ParamValue::U32 { value } => write!(f, "{value}"),
//             ParamValue::Freq(freq) => write!(f, "{freq:.2}"),
//         }
//     }
// }

// impl ParamType for ParamValue {
//     fn as_value(&self) -> ParamValue {
//         *self
//     }

//     fn set_value(&mut self, value: ParamValue) {
//         *self = value
//     }
// }

// impl ParamValue {
//     pub fn as_unit_interval(&self) -> f32 {
//         match self {
//             &ParamValue::UnitInterval { value, .. } => value,
//             &ParamValue::HalfUnitInterval { value, .. } => value * 2.0,
//             &ParamValue::U8 { value, .. } => value as f32 / u8::MAX as f32,
//             &ParamValue::U32 { value, .. } => value as f32 / u32::MAX as f32,
//             ParamValue::Freq(freq) => todo!(),
//         }
//     }

//     pub fn as_half_unit_interval(&self) -> f32 {
//         match self {
//             ParamValue::UnitInterval { value } => value / 2.0,
//             &ParamValue::HalfUnitInterval { value } => value,
//             &ParamValue::U8 { value } => value as f32 / (255.0 * 2.0),
//             &ParamValue::U32 { value } => value as f32 / (u32::MAX as f32 * 2.0),
//             ParamValue::Freq(freq) => todo!(),
//         }
//     }

//     pub fn as_u8_range(&self) -> u8 {
//         match self {
//             ParamValue::UnitInterval { value, .. } => (value * u8::MAX as f32) as u8,
//             ParamValue::HalfUnitInterval { value, .. } => (value * u8::MAX as f32 * 2.0) as u8,
//             &ParamValue::U8 { value, .. } => value,
//             &ParamValue::U32 { value, .. } => {
//                 (value as u64 * u8::MAX as u64 / u32::MAX as u64) as u8
//             }
//             ParamValue::Freq(freq) => todo!(),
//         }
//     }

//     pub fn as_u32_range(&self) -> u32 {
//         match self {
//             ParamValue::UnitInterval { value, .. } => (value * u32::MAX as f32) as u32,
//             ParamValue::HalfUnitInterval { value, .. } => (value * u32::MAX as f32 * 2.0) as u32,
//             &ParamValue::U8 { value, .. } => value as u32 * (u32::MAX / u8::MAX as u32),
//             &ParamValue::U32 { value, .. } => value,
//             ParamValue::Freq(freq) => todo!(),
//         }
//     }

//     pub fn as_freq(&self) -> Freq {
//         match self {
//             ParamValue::UnitInterval { value } => todo!(),
//             ParamValue::HalfUnitInterval { value } => todo!(),
//             ParamValue::U8 { value } => todo!(),
//             ParamValue::U32 { value } => todo!(),
//             &ParamValue::Freq(freq) => freq,
//         }
//     }

//     pub fn clamp(&self, min: Self, max: Self) -> Self {
//         match self {
//             &ParamValue::UnitInterval { value } => {
//                 let (min, max) = (min.as_unit_interval(), max.as_unit_interval());

//                 Self::UnitInterval {
//                     value: value.clamp(min, max),
//                 }
//             }
//             &ParamValue::HalfUnitInterval { value } => {
//                 let (min, max) = (min.as_half_unit_interval(), max.as_half_unit_interval());

//                 Self::HalfUnitInterval {
//                     value: value.clamp(min, max),
//                 }
//             }
//             &ParamValue::U8 { value } => {
//                 let (min, max) = (min.as_u8_range(), max.as_u8_range());

//                 Self::U8 {
//                     value: value.clamp(min, max),
//                 }
//             }
//             &ParamValue::U32 { value } => {
//                 let (min, max) = (min.as_u32_range(), max.as_u32_range());

//                 Self::U32 {
//                     value: value.clamp(min, max),
//                 }
//             }
//             &ParamValue::Freq(freq) => {
//                 let (min, max) = (min.as_freq(), max.as_freq());

//                 Self::Freq(freq.clamp(min, max))
//             }
//         }
//     }
// }

// #[derive(Debug, Clone, Copy, Default)]
// pub enum ParamFormat {
//     #[default]
//     Auto,
//     UnitInterval,
//     /// Time defined in sample count with sample rate
//     TimeInSamples(u32),
//     // TODO: Time
// }

// /// Generic parameter
// pub struct Param<'a> {
//     name: &'static str,
//     value: &'a mut dyn ParamType,
//     // TODO: Do we need clamp here and in UnitInterval/HalfUnitInterval, etc.?
//     clamp: Option<(ParamValue, ParamValue)>,
//     logarithmic: bool,
//     format: ParamFormat,
// }

// impl<'a> Param<'a> {
//     pub fn new(name: &'static str, value: &'a mut impl ParamType) -> Self {
//         let format = value.format();

//         Self {
//             name,
//             value,
//             clamp: None,
//             logarithmic: false,
//             format,
//         }
//     }

//     pub fn clamped(mut self, min: impl ParamType, max: impl ParamType) -> Self {
//         self.clamp = Some((min.as_value(), max.as_value()));
//         self
//     }

//     pub fn logarithmic(mut self, logarithmic: bool) -> Self {
//         self.logarithmic = logarithmic;
//         self
//     }

//     pub fn with_format(mut self, format: ParamFormat) -> Self {
//         self.format = format;
//         self
//     }

//     pub fn is_logarithmic(&self) -> bool {
//         self.logarithmic
//     }

//     pub fn clamp(&self) -> Option<(ParamValue, ParamValue)> {
//         self.clamp
//     }

//     pub fn name(&self) -> &'static str {
//         self.name
//     }

//     pub fn value(&self) -> ParamValue {
//         self.value.as_value()
//     }

//     pub fn format(&self) -> ParamFormat {
//         self.format
//     }

//     pub fn set(&mut self, value: impl ParamType) {
//         let value = value.as_value();
//         let value = if let Some((min, max)) = self.clamp {
//             value.clamp(min, max)
//         } else {
//             value
//         };
//         self.value.set_value(value);
//     }
// }
