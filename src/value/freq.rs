// use fugit::Rate;

// pub trait RateExt {
//     #[allow(non_snake_case)]
//     fn to_milliHz(self) -> u32;

//     #[allow(non_snake_case)]
//     fn to_Hz_f32(self) -> f32;

//     // #[allow(non_snake_case)]
//     // fn to_kHz_f32(&self) -> f32;

//     // #[allow(non_snake_case)]
//     // fn to_MHz_f32(&self) -> f32;
// }

// impl<const NOM: u32, const DENOM: u32> RateExt for Rate<u32, NOM, DENOM> {
//     fn to_milliHz(self) -> u32 {
//         fugit::
//     }
// }

// pub trait U32Ext {
//     #[allow(non_snake_case)]
//     fn milliHz(self) -> Millihertz;
// }

// impl U32Ext for u32 {
//     fn milliHz(self) -> Millihertz {
//         Millihertz::from_raw(self)
//     }
// }

// pub type Millihertz = Rate<u32, 1, 1_000>;
// pub type FreqBase = Millihertz;

// #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
// pub struct Freq(FreqBase);

// /// Frequency-indexed map
// pub struct FreqMap<T, const SIZE: usize> {
//     map: [Option<T>; SIZE],
// }

// #[cfg(test)]
// mod tests {
//     use crate::value::freq::RateExt;

//     use super::Millihertz;

//     #[test]
//     fn millihertz_conv() {
//         let mh = Millihertz::from_raw(1);

//         assert_eq!(mh.to_milliHz(), 1_000);
//     }
// }

// pub type MillihertzInner = fixed::types::U24F8;

// #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
// pub struct Millihertz(MillihertzInner);

// impl Millihertz {
//     pub fn new(value: MillihertzInner) -> Self {
//         Self(value)
//     }

//     pub fn inner(&self) -> MillihertzInner {
//         self.0
//     }
// }

use crate::param::ParamType;

pub type Millihertz = fixed::types::U24F8;

/// For frequency millihertz are used as 32-bit unsigned fixed point number with 8 bit fractional part, so it is 256 point fractional part and >16.7MHz for integer part which is enough for audio. I chose to use fractional hertz type so this frequency type is general for everything, of course we don't need to produce such low-frequency audio data, it is inaudible, but we need these low frequencies in LFOs.
/// The smallest frequency after zero is 0.0039Hz (once every 256s or 4 minutes)
pub type Freq = Millihertz;

impl ParamType for Freq {
    fn as_value(&self) -> crate::param::ParamValue {
        crate::param::ParamValue::Freq(*self)
    }

    fn set_value(&mut self, value: crate::param::ParamValue) {
        *self = value.as_freq();
    }
}
