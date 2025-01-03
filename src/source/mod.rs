use crate::sample::Sample;

pub trait Source: Iterator
where
    Self::Item: Sample,
{
}
