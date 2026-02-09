// editorconfig-checker-disable
//! J1939 v1.0.0 for CAN by CSS ELECTRONICS (WWW.CSSELECTRONICS.COM)
#![allow(dead_code)]
use ::my_serde as _serde;
pub mod eec1 {
    use ::my_veecle_os_data_support_can::reëxports::bits;
    use ::my_serde as _serde;
    /** ```text
Actual engine speed which is calculated over a minimum crankshaft angle of 720 degrees divided by the number of cylinders.…
```*/
    #[derive(Clone, Copy, PartialEq, PartialOrd, _serde::Serialize)]
    #[serde(crate = "_serde")]
    pub struct EngineSpeed {
        raw: u16,
    }
    impl EngineSpeed {
        pub const MAX: Self = Self { raw: 64255 };
        pub const MIN: Self = Self { raw: 0 };
        fn try_from_raw(
            raw: u16,
        ) -> Result<Self, ::my_veecle_os_data_support_can::CanDecodeError> {
            Self::try_from(raw as f64 * 0.125)
        }
        fn raw(&self) -> u16 {
            self.raw
        }
        pub(super) fn read_bits(
            bytes: &[u8],
        ) -> Result<Self, ::my_veecle_os_data_support_can::CanDecodeError> {
            Self::try_from_raw(
                u16::try_from(bits::read_little_endian_unsigned(bytes, 24, 16)).unwrap(),
            )
        }
        pub(super) fn write_bits(&self, bytes: &mut [u8]) {
            bits::write_little_endian_unsigned(bytes, 24, 16, self.raw().into())
        }
        pub fn value(&self) -> f64 {
            self.raw as f64 * 0.125
        }
    }
    impl Default for EngineSpeed {
        fn default() -> Self {
            Self::MIN
        }
    }
    impl TryFrom<f64> for EngineSpeed {
        type Error = ::my_veecle_os_data_support_can::CanDecodeError;
        fn try_from(value: f64) -> Result<Self, Self::Error> {
            if (0.0..=8031.875).contains(&value) {
                Ok(Self {
                    raw: ((value / 0.125 + 0.5) as u16),
                })
            } else {
                Err(Self::Error::OutOfRange {
                    name: stringify!(EngineSpeed),
                    ty: stringify!(f64),
                    message: "out of range 0.0..=8031.875",
                })
            }
        }
    }
    impl ::my_veecle_os_runtime::Storable for EngineSpeed {
        type DataType = Self;
    }
    impl core::fmt::Debug for EngineSpeed {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("EngineSpeed")
                .field("raw", &self.raw)
                .field("value", &self.value())
                .finish()
        }
    }
    #[cfg(all())]
    impl<'a> ::my_arbitrary::Arbitrary<'a> for EngineSpeed {
        fn arbitrary(
            u: &mut ::my_arbitrary::Unstructured<'a>,
        ) -> ::my_arbitrary::Result<Self> {
            let min = Self::MIN.raw();
            let max = Self::MAX.raw();
            Ok(
                Self::try_from_raw(u.int_in_range(min..=max)?)
                    .expect("we generate in range"),
            )
        }
    }
}
/** ```text
Electronic Engine Controller 1
```*/
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, _serde::Serialize)]
#[serde(crate = "_serde")]
pub struct Eec1 {
    pub engine_speed: eec1::EngineSpeed,
}
impl Eec1 {
    pub const FRAME_ID: ::my_veecle_os_data_support_can::Id = ::my_veecle_os_data_support_can::Id::Extended(
        ::my_veecle_os_data_support_can::ExtendedId::new_unwrap(0xcf004fe),
    );
    pub const FRAME_LENGTH: usize = 8usize;
}
impl TryFrom<&::my_veecle_os_data_support_can::Frame> for Eec1 {
    type Error = ::my_veecle_os_data_support_can::CanDecodeError;
    fn try_from(
        frame: &::my_veecle_os_data_support_can::Frame,
    ) -> Result<Self, Self::Error> {
        if frame.id() != Self::FRAME_ID {
            return Err(::my_veecle_os_data_support_can::CanDecodeError::IncorrectId);
        }
        let bytes: [u8; Self::FRAME_LENGTH] = frame
            .data()
            .try_into()
            .map_err(|_| {
                ::my_veecle_os_data_support_can::CanDecodeError::IncorrectBufferSize
            })?;
        Ok(Self {
            engine_speed: eec1::EngineSpeed::read_bits(&bytes)?,
        })
    }
}
impl TryFrom<::my_veecle_os_data_support_can::Frame> for Eec1 {
    type Error = ::my_veecle_os_data_support_can::CanDecodeError;
    fn try_from(
        frame: ::my_veecle_os_data_support_can::Frame,
    ) -> Result<Self, Self::Error> {
        Self::try_from(&frame)
    }
}
impl From<&Eec1> for ::my_veecle_os_data_support_can::Frame {
    fn from(value: &Eec1) -> Self {
        let mut bytes = [0u8; Eec1::FRAME_LENGTH];
        value.engine_speed.write_bits(&mut bytes);
        Frame::new(Eec1::FRAME_ID, bytes)
    }
}
impl From<Eec1> for ::my_veecle_os_data_support_can::Frame {
    fn from(value: Eec1) -> Self {
        Self::from(&value)
    }
}
impl ::my_veecle_os_runtime::Storable for Eec1 {
    type DataType = Self;
}
#[cfg(all())]
impl<'a> ::my_arbitrary::Arbitrary<'a> for Eec1 {
    fn arbitrary(
        u: &mut ::my_arbitrary::Unstructured<'a>,
    ) -> ::my_arbitrary::Result<Self> {
        Ok(Self {
            engine_speed: u.arbitrary()?,
        })
    }
}
pub mod ccvs1 {
    use ::my_veecle_os_data_support_can::reëxports::bits;
    use ::my_serde as _serde;
    /** ```text
Wheel-Based Vehicle Speed: Speed of the vehicle as calculated from wheel or tailshaft speed.
```*/
    #[derive(Clone, Copy, PartialEq, PartialOrd, _serde::Serialize)]
    #[serde(crate = "_serde")]
    pub struct WheelBasedVehicleSpeed {
        raw: u16,
    }
    impl WheelBasedVehicleSpeed {
        pub const MAX: Self = Self { raw: 64255 };
        pub const MIN: Self = Self { raw: 0 };
        fn try_from_raw(
            raw: u16,
        ) -> Result<Self, ::my_veecle_os_data_support_can::CanDecodeError> {
            Self::try_from(raw as f64 * 0.00390625)
        }
        fn raw(&self) -> u16 {
            self.raw
        }
        pub(super) fn read_bits(
            bytes: &[u8],
        ) -> Result<Self, ::my_veecle_os_data_support_can::CanDecodeError> {
            Self::try_from_raw(
                u16::try_from(bits::read_little_endian_unsigned(bytes, 8, 16)).unwrap(),
            )
        }
        pub(super) fn write_bits(&self, bytes: &mut [u8]) {
            bits::write_little_endian_unsigned(bytes, 8, 16, self.raw().into())
        }
        pub fn value(&self) -> f64 {
            self.raw as f64 * 0.00390625
        }
    }
    impl Default for WheelBasedVehicleSpeed {
        fn default() -> Self {
            Self::MIN
        }
    }
    impl TryFrom<f64> for WheelBasedVehicleSpeed {
        type Error = ::my_veecle_os_data_support_can::CanDecodeError;
        fn try_from(value: f64) -> Result<Self, Self::Error> {
            if (0.0..=250.996).contains(&value) {
                Ok(Self {
                    raw: ((value / 0.00390625 + 0.5) as u16),
                })
            } else {
                Err(Self::Error::OutOfRange {
                    name: stringify!(WheelBasedVehicleSpeed),
                    ty: stringify!(f64),
                    message: "out of range 0.0..=250.996",
                })
            }
        }
    }
    impl ::my_veecle_os_runtime::Storable for WheelBasedVehicleSpeed {
        type DataType = Self;
    }
    impl core::fmt::Debug for WheelBasedVehicleSpeed {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("WheelBasedVehicleSpeed")
                .field("raw", &self.raw)
                .field("value", &self.value())
                .finish()
        }
    }
    #[cfg(all())]
    impl<'a> ::my_arbitrary::Arbitrary<'a> for WheelBasedVehicleSpeed {
        fn arbitrary(
            u: &mut ::my_arbitrary::Unstructured<'a>,
        ) -> ::my_arbitrary::Result<Self> {
            let min = Self::MIN.raw();
            let max = Self::MAX.raw();
            Ok(
                Self::try_from_raw(u.int_in_range(min..=max)?)
                    .expect("we generate in range"),
            )
        }
    }
}
/** ```text
Cruise Control/Vehicle Speed 1
```*/
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, _serde::Serialize)]
#[serde(crate = "_serde")]
pub struct Ccvs1 {
    pub wheel_based_vehicle_speed: ccvs1::WheelBasedVehicleSpeed,
}
impl Ccvs1 {
    pub const FRAME_ID: ::my_veecle_os_data_support_can::Id = ::my_veecle_os_data_support_can::Id::Extended(
        ::my_veecle_os_data_support_can::ExtendedId::new_unwrap(0x18fef1fe),
    );
    pub const FRAME_LENGTH: usize = 8usize;
}
impl TryFrom<&::my_veecle_os_data_support_can::Frame> for Ccvs1 {
    type Error = ::my_veecle_os_data_support_can::CanDecodeError;
    fn try_from(
        frame: &::my_veecle_os_data_support_can::Frame,
    ) -> Result<Self, Self::Error> {
        if frame.id() != Self::FRAME_ID {
            return Err(::my_veecle_os_data_support_can::CanDecodeError::IncorrectId);
        }
        let bytes: [u8; Self::FRAME_LENGTH] = frame
            .data()
            .try_into()
            .map_err(|_| {
                ::my_veecle_os_data_support_can::CanDecodeError::IncorrectBufferSize
            })?;
        Ok(Self {
            wheel_based_vehicle_speed: ccvs1::WheelBasedVehicleSpeed::read_bits(&bytes)?,
        })
    }
}
impl TryFrom<::my_veecle_os_data_support_can::Frame> for Ccvs1 {
    type Error = ::my_veecle_os_data_support_can::CanDecodeError;
    fn try_from(
        frame: ::my_veecle_os_data_support_can::Frame,
    ) -> Result<Self, Self::Error> {
        Self::try_from(&frame)
    }
}
impl From<&Ccvs1> for ::my_veecle_os_data_support_can::Frame {
    fn from(value: &Ccvs1) -> Self {
        let mut bytes = [0u8; Ccvs1::FRAME_LENGTH];
        value.wheel_based_vehicle_speed.write_bits(&mut bytes);
        Frame::new(Ccvs1::FRAME_ID, bytes)
    }
}
impl From<Ccvs1> for ::my_veecle_os_data_support_can::Frame {
    fn from(value: Ccvs1) -> Self {
        Self::from(&value)
    }
}
impl ::my_veecle_os_runtime::Storable for Ccvs1 {
    type DataType = Self;
}
#[cfg(all())]
impl<'a> ::my_arbitrary::Arbitrary<'a> for Ccvs1 {
    fn arbitrary(
        u: &mut ::my_arbitrary::Unstructured<'a>,
    ) -> ::my_arbitrary::Result<Self> {
        Ok(Self {
            wheel_based_vehicle_speed: u.arbitrary()?,
        })
    }
}
use ::my_veecle_os_data_support_can::Frame;
/// An actor that will attempt to parse any [`Frame`] messages and publish the parsed messages.
///
/// If used you must also provide some interface-actor that writes the `Frame`s from your transceiver.
#[::my_veecle_os_runtime::actor(crate = ::my_veecle_os_runtime)]
pub async fn deserialize_frames(
    mut reader: ::my_veecle_os_runtime::single_writer::Reader<'_, Frame>,
    mut eec1_writer: ::my_veecle_os_runtime::single_writer::Writer<'_, Eec1>,
    mut ccvs1_writer: ::my_veecle_os_runtime::single_writer::Writer<'_, Ccvs1>,
) -> ::my_veecle_os_runtime::Never {
    loop {
        let frame = reader.read_updated_cloned().await;
        match frame.id() {
            Eec1::FRAME_ID => {
                let Ok(msg) = Eec1::try_from(frame) else { continue };
                eec1_writer.write(msg).await;
            }
            Ccvs1::FRAME_ID => {
                let Ok(msg) = Ccvs1::try_from(frame) else { continue };
                ccvs1_writer.write(msg).await;
            }
            _ => {}
        }
    }
}
