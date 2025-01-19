use crate::param::ParamType;
use core::{
    fmt::Display,
    ops::{Add, Mul},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SampleCount<const SAMPLE_RATE: u32>(u32);

impl<const SAMPLE_RATE: u32> PartialOrd<u32> for SampleCount<SAMPLE_RATE> {
    fn partial_cmp(&self, other: &u32) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<const SAMPLE_RATE: u32> PartialEq<u32> for SampleCount<SAMPLE_RATE> {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl<const SAMPLE_RATE: u32> From<u32> for SampleCount<SAMPLE_RATE> {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl<const SAMPLE_RATE: u32> SampleCount<SAMPLE_RATE> {
    pub const ZERO: Self = Self::new(0);
    pub const MAX: Self = Self::new(u32::MAX);
    pub const SECOND: Self = Self::new(SAMPLE_RATE);
    pub const MILLISECOND: Self = Self::new(SAMPLE_RATE / 1_000);

    pub const fn new(count: u32) -> Self {
        Self(count)
    }

    pub const fn inner(self) -> u32 {
        self.0
    }

    /// Create SampleCount from seconds
    pub const fn from_seconds(seconds: u32) -> Self {
        Self::new(Self::SECOND.inner() * seconds)
    }

    /// Create SampleCount from milliseconds rounding to ceiling
    pub const fn from_millis(millis: u32) -> Self {
        Self::new(Self::MILLISECOND.inner() * millis)
        // Self::new((SAMPLE_RATE * millis).div_ceil(1_000))
    }

    pub const fn from_millis_f32(millis: f32) -> Self {
        Self::new((SAMPLE_RATE as f32 / 1_000.0 * millis) as u32)
    }

    /// Get milliseconds from sample count rounding to ceiling
    pub const fn seconds(self) -> u32 {
        // self.0.div_ceil(SAMPLE_RATE)
        self.0 / SAMPLE_RATE
    }

    /// Get milliseconds from sample count rounding to ceiling
    pub const fn millis(self) -> u32 {
        // (self.0 * 1_000).div_ceil(SAMPLE_RATE)
        self.0 * 1_000 / SAMPLE_RATE
    }

    pub const fn millis_f32(self) -> f32 {
        self.0 as f32 * 1_000.0 / SAMPLE_RATE as f32
    }
}

impl<const SAMPLE_RATE: u32> Add for SampleCount<SAMPLE_RATE> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0 + rhs.0)
    }
}

impl<const SAMPLE_RATE: u32> Mul<u32> for SampleCount<SAMPLE_RATE> {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self::new(self.inner() * rhs)
    }
}

impl<const SAMPLE_RATE: u32> ParamType for SampleCount<SAMPLE_RATE> {
    /*
    Self::new(
        (self.0 as i64 + offset as i64).clamp(Self::ZERO.0 as i64, Self::MAX.0 as i64) as u32,
    )
     */

    fn as_value(&self) -> crate::param::ParamValue {
        crate::param::ParamValue::U32 { value: self.0 }
    }

    fn set_value(&mut self, value: crate::param::ParamValue) {
        self.0 = value.as_u32_range();
    }

    fn format(&self) -> crate::param::ParamFormat {
        crate::param::ParamFormat::TimeInSamples(SAMPLE_RATE)
    }
}

#[cfg(test)]
mod tests {
    use crate::sample::time::SampleCount;

    #[test]
    fn constants() {
        const SR: u32 = 48_000;
        type SC = SampleCount<SR>;
        assert_eq!(SC::ZERO.inner(), 0);
        // assert_eq!(SC::MICROSECOND.inner(), 21);
        assert_eq!(SC::MILLISECOND.inner(), 48);
        assert_eq!(SC::SECOND.inner(), SR);
    }
}
