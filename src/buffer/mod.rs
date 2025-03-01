pub trait RingIndex {
    type Output;

    fn ring_index(&self, index: usize) -> &Self::Output;
}

impl<T, const SIZE: usize> RingIndex for [T; SIZE] {
    type Output = T;

    fn ring_index(&self, index: usize) -> &Self::Output {
        &self[index % SIZE]
    }
}

impl<T> RingIndex for &[T] {
    type Output = T;

    fn ring_index(&self, index: usize) -> &Self::Output {
        &self[index % self.len()]
    }
}

pub trait RingIndexMut {
    type Output;

    fn ring_index_mut(&mut self, index: usize) -> &mut Self::Output;
}

impl<T, const SIZE: usize> RingIndexMut for [T; SIZE] {
    type Output = T;

    fn ring_index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self[index % SIZE]
    }
}

impl<T> RingIndexMut for &mut [T] {
    type Output = T;

    fn ring_index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self[index % self.len()]
    }
}
