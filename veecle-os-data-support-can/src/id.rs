/// A standard CAN id.
#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct StandardId(u16);

/// An extended CAN id.
#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct ExtendedId(u32);

impl StandardId {
    /// Creates a `StandardId`, returns `Some` if, and only if, `value < 0x800`.
    pub const fn new(value: u16) -> Option<Self> {
        if value < 0x800 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// The equivalent of `StandardId::new(value).unwrap()`, but as a `const fn`, while `unwrap` is
    /// not `const`-compatible.
    pub const fn new_unwrap(value: u16) -> Self {
        match Self::new(value) {
            Some(value) => value,
            None => panic!("out of range id"),
        }
    }

    /// Returns the CAN Identifier as a 16-bit integer.
    pub fn to_raw(self) -> u16 {
        self.into()
    }
}

impl TryFrom<u16> for StandardId {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(())
    }
}

impl TryFrom<u32> for StandardId {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        u16::try_from(value).ok().and_then(Self::new).ok_or(())
    }
}

impl From<StandardId> for u16 {
    fn from(value: StandardId) -> Self {
        value.0
    }
}

impl core::fmt::Debug for StandardId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl<'de> serde::Deserialize<'de> for StandardId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u16::deserialize(deserializer)?;
        StandardId::new(value)
            .ok_or_else(|| serde::de::Error::custom("standard CAN id must be < 0x800"))
    }
}

impl ExtendedId {
    /// Creates an `ExtendedId`, returns `Some` if, and only if, `value < 0x2000_0000`.
    pub const fn new(value: u32) -> Option<Self> {
        if value < 0x2000_0000 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// The equivalent of `ExtendedId::new(value).unwrap()`, but as a `const fn`, while `unwrap` is
    /// not `const`-compatible.
    pub const fn new_unwrap(value: u32) -> Self {
        match Self::new(value) {
            Some(value) => value,
            None => panic!("out of range id"),
        }
    }

    /// Returns the CAN Identifier as a 32-bit integer.
    pub fn to_raw(self) -> u32 {
        self.into()
    }
}

impl TryFrom<u32> for ExtendedId {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(())
    }
}

impl From<ExtendedId> for u32 {
    fn from(value: ExtendedId) -> Self {
        value.0
    }
}

impl core::fmt::Debug for ExtendedId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.0)?;
        Ok(())
    }
}

impl<'de> serde::Deserialize<'de> for ExtendedId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        ExtendedId::new(value)
            .ok_or_else(|| serde::de::Error::custom("extended CAN id must be < 0x2000_0000"))
    }
}

/// Either a standard or extended CAN id.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Id {
    /// A standard CAN id.
    Standard(StandardId),

    /// An extended CAN id.
    Extended(ExtendedId),
}

impl From<StandardId> for Id {
    fn from(standard: StandardId) -> Self {
        Self::Standard(standard)
    }
}

impl From<ExtendedId> for Id {
    fn from(extended: ExtendedId) -> Self {
        Self::Extended(extended)
    }
}

/// All `Id` values are <0x2000_0000 so we have the top three bits spare, this type packs the discriminant into the top
/// bit and removes alignment to minimize the storage space required.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(Rust, packed)]
pub struct PackedId(u32);

impl From<Id> for PackedId {
    fn from(id: Id) -> Self {
        match id {
            Id::Standard(StandardId(value)) => PackedId(u32::from(value)),
            Id::Extended(ExtendedId(value)) => PackedId(value | 0x8000_0000),
        }
    }
}

impl From<PackedId> for Id {
    fn from(id: PackedId) -> Self {
        let PackedId(value) = id;
        if value & 0x8000_0000 == 0x8000_0000 {
            Id::Extended(ExtendedId(value & !0x8000_0000))
        } else {
            Id::Standard(StandardId(value as u16))
        }
    }
}

// Manual serde implementations to serialize/deserialize via the `Id` enum instead of the packed representation.
// This ensures the serialized format uses the logical representation and deserialization maintains invariants.
impl serde::Serialize for PackedId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Id::from(*self).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for PackedId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = Id::deserialize(deserializer)?;
        Ok(PackedId::from(id))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::format;
    use std::string::ToString;

    use crate::id::PackedId;
    use crate::{ExtendedId, Id, StandardId};

    const STANDARD_ID_VALIDS: [u16; 4] = [0, 1, 0x7FE, 0x7FF];
    const STANDARD_ID_INVALIDS: [u16; 2] = [0x800, 0xFFFF];
    const EXTENDED_ID_VALIDS: [u32; 4] = [0, 1, 0x1FFF_FFFE, 0x1FFF_FFFF];
    const EXTENDED_ID_INVALIDS: [u32; 2] = [0x2000_0000, 0xFFFF_FFFF];

    #[test]
    fn pack_roundtrip() {
        for id in STANDARD_ID_VALIDS {
            let id = Id::Standard(StandardId::new(id).unwrap());
            assert_eq!(id, Id::from(PackedId::from(id)));
        }
        for id in EXTENDED_ID_VALIDS {
            let id = Id::Extended(ExtendedId::new(id).unwrap());
            assert_eq!(id, Id::from(PackedId::from(id)));
        }
    }

    #[test]
    fn id_to_integer() {
        for value in STANDARD_ID_VALIDS {
            let standard_id = StandardId::new(value).unwrap();
            assert_eq!(standard_id.to_raw(), u16::from(standard_id));
            assert_eq!(value, standard_id.to_raw());
        }
        for value in EXTENDED_ID_VALIDS {
            let extended_id = ExtendedId::new(value).unwrap();
            assert_eq!(extended_id.to_raw(), u32::from(extended_id));
            assert_eq!(value, extended_id.to_raw());
        }
    }

    #[test]
    fn test_deserialize_standard_id_valid() {
        for value in STANDARD_ID_VALIDS {
            let json = value.to_string();
            let id: StandardId = serde_json::from_str(&json).unwrap();
            assert_eq!(id, StandardId::new(value).unwrap());
            assert_eq!(json, serde_json::to_string(&id).unwrap());
        }
    }

    #[test]
    fn test_deserialize_standard_id_invalid() {
        for value in STANDARD_ID_INVALIDS {
            let json = value.to_string();
            assert!(serde_json::from_str::<StandardId>(&json).is_err());
        }
    }

    #[test]
    fn test_deserialize_extended_id_valid() {
        for value in EXTENDED_ID_VALIDS {
            let json = value.to_string();
            let id: ExtendedId = serde_json::from_str(&json).unwrap();
            assert_eq!(id, ExtendedId::new(value).unwrap());
            assert_eq!(json, serde_json::to_string(&id).unwrap());
        }
    }

    #[test]
    fn test_deserialize_extended_id_invalid() {
        for value in EXTENDED_ID_INVALIDS {
            let json = value.to_string();
            assert!(serde_json::from_str::<ExtendedId>(&json).is_err());
        }
    }

    #[test]
    fn test_deserialize_id_and_packed_id_valid() {
        for value in STANDARD_ID_VALIDS {
            let json = format!(r#"{{"Standard":{value}}}"#);
            let id: Id = serde_json::from_str(&json).unwrap();
            assert_eq!(id, Id::Standard(StandardId::new(value).unwrap()));
            assert_eq!(json, serde_json::to_string(&id).unwrap());
            let packed: PackedId = serde_json::from_str(&json).unwrap();
            assert_eq!(packed, PackedId::from(id));
            assert_eq!(json, serde_json::to_string(&packed).unwrap());
        }
        for value in EXTENDED_ID_VALIDS {
            let json = format!(r#"{{"Extended":{value}}}"#);
            let id: Id = serde_json::from_str(&json).unwrap();
            assert_eq!(id, Id::Extended(ExtendedId::new(value).unwrap()));
            assert_eq!(json, serde_json::to_string(&id).unwrap());
            let packed: PackedId = serde_json::from_str(&json).unwrap();
            assert_eq!(packed, PackedId::from(id));
            assert_eq!(json, serde_json::to_string(&packed).unwrap());
        }
    }

    #[test]
    fn test_deserialize_id_and_packed_id_invalid() {
        for value in STANDARD_ID_INVALIDS {
            let json = format!(r#"{{"Standard":{value}}}"#);
            assert!(serde_json::from_str::<Id>(&json).is_err());
            assert!(serde_json::from_str::<PackedId>(&json).is_err());
        }
        for value in EXTENDED_ID_INVALIDS {
            let json = format!(r#"{{"Extended":{value}}}"#);
            assert!(serde_json::from_str::<Id>(&json).is_err());
            assert!(serde_json::from_str::<PackedId>(&json).is_err());
        }
    }
}
