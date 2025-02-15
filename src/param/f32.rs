use core::{
    fmt::Display,
    iter::Sum,
    ops::{Add, Div, Mul, RangeInclusive},
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct UnitInterval(f32);

impl UnitInterval {
    pub const RANGE: RangeInclusive<f32> = 0.0..=1.0;
    pub const MIN: Self = Self(0.0);
    pub const EQUILIBRIUM: Self = Self(0.5);
    pub const MAX: Self = Self(1.0);

    pub fn new(value: f32) -> Self {
        Self(value.clamp(*Self::RANGE.start(), *Self::RANGE.end()))
    }

    /// Construct new [`UnitInterval`] panicking if value is not lying in its range
    pub fn new_checked(value: f32) -> Self {
        debug_assert!(Self::RANGE.contains(&value));
        Self(value)
    }

    // /// Get factor to be multiplied by, i.e. for [`UnitInterval`] 0.0 it is 1.0, for 1.0 it is
    // pub fn into_factor(&self) -> f32 {

    // }

    #[inline]
    pub fn inner(&self) -> f32 {
        self.0
    }

    pub fn remap_into_signed(&self) -> SignedUnitInterval {
        SignedUnitInterval::new(self.0 * 2.0 - 1.0)
    }
}

impl Display for UnitInterval {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.inner())
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

impl Display for HalfUnitInterval {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.inner())
    }
}

impl HalfUnitInterval {
    pub const RANGE: RangeInclusive<f32> = 0.0..=0.5;
    pub const MIN: Self = Self(0.0);
    pub const EQUILIBRIUM: Self = Self(0.25);
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SignedUnitInterval(f32);

impl Display for SignedUnitInterval {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner().fmt(f)
    }
}

impl Sum for SignedUnitInterval {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self::new(iter.map(|el| el.inner()).sum())
    }
}

impl Mul for SignedUnitInterval {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.0 * rhs.0)
    }
}

impl Mul<f32> for SignedUnitInterval {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.0 * rhs)
    }
}

impl Div<f32> for SignedUnitInterval {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.0 / rhs)
    }
}

impl Mul<UnitInterval> for SignedUnitInterval {
    type Output = Self;

    fn mul(self, rhs: UnitInterval) -> Self::Output {
        Self::new(self.0 * rhs.0)
    }
}

impl Add for SignedUnitInterval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0 + rhs.0)
    }
}

impl SignedUnitInterval {
    pub const RANGE: RangeInclusive<f32> = -1.0..=1.0;
    pub const MIN: Self = Self(-1.0);
    pub const EQUILIBRIUM: Self = Self(0.0);
    pub const MAX: Self = Self(1.0);

    pub fn new(value: f32) -> Self {
        Self(value.clamp(*Self::RANGE.start(), *Self::RANGE.end()))
    }

    /// Construct new [`SignedUnitInterval`] panicking if value is not lying in its range
    pub fn new_checked(value: f32) -> Self {
        debug_assert!(Self::RANGE.contains(&value));
        Self(value)
    }

    #[inline]
    pub fn inner(&self) -> f32 {
        self.0
    }

    pub fn remap_into_unsigned(&self) -> UnitInterval {
        UnitInterval::new((self.0 + 1.0) / 2.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::param::f32::SignedUnitInterval;

    use super::UnitInterval;

    #[test]
    fn remapping() {
        assert_eq!(
            UnitInterval::MIN.remap_into_signed(),
            SignedUnitInterval::MIN
        );

        assert_eq!(
            UnitInterval::MAX.remap_into_signed(),
            SignedUnitInterval::MAX
        );

        assert_eq!(
            UnitInterval::EQUILIBRIUM.remap_into_signed(),
            SignedUnitInterval::EQUILIBRIUM
        );

        assert_eq!(
            SignedUnitInterval::MIN.remap_into_unsigned(),
            UnitInterval::MIN,
        );

        assert_eq!(
            SignedUnitInterval::MAX.remap_into_unsigned(),
            UnitInterval::MAX,
        );

        assert_eq!(
            SignedUnitInterval::EQUILIBRIUM.remap_into_unsigned(),
            UnitInterval::EQUILIBRIUM,
        );
    }
}
