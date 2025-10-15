use tinyvec::ArrayVec;

use crate::id::{Id, PackedId};

/// A frame of CAN data, useful for passing received frames between Veecle OS actors before they get deserialized.
#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Frame {
    /// The `id` is stored packed to save space, with `PackedId` a `Frame` is 14 bytes, without it, it is 20 bytes.
    id: PackedId,
    data: ArrayVec<[u8; 8]>,
}

mod sealed {
    /// Stop external implementations, technically not necessary because we only care about implementations on `[u8; N]`
    /// which no external crate can add. But, having this should hopefully stop anyone attempting to implement
    /// `FrameSize` on their types
    pub trait Sealed {}
    impl Sealed for [u8; 0] {}
    impl Sealed for [u8; 1] {}
    impl Sealed for [u8; 2] {}
    impl Sealed for [u8; 3] {}
    impl Sealed for [u8; 4] {}
    impl Sealed for [u8; 5] {}
    impl Sealed for [u8; 6] {}
    impl Sealed for [u8; 7] {}
    impl Sealed for [u8; 8] {}
}

/// A marker trait for arrays that are valid sizes for CAN frames.
pub trait FrameSize: sealed::Sealed {}
impl FrameSize for [u8; 0] {}
impl FrameSize for [u8; 1] {}
impl FrameSize for [u8; 2] {}
impl FrameSize for [u8; 3] {}
impl FrameSize for [u8; 4] {}
impl FrameSize for [u8; 5] {}
impl FrameSize for [u8; 6] {}
impl FrameSize for [u8; 7] {}
impl FrameSize for [u8; 8] {}

impl Frame {
    /// Create a frame with the passed id and data.
    ///
    /// Statically checked that `N <= 8` via [`FrameSize`].
    pub fn new<const N: usize>(id: impl Into<Id>, data: [u8; N]) -> Self
    where
        [u8; N]: FrameSize,
    {
        Self::new_checked(id, &data).expect("the const generic guarantees it's ok")
    }

    /// Create a frame with the passed id and data.
    ///
    /// Returns `Some` iff `data.len() <= 8`.
    pub fn new_checked(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        let id = PackedId::from(id.into());
        let data = ArrayVec::try_from(data).ok()?;
        Some(Self { id, data })
    }

    /// The id this frame was received with.
    pub fn id(&self) -> Id {
        self.id.into()
    }

    /// The data this frame was received with.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new(crate::StandardId::new(0).unwrap(), [])
    }
}

impl veecle_os_runtime::Storable for Frame {
    type DataType = Self;
}

impl core::fmt::Debug for Frame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Frame {{ id: {:?}, data: '", self.id())?;
        for byte in self.data() {
            write!(f, "{byte:02x}")?;
        }
        f.write_str("' }")?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::Frame;

    #[test]
    fn test_deserialize_frame_standard() {
        let json = r#"{"id":{"Standard":291},"data":[1,2,3,4]}"#;
        let frame: Frame = serde_json::from_str(json).unwrap();
        assert_eq!(frame.id(), crate::StandardId::new(0x123).unwrap().into());
        assert_eq!(frame.data(), &[1, 2, 3, 4]);
        assert_eq!(json, serde_json::to_string(&frame).unwrap());
    }

    #[test]
    fn test_deserialize_frame_extended() {
        let json = r#"{"id":{"Extended":74565},"data":[1,2,3,4]}"#;
        let frame: Frame = serde_json::from_str(json).unwrap();
        assert_eq!(frame.id(), crate::ExtendedId::new(74565).unwrap().into());
        assert_eq!(frame.data(), &[1, 2, 3, 4]);
        assert_eq!(json, serde_json::to_string(&frame).unwrap());
    }

    /// More of an example of the output format than a real test, but as a test to force updating it.
    #[test]
    fn test_debug() {
        fn to_debug(value: impl core::fmt::Debug) -> std::string::String {
            std::format!("{value:?}")
        }

        assert_eq!(
            to_debug(Frame::new(crate::StandardId::new(0).unwrap(), [])),
            "Frame { id: Standard(0x0), data: '' }"
        );

        assert_eq!(
            to_debug(Frame::new(
                crate::ExtendedId::new(0x153EAB12).unwrap(),
                [0x04, 0xA2, 0xC2, 0xED, 0xCA, 0xE3, 0x88, 0x74]
            )),
            "Frame { id: Extended(0x153eab12), data: '04a2c2edcae38874' }"
        );

        assert_eq!(
            to_debug(Frame::new(
                crate::ExtendedId::new(0x1B56C72D).unwrap(),
                [0x40, 0x71, 0xEF, 0x61]
            )),
            "Frame { id: Extended(0x1b56c72d), data: '4071ef61' }"
        );
    }
}
