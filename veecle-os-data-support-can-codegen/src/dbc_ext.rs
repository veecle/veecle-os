//! Helper extension methods on `can_dbc` types.

pub(crate) trait AttributeValueExt {
    fn as_str(&self) -> Option<&str>;
}

pub(crate) trait AttributeValuedForObjectTypeExt {
    fn as_raw(&self) -> Option<&can_dbc::AttributeValue>;
}

pub(crate) trait DBCExt {
    fn find_raw_attribute_string(&self, name: &str) -> Option<&str>;
}

impl AttributeValueExt for can_dbc::AttributeValue {
    fn as_str(&self) -> Option<&str> {
        match self {
            Self::AttributeValueCharString(s) => Some(s),
            _ => None,
        }
    }
}

impl AttributeValuedForObjectTypeExt for can_dbc::AttributeValuedForObjectType {
    fn as_raw(&self) -> Option<&can_dbc::AttributeValue> {
        match self {
            Self::RawAttributeValue(value) => Some(value),
            _ => None,
        }
    }
}

impl DBCExt for can_dbc::DBC {
    fn find_raw_attribute_string(&self, name: &str) -> Option<&str> {
        self.attribute_values()
            .iter()
            .find(|value| value.attribute_name() == name)?
            .attribute_value()
            .as_raw()?
            .as_str()
    }
}
