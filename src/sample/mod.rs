use crate::param::f32::SignedUnitInterval;
use core::{
    iter::Sum,
    ops::{Add, Sub},
};

pub mod time;

pub trait Sample: Copy + Add<Self, Output = Self> + Sub<Self, Output = Self> + Sized + Sum {
    fn lerp(self, to: Self, num: u32, denom: u32) -> Self;
    fn saturating_add(self, other: Self) -> Self;
    // fn to_f32(self) -> f32;
    fn to_sui(self) -> SignedUnitInterval;
    fn zero() -> Self;
    fn amp(self, amp: f32) -> Self;
    fn max(self, other: Self) -> Self;
    fn fold_mean(self, prev_mean: Self, index: usize) -> Self;

    // fn lerp_time(self, to: Self, time_point: TimePoint) -> Self
    // where
    //     Self: Sized,
    // {
    //     // let to_factor = time_point.frac().
    // }
}

impl Sample for u16 {
    fn lerp(self, to: Self, num: u32, denom: u32) -> Self {
        let from = self as i32;

        (from + (((to as i32) - from) * (num as i32)) / (denom as i32)) as u16
    }

    fn saturating_add(self, other: Self) -> Self {
        self.saturating_add(other)
    }

    // fn to_f32(self) -> f32 {
    //     ((self as f32) - (Self::MAX as f32)) / (Self::MAX as f32)
    // }

    fn to_sui(self) -> SignedUnitInterval {
        SignedUnitInterval::new_checked((self as f32 - Self::MAX as f32) / Self::MAX as f32)
    }

    fn zero() -> Self {
        0
    }

    fn amp(self, amp: f32) -> Self {
        // TODO: Is this right or amp can be negative and greater than 1.0?
        debug_assert!(amp >= 0.0 && amp <= 1.0);
        (self as f32 * amp / Self::MAX as f32) as Self
    }

    fn max(self, other: Self) -> Self {
        Ord::max(self, other)
    }

    fn fold_mean(self, prev_mean: Self, index: usize) -> Self {
        // FIXME: subtract overflow issue
        prev_mean + (self - prev_mean) / (index as u16 + 1)
    }
}

impl Sample for f32 {
    fn lerp(self, to: Self, num: u32, denom: u32) -> Self {
        self + ((to - self) * (num as f32)) / (denom as f32)
    }

    fn saturating_add(self, other: Self) -> Self {
        // TODO: Clip or not to clip?
        self + other
    }

    // fn to_f32(self) -> f32 {
    //     self
    // }

    fn to_sui(self) -> SignedUnitInterval {
        SignedUnitInterval::new(self)
    }

    fn zero() -> Self {
        0.0
    }

    fn amp(self, amp: f32) -> Self {
        self * amp
    }

    fn max(self, other: Self) -> Self {
        f32::max(self, other)
    }

    fn fold_mean(self, prev_mean: Self, index: usize) -> Self {
        prev_mean + (self - prev_mean) / (index as f32 + 1.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::sample::Sample;

    #[test]
    fn mean_identity() {
        let samples = [0.2f32, 0.3, 0.4, 0.5];

        assert!(
            samples
                .iter()
                .copied()
                .enumerate()
                .fold(0.0, |mean, (index, sample)| {
                    sample.fold_mean(mean, index)
                })
                - (samples.iter().sum::<f32>() / samples.len() as f32)
                < f32::EPSILON
        );
    }
}
