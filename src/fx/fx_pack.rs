use super::Fx;

pub struct FxPack<const SIZE: usize> {
    effects: [Option<alloc::boxed::Box<dyn Fx>>; SIZE],
}

impl<const SIZE: usize> Fx for FxPack<SIZE> {
    fn tick(
        &mut self,
        input: crate::sample::Frame,
        clock: &crate::osc::clock::Clock,
    ) -> crate::sample::Frame {
        todo!()
    }
}

impl<const SIZE: usize> FxPack<SIZE> {
    pub fn new() -> Self {
        Self {
            effects: core::array::from_fn(|_| None),
        }
    }
}
