use super::ParamType;
use num::Integer;

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// pub struct IntRange<T: Integer = u8>(T);

// impl ParamType for IntRange<u8> {
//     type Inner = u8;

//     const MIN: Self = Self(0);

//     const MAX: Self = Self(u8::MAX);

//     fn new(value: Self::Inner) -> Self {
//         Self(value)
//     }

//     fn offset(&self, offset: i32) -> Self {
//         let value = (self.0 as i32).saturating_add(offset * )
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct U8Range(u8);

impl ParamType for U8Range {
    fn as_value(&self) -> super::ParamValue {
        super::ParamValue::U8 { value: self.0 }
    }

    fn set_value(&mut self, value: super::ParamValue) {
        self.0 = value.as_u8_range();
    }
}

impl U8Range {
    pub fn new(value: u8) -> Self {
        Self(value)
    }
}
