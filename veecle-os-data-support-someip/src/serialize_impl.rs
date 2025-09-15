//! Implementation of [`Serialize`] for various data types.

use crate::serialize::{ByteWriter, Serialize, SerializeError};

impl Serialize for bool {
    fn required_length(&self) -> usize {
        1
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        byte_writer.write_byte(match self {
            true => 1,
            false => 0,
        })
    }
}

macro_rules! impl_for_numeric {
    ($ty:ident) => {
        impl Serialize for $ty {
            fn required_length(&self) -> usize {
                core::mem::size_of::<Self>()
            }

            fn serialize_partial(
                &self,
                byte_writer: &mut ByteWriter,
            ) -> Result<(), SerializeError> {
                byte_writer.write_slice(&self.to_be_bytes())
            }
        }
    };
}

impl_for_numeric!(u8);
impl_for_numeric!(u16);
impl_for_numeric!(u32);
impl_for_numeric!(u64);

impl_for_numeric!(i8);
impl_for_numeric!(i16);
impl_for_numeric!(i32);
impl_for_numeric!(i64);

impl_for_numeric!(f32);
impl_for_numeric!(f64);
