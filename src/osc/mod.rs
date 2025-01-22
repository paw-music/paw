use crate::{sample::Sample, source::Source, value::freq::Freq};

pub trait Osc: Source + Clone
where
    <Self as Iterator>::Item: Sample,
{
    fn set_freq(&mut self, freq: Freq) -> &mut Self;
    fn freq(&self) -> Freq;

    /// Reset only dynamic states. I.e.
    fn reset(&mut self) -> &mut Self;

    /// Clone and reset
    fn replicate(&self) -> Self {
        let mut replica = self.clone();
        replica.reset();
        replica
    }
}
