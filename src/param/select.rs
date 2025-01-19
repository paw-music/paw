use super::ParamValue;

pub struct SelectValue<'a> {
    prev: Option<&'a SelectValue<'a>>,
    next: Option<&'a SelectValue<'a>>,
    value: ParamValue,
}
