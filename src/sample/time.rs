use core::{
    fmt::Display,
    ops::{Add, Mul},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SampleCount<const SAMPLE_RATE: u32>(u32);

impl<const SAMPLE_RATE: u32> Display for SampleCount<SAMPLE_RATE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.0 == 0 {
            write!(f, "0")
        } else if self.millis() == 0 {
            write!(f, "{}t", self.inner())
        } else if self < &Self::SECOND {
            write!(f, "{}ms", self.millis())
        } else {
            write!(f, "{}s", self.seconds())
        }
    }
}

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
    // pub const MICROSECOND: Self = Self::new(SAMPLE_RATE.div_ceil(1_000_000));
    // pub const NANOSECOND: Self = Self::from_nanos(1);

    pub const fn new(count: u32) -> Self {
        Self(count)
    }

    pub const fn inner(self) -> u32 {
        self.0
    }

    // pub const fn conv<const OTHER_SAMPLE_RATE: u32>(self) -> SampleCount<OTHER_SAMPLE_RATE> {
    //     SampleCount::new(self.0 * SAMPLE_RATE / OTHER_SAMPLE_RATE)
    // }

    // /// Create SampleCount from floating point seconds value with precision of 1us. Maximum is 4294s
    // pub const fn from_seconds_f32(seconds: f32) -> Self {
    //     Self::from_micros((seconds * 1_000_000.0) as u32)
    // }

    /// Create SampleCount from seconds
    pub const fn from_seconds(seconds: u32) -> Self {
        Self::new(Self::SECOND.inner() * seconds)
    }

    /// Create SampleCount from milliseconds rounding to ceiling
    pub const fn from_millis(millis: u32) -> Self {
        Self::new(Self::MILLISECOND.inner() * millis)
        // Self::new((SAMPLE_RATE * millis).div_ceil(1_000))
    }

    // /// Create SampleCount from microseconds rounding to ceiling
    // pub const fn from_micros(micros: u32) -> Self {
    //     Self::new(Self::MICROSECOND.inner() * micros)
    //     // Self::new((SAMPLE_RATE * micros).div_ceil(1_000_000))
    // }

    // /// Create SampleCount from nanoseconds rounding to ceiling
    // pub const fn from_nanos(nanos: u32) -> Self {
    //     Self::new(Self::NANOSECOND)
    // }

    // /// Get floating point seconds from sample count with precision of 1us
    // pub const fn seconds_f32(self) -> f32 {
    //     self.micros() as f32 / 1_000_000.0
    // }

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

    // /// Get microseconds from sample count rounding to ceiling
    // pub const fn micros(self) -> u32 {
    //     (self.0 * 1_000_000).div_ceil(SAMPLE_RATE)
    // }

    // /// Get nanoseconds from sample count rounding to ceiling
    // pub const fn nanos(self) -> u32 {
    //     (self.0 as u64 * 1_000_000_000).div_ceil(SAMPLE_RATE as u64) as u32
    // }
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
