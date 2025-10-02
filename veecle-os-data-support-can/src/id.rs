/// A standard CAN id.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StandardId(u16);

/// An extended CAN id.
#[derive(Clone, Copy, PartialEq, Eq)]
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

/// Either a standard or extended CAN id.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
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

#[cfg(test)]
mod tests {
    use crate::id::PackedId;
    use crate::{ExtendedId, Id, StandardId};

    #[test]
    fn pack_roundtrip() {
        for id in [0, 1, 0x7FE, 0x7FF] {
            let id = Id::Standard(StandardId::new(id).unwrap());
            assert_eq!(id, Id::from(PackedId::from(id)));
        }
        for id in [0, 1, 0x1FFF_FFFE, 0x1FFF_FFFF] {
            let id = Id::Extended(ExtendedId::new(id).unwrap());
            assert_eq!(id, Id::from(PackedId::from(id)));
        }
    }

    #[test]
    fn id_to_integer() {
        for id in [0, 1, 0x7FE, 0x7FF] {
            let standard_id = StandardId::new(id).unwrap();
            assert_eq!(standard_id.to_raw(), u16::from(standard_id));
            assert_eq!(id, standard_id.to_raw());
        }
        for id in [0, 1, 0x1FFF_FFFE, 0x1FFF_FFFF] {
            let extended_id = ExtendedId::new(id).unwrap();
            assert_eq!(extended_id.to_raw(), u32::from(extended_id));
            assert_eq!(id, extended_id.to_raw());
        }
    }

    #[test]
    fn test_deserialize_packed_id() {
        let id = PackedId::from(Id::from(ExtendedId::new_unwrap(0x1234_5678)));
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: PackedId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }
}
