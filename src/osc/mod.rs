use crate::{sample::Sample, source::Source};

pub trait Osc: Source + Clone
where
    <Self as Iterator>::Item: Sample,
{
    fn set_freq(&mut self, freq: f32) -> &mut Self;
    fn freq(&self) -> f32;

    /// Reset only dynamic states. I.e.
    fn reset(&mut self) -> &mut Self;

    /// Clone and reset
    fn replicate(&self) -> Self {
        let mut replica = self.clone();
        replica.reset();
        replica
    }
}
