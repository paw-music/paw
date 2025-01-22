use core::ops::{Add, Mul, RangeInclusive, Sub};

// TODO: Generalize?
//  -- No, remove at all. ParamType should not denote ranges. It is the role of Param. Yes?

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct UnitInterval(f32);

// impl Add<f32> for UnitInterval {
//     type Output = f32;

//     fn add(self, rhs: f32) -> Self::Output {
//         self.0 + rhs
//     }
// }

// impl Add<UnitInterval> for f32 {
//     type Output = UnitInterval;

//     fn add(self, rhs: UnitInterval) -> Self::Output {
//         UnitInterval::new(self + rhs.0)
//     }
// }

// impl Sub<f32> for UnitInterval {
//     type Output = Self;

//     fn sub(self, rhs: f32) -> Self::Output {
//         Self::new(self.0 - rhs)
//     }
// }

// impl Sub<UnitInterval> for f32 {
//     type Output = UnitInterval;

//     fn sub(self, rhs: UnitInterval) -> Self::Output {
//         UnitInterval::new(self - rhs.0)
//     }
// }

// impl Mul<f32> for UnitInterval {
//     type Output = Self;

//     fn mul(self, rhs: f32) -> Self::Output {
//         Self::new(self.0 * rhs)
//     }
// }

// impl Mul<UnitInterval> for f32 {
//     type Output = UnitInterval;

//     fn mul(self, rhs: UnitInterval) -> Self::Output {
//         UnitInterval::new(self * rhs.0)
//     }
// }

impl UnitInterval {
    pub const RANGE: RangeInclusive<f32> = 0.0..=1.0;
    pub const ZERO: Self = Self(0.0);
    pub const MAX: Self = Self(1.0);

    pub fn new(value: f32) -> Self {
        Self(value.clamp(*Self::RANGE.start(), *Self::RANGE.end()))
    }

    #[inline]
    pub fn inner(&self) -> f32 {
        self.0
    }
}

impl PartialOrd<f32> for UnitInterval {
    fn partial_cmp(&self, other: &f32) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<f32> for UnitInterval {
    fn eq(&self, other: &f32) -> bool {
        self.0 == *other
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct HalfUnitInterval(f32);

impl HalfUnitInterval {
    pub const RANGE: RangeInclusive<f32> = 0.0..=0.5;
    pub const ZERO: Self = Self(0.0);
    pub const MAX: Self = Self(0.5);

    pub fn new(value: f32) -> Self {
        Self(value.clamp(*Self::RANGE.start(), *Self::RANGE.end()))
    }

    #[inline]
    pub fn inner(&self) -> f32 {
        self.0
    }
}

impl PartialOrd<f32> for HalfUnitInterval {
    fn partial_cmp(&self, other: &f32) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<f32> for HalfUnitInterval {
    fn eq(&self, other: &f32) -> bool {
        self.0 == *other
    }
}
