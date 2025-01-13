pub enum ParamValueKind {
    /// From 0.0 to 1.0 inclusive
    UnitInterval,
    /// From 0.0 to 0.5 inclusive
    HalfUnitInterval,
    /// All u8 values
    U8,
}

/// Generic parameter
pub struct Param {
    name: &'static str,
}
