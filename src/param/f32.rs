use super::{ParamType, ParamValue};
use core::ops::RangeInclusive;

// TODO: Generalize?
//  -- No, remove at all. ParamType should not denote ranges. It is the role of Param. Yes?

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct UnitInterval(f32);

impl UnitInterval {
    pub const RANGE: RangeInclusive<f32> = 0.0..=1.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(*Self::RANGE.start(), *Self::RANGE.end()))
    }

    pub fn range(clamp: Option<(ParamValue, ParamValue)>) -> RangeInclusive<f32> {
        clamp
            .map(|clamp| clamp.0.as_unit_interval()..=clamp.1.as_unit_interval())
            .unwrap_or(Self::RANGE)
    }

    // Only for egui
    pub fn range_f64(clamp: Option<(ParamValue, ParamValue)>) -> RangeInclusive<f64> {
        clamp
            .map(|clamp| clamp.0.as_unit_interval() as f64..=clamp.1.as_unit_interval() as f64)
            .unwrap_or(*Self::RANGE.start() as f64..=*Self::RANGE.end() as f64)
    }

    pub fn inner(&self) -> f32 {
        self.0
    }
}

impl ParamType for UnitInterval {
    fn as_value(&self) -> super::ParamValue {
        super::ParamValue::UnitInterval { value: self.0 }
    }

    fn set_value(&mut self, value: super::ParamValue) {
        self.0 = value.as_unit_interval();
    }

    fn format(&self) -> super::ParamFormat {
        super::ParamFormat::UnitInterval
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct HalfUnitInterval(f32);

impl HalfUnitInterval {
    pub const RANGE: RangeInclusive<f32> = 0.0..=0.5;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(*Self::RANGE.start(), *Self::RANGE.end()))
    }

    pub fn range(clamp: Option<(ParamValue, ParamValue)>) -> RangeInclusive<f32> {
        clamp
            .map(|clamp| clamp.0.as_half_unit_interval()..=clamp.1.as_half_unit_interval())
            .unwrap_or(Self::RANGE)
    }

    pub fn range_f64(clamp: Option<(ParamValue, ParamValue)>) -> RangeInclusive<f64> {
        clamp
            .map(|clamp| {
                clamp.0.as_half_unit_interval() as f64..=clamp.1.as_half_unit_interval() as f64
            })
            .unwrap_or(*Self::RANGE.start() as f64..=*Self::RANGE.end() as f64)
    }

    pub fn inner(&self) -> f32 {
        self.0
    }
}

impl ParamType for HalfUnitInterval {
    fn as_value(&self) -> super::ParamValue {
        super::ParamValue::HalfUnitInterval { value: self.0 }
    }

    fn set_value(&mut self, value: super::ParamValue) {
        self.0 = value.as_half_unit_interval();
    }
}

// impl<const STEP_DENOM: usize> UnitInterval<STEP_DENOM> {
//     pub const STEP: f32 = 1.0 / STEP_DENOM as f32;

//     fn new(value: f32) -> Self {
//         Self(value.clamp(Self::MIN.0, Self::MAX.0))
//     }
// }

// impl<const STEP_DENOM: usize> ParamType for UnitInterval<STEP_DENOM> {
//     const MIN: Self = Self(0.0);
//     const MAX: Self = Self(1.0);

//     fn offset(&self, offset: i32) -> Self {
//         let value = self.0 + offset as f32 * Self::STEP;
//         Self::new(value)
//     }
// }

// // TODO
// // pub struct HalfUnitInterval(f32);

// #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
// pub struct SignedUnitInterval<const STEP_DENOM: usize>(f32);

// impl<const STEP_DENOM: usize> SignedUnitInterval<STEP_DENOM> {
//     pub const STEP: f32 = 1.0 / STEP_DENOM as f32;

//     fn new(value: f32) -> Self {
//         Self(value.clamp(Self::MIN.0, Self::MAX.0))
//     }
// }

// impl<const STEP_DENOM: usize> ParamType for SignedUnitInterval<STEP_DENOM> {
//     const MIN: Self = Self(-1.0);

//     const MAX: Self = Self(1.0);

//     fn offset(&self, offset: i32) -> Self {
//         let value = self.0 + offset as f32 * Self::STEP;
//         Self::new(value)
//     }
// }
