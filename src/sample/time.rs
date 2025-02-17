use core::ops::{Add, Mul};

use crate::osc::clock::Clock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SampleCount(u32);

impl PartialOrd<u32> for SampleCount {
    fn partial_cmp(&self, other: &u32) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<u32> for SampleCount {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl From<u32> for SampleCount {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl SampleCount {
    pub const ZERO: Self = Self::new(0);
    // pub const MAX: Self = Self::new(u32::MAX);
    // pub const SECOND: Self = Self::new(SAMPLE_RATE);
    // pub const MILLISECOND: Self = Self::new(SAMPLE_RATE / 1_000);

    pub const fn new(count: u32) -> Self {
        Self(count)
    }

    pub const fn inner(self) -> u32 {
        self.0
    }

    pub const fn zero() -> Self {
        Self::new(0)
    }

    pub const fn max() -> Self {
        Self::new(u32::MAX)
    }

    pub const fn second(sample_rate: u32) -> Self {
        Self::new(sample_rate)
    }

    pub const fn millisecond(sample_rate: u32) -> Self {
        Self::new(sample_rate / 1_000)
    }

    /// Create SampleCount from seconds
    pub const fn from_seconds(seconds: u32, sample_rate: u32) -> Self {
        Self::new(Self::second(sample_rate).0 * seconds)
    }

    /// Create SampleCount from milliseconds rounding to ceiling
    pub const fn from_millis(millis: u32, sample_rate: u32) -> Self {
        Self::new(Self::millisecond(sample_rate).inner() * millis)
        // Self::new((SAMPLE_RATE * millis).div_ceil(1_000))
    }

    pub const fn from_millis_f32(millis: f32, sample_rate: u32) -> Self {
        Self::new((sample_rate as f32 / 1_000.0 * millis) as u32)
    }

    /// Get milliseconds from sample count rounding to ceiling
    pub const fn seconds(self, sample_rate: u32) -> u32 {
        // self.0.div_ceil(SAMPLE_RATE)
        self.0 / sample_rate
    }

    /// Get milliseconds from sample count rounding to ceiling
    pub const fn millis(self, sample_rate: u32) -> u32 {
        // (self.0 * 1_000).div_ceil(SAMPLE_RATE)
        self.0 * 1_000 / sample_rate
    }

    pub const fn millis_f32(self, sample_rate: u32) -> f32 {
        self.0 as f32 * 1_000.0 / sample_rate as f32
    }

    pub const fn is_zero(&self) -> bool {
        self.inner() == 0
    }
}

impl Add for SampleCount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0 + rhs.0)
    }
}

impl Mul<u32> for SampleCount {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self::new(self.inner() * rhs)
    }
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn constants() {
    //     const SR: u32 = 48_000;
    //     type SC = SampleCount<SR>;
    //     assert_eq!(SC::ZERO.inner(), 0);
    //     // assert_eq!(SC::MICROSECOND.inner(), 21);
    //     assert_eq!(SC::MILLISECOND.inner(), 48);
    //     assert_eq!(SC::SECOND.inner(), SR);
    // }
}
