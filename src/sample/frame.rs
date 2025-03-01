use core::{
    iter::Sum,
    ops::{Add, Div, Index, Mul},
};

use crate::param::f32::{SignedUnitInterval, UnitInterval};

use super::Sample;

#[derive(Clone, Copy)]
pub struct Frame<T = f32, const SIZE: usize = 2> {
    channels: [T; SIZE],
}

impl<T> From<(T, T)> for Frame<T> {
    fn from(value: (T, T)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl<T> Frame<T, 2> {
    pub fn new(left: T, right: T) -> Self {
        Self {
            channels: [left, right],
        }
    }

    pub fn swapped(self) -> Self {
        let [left, right] = self.channels;
        Self::new(right, left)
    }

    pub fn left(&self) -> &T {
        &self.channels[0]
    }

    pub fn right(&self) -> &T {
        &self.channels[1]
    }

    pub fn left_mut(&mut self) -> &mut T {
        &mut self.channels[0]
    }

    pub fn right_mut(&mut self) -> &mut T {
        &mut self.channels[1]
    }
}

impl Frame<f32, 2> {
    pub fn balanced(&self, balance: UnitInterval) -> Self {
        Self {
            channels: [
                self.channels[0] * (1.0 - balance.inner()),
                self.channels[1] * balance.inner(),
            ],
        }
    }
}

impl<T: Copy, const SIZE: usize> Frame<[T; SIZE], 2> {
    pub fn at(&self, index: usize) -> Frame<T, 2> {
        Frame::new(self.left()[index], self.right()[index])
    }

    pub fn set(&mut self, index: usize, frame: Frame<T, 2>) {
        let [left, right] = frame.channels;
        self.left_mut()[index] = left;
        self.right_mut()[index] = right;
    }
}

impl<T: Mul<Output = T> + Copy, const SIZE: usize> Mul for Frame<T, SIZE> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.zip(rhs, |lhs, rhs| *lhs * *rhs)
    }
}

impl<T: Mul<Output = T> + Copy, const SIZE: usize> Mul<T> for Frame<T, SIZE> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        self.map(|val| *val * rhs)
    }
}

impl<T: Div<Output = T> + Copy, const SIZE: usize> Div for Frame<T, SIZE> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        self.zip(rhs, |lhs, rhs| *lhs / *rhs)
    }
}

impl<T: Add<Output = T> + Copy, const SIZE: usize> Add for Frame<T, SIZE> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.zip(rhs, |lhs, rhs| *lhs + *rhs)
    }
}

impl<T: Sample, const SIZE: usize> IntoIterator for Frame<T, SIZE> {
    type Item = T;

    type IntoIter = core::array::IntoIter<T, SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        self.channels.into_iter()
    }
}

impl<T: Sample, const SIZE: usize> Sum for Frame<T, SIZE> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |sum, frame| sum + frame)
    }
}

impl<T: Sample, const SIZE: usize> Frame<T, SIZE> {
    pub fn zero() -> Self {
        Self::equal(T::zero())
    }
}

impl<T, const SIZE: usize> Frame<T, SIZE> {
    pub fn from_fn(f: impl FnMut(usize) -> T) -> Self {
        Self {
            channels: core::array::from_fn(f),
        }
    }

    pub fn equal(value: T) -> Self
    where
        T: Copy,
    {
        Self::from_fn(|_| value)
    }

    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> Frame<U, SIZE> {
        Frame::from_fn(|index| f(&self.channels[index]))
    }

    pub fn zip<U, O>(&self, other: Frame<U, SIZE>, f: impl Fn(&T, &U) -> O) -> Frame<O, SIZE> {
        Frame::from_fn(|index| f(&self.channels[index], &other.channels[index]))
    }

    pub fn zip_mut<U, O>(
        &mut self,
        other: &mut Frame<U, SIZE>,
        mut f: impl FnMut(&mut T, &mut U) -> O,
    ) -> Frame<O, SIZE> {
        Frame::from_fn(|index| f(&mut self.channels[index], &mut other.channels[index]))
    }
}
