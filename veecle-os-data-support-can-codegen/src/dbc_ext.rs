//! Helper extension methods on `can_dbc` types.

pub(crate) trait AttributeValueExt {
    fn as_str(&self) -> Option<&str>;
}

pub(crate) trait DbcExt {
    fn find_raw_attribute_string(&self, name: &str) -> Option<&str>;
}

impl AttributeValueExt for can_dbc::AttributeValue {
    fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}

impl DbcExt for can_dbc::Dbc {
    fn find_raw_attribute_string(&self, name: &str) -> Option<&str> {
        self.attribute_values_database
            .iter()
            .find(|value| value.name == name)?
            .value
            .as_str()
    }
}
