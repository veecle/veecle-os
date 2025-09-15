//! SOME/IP dynamic and fixed length array de-/serialization.

use core::cmp::Ordering;
use core::marker::PhantomData;

use super::parse::{Parse, ParseError};
use crate::length::{LengthField, NoLengthField, OptionalLengthField};
use crate::parse::ByteReader;
use crate::serialize::{ByteWriter, Serialize, SerializeError};

/// A SOME/IP dynamic length array.
///
/// Dynamic length arrays are prefixed with their length in bytes, followed by an arbitrary number of elements.
///
/// The length is _only_ known inside [`parse_partial`](Parse::parse_partial), so the user of this type doesn't
/// have to manage it manually.
#[derive(Debug)]
pub struct DynamicLengthArray<'a, T, L, const MAX_ELEMENTS: usize> {
    /// Slice containing the array elements.
    reader: ByteReader<'a>,
    /// Marker for the element type and the length type.
    _marker: PhantomData<(T, L)>,
}

impl<'a, T, L, const MAX_ELEMENTS: usize> DynamicLengthArray<'a, T, L, MAX_ELEMENTS> {
    /// Creates a new array by serializing the elements to the provided buffer.
    pub fn create<'b, I>(
        mut elements: I,
        buffer: &'a mut [u8],
    ) -> Result<DynamicLengthArray<'a, T, L, MAX_ELEMENTS>, SerializeError>
    where
        T: Serialize + 'b,
        I: Iterator<Item = &'b T>,
    {
        let mut byte_writer = ByteWriter::new(buffer);

        let used_bytes = byte_writer.write_counted(move |byte_writer| {
            let mut element_count = 0;

            for element in elements.by_ref() {
                element.serialize_partial(byte_writer)?;
                element_count += 1;
            }

            match element_count <= MAX_ELEMENTS {
                true => Ok(()),
                false => Err(SerializeError::DynamicTypeOverflow),
            }
        })?;

        let reader = ByteReader::new(&buffer[..used_bytes]);

        Ok(DynamicLengthArray {
            reader,
            _marker: PhantomData,
        })
    }

    /// Returns an iterator over all elements in the array.
    pub fn iter(&self) -> DynamicLengthArrayIterator<'a, T, MAX_ELEMENTS> {
        DynamicLengthArrayIterator {
            reader: self.reader.clone(),
            element_count: 0,
            _marker: PhantomData,
        }
    }
}

impl<T, L, const MAX_ELEMENTS: usize> Clone for DynamicLengthArray<'_, T, L, MAX_ELEMENTS> {
    fn clone(&self) -> Self {
        Self {
            reader: self.reader.clone(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T, L, const MAX_ELEMENTS: usize> PartialEq for DynamicLengthArray<'a, T, L, MAX_ELEMENTS>
where
    T: Parse<'a> + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'a, T, L, const MAX_ELEMENTS: usize> Eq for DynamicLengthArray<'a, T, L, MAX_ELEMENTS> where
    T: Parse<'a> + Eq
{
}

impl<'a, T, L, const MAX_ELEMENTS: usize> Parse<'a> for DynamicLengthArray<'a, T, L, MAX_ELEMENTS>
where
    T: Parse<'a>,
    L: LengthField,
{
    fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
        let length = L::get_length(reader)?;
        let reader = reader.sub_reader(length)?;

        // Ensure that all the elements are valid.
        {
            let mut element_reader = reader.clone();
            let mut element_count = 0;

            // Variable length arrays exceeding expected length: interpret only specified elements, skip extra bytes.
            while !element_reader.is_empty() && element_count < MAX_ELEMENTS {
                let _ = T::parse_partial(&mut element_reader)?;
                element_count += 1;
            }
        }

        Ok(Self {
            reader,
            _marker: PhantomData,
        })
    }
}

impl<T, L, const MAX_ELEMENTS: usize> Serialize for DynamicLengthArray<'_, T, L, MAX_ELEMENTS>
where
    L: LengthField + Serialize,
{
    fn required_length(&self) -> usize {
        core::mem::size_of::<L>() + self.reader.len()
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        let reserved_length = byte_writer.reserve_length()?;

        let length = byte_writer
            .write_counted(|byte_writer| byte_writer.write_slice(self.reader.remaining_slice()))?;

        byte_writer.write_length(reserved_length, &L::from_length(length)?)
    }
}

/// Iterator for a [`DynamicLengthArray`].
#[derive(Debug)]
pub struct DynamicLengthArrayIterator<'a, T, const MAX_ELEMENTS: usize> {
    reader: ByteReader<'a>,
    element_count: usize,
    _marker: PhantomData<T>,
}

impl<'a, T, const MAX_ELEMENTS: usize> Iterator for DynamicLengthArrayIterator<'a, T, MAX_ELEMENTS>
where
    T: Parse<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.is_empty() || self.element_count >= MAX_ELEMENTS {
            return None;
        }

        self.element_count += 1;

        Some(T::parse_partial(&mut self.reader).unwrap())
    }
}

/// A SOME/IP fixed length array.
///
/// The length is _only_ known inside [`parse_partial`](Parse::parse_partial), so the user of this type doesn't
/// have to manage it manually.
#[derive(Debug)]
pub struct FixedLengthArray<'a, T, L, const ELEMENT_COUNT: usize> {
    /// Slice containing the array elements.
    reader: ByteReader<'a>,
    /// Marker for the element type and the length type.
    _marker: PhantomData<(T, L)>,
}

impl<'a, T, L, const ELEMENT_COUNT: usize> FixedLengthArray<'a, T, L, ELEMENT_COUNT> {
    /// Creates a new array by serializing the elements to the provided buffer.
    pub fn create<'b, I>(
        mut elements: I,
        buffer: &'a mut [u8],
    ) -> Result<FixedLengthArray<'a, T, L, ELEMENT_COUNT>, SerializeError>
    where
        T: Serialize + 'b,
        I: Iterator<Item = &'b T>,
    {
        let mut byte_writer = ByteWriter::new(buffer);

        let used_bytes = byte_writer.write_counted(move |byte_writer| {
            let mut element_count = 0;

            for element in elements.by_ref() {
                element.serialize_partial(byte_writer)?;
                element_count += 1;
            }

            match element_count.cmp(&ELEMENT_COUNT) {
                Ordering::Less => Err(SerializeError::DynamicTypeUnderflow),
                Ordering::Equal => Ok(()),
                Ordering::Greater => Err(SerializeError::DynamicTypeOverflow),
            }
        })?;

        let reader = ByteReader::new(&buffer[..used_bytes]);

        Ok(FixedLengthArray {
            reader,
            _marker: PhantomData,
        })
    }

    /// Returns an iterator over all elements in the array.
    pub fn iter(&self) -> FixedLengthArrayIterator<'a, T, ELEMENT_COUNT> {
        FixedLengthArrayIterator {
            reader: self.reader.clone(),
            element_count: 0,
            _marker: PhantomData,
        }
    }
}

impl<T, L, const ELEMENT_COUNT: usize> Clone for FixedLengthArray<'_, T, L, ELEMENT_COUNT> {
    fn clone(&self) -> Self {
        Self {
            reader: self.reader.clone(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T, L, const ELEMENT_COUNT: usize> PartialEq for FixedLengthArray<'a, T, L, ELEMENT_COUNT>
where
    T: Parse<'a> + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'a, T, L, const ELEMENT_COUNT: usize> Eq for FixedLengthArray<'a, T, L, ELEMENT_COUNT> where
    T: Parse<'a> + Eq
{
}

impl<'a, T, L, const ELEMENT_COUNT: usize> Parse<'a> for FixedLengthArray<'a, T, L, ELEMENT_COUNT>
where
    T: Parse<'a>,
    L: OptionalLengthField,
{
    fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
        let optional_length = L::try_get_length(reader)?;
        let array_reader = reader.clone();

        // Ensure that all the elements are valid.
        {
            // Missing elements should be substituted with the default values (if possible). This
            // is tricky in our case and vsomeip also seems to return a malformed message error if the array
            // is shorter than expected.
            if optional_length.is_some_and(|length| length < ELEMENT_COUNT) {
                return Err(ParseError::MalformedMessage {
                    failed_at: core::any::type_name::<Self>(),
                });
            }

            // Fixed length arrays exceeding expected length: interpret only specified elements, skip extra bytes.
            if optional_length.is_some_and(|length| length > ELEMENT_COUNT) {
                // TODO: We are unsure how to determine how many bytes need to be skipped, so for now we just return
                // a malformed message (this is what vsomeip also does).

                return Err(ParseError::MalformedMessage {
                    failed_at: core::any::type_name::<Self>(),
                });
            }

            for _ in 0..ELEMENT_COUNT {
                let _ = T::parse_partial(reader)?;
            }
        }

        Ok(Self {
            reader: array_reader,
            _marker: PhantomData,
        })
    }
}

impl<T, L, const ELEMENT_COUNT: usize> Serialize for FixedLengthArray<'_, T, L, ELEMENT_COUNT>
where
    L: LengthField + Serialize,
{
    fn required_length(&self) -> usize {
        core::mem::size_of::<L>() + self.reader.len()
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        L::from_length(ELEMENT_COUNT)?.serialize_partial(byte_writer)?;
        byte_writer.write_slice(self.reader.remaining_slice())
    }
}

impl<T, const ELEMENT_COUNT: usize> Serialize
    for FixedLengthArray<'_, T, NoLengthField, ELEMENT_COUNT>
{
    fn required_length(&self) -> usize {
        self.reader.len()
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        byte_writer.write_slice(self.reader.remaining_slice())
    }
}

/// Iterator for a [`FixedLengthArray`].
#[derive(Debug)]
pub struct FixedLengthArrayIterator<'a, T, const ELEMENT_COUNT: usize> {
    reader: ByteReader<'a>,
    element_count: usize,
    _marker: PhantomData<T>,
}

impl<'a, T, const ELEMENT_COUNT: usize> Iterator for FixedLengthArrayIterator<'a, T, ELEMENT_COUNT>
where
    T: Parse<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.element_count >= ELEMENT_COUNT {
            return None;
        }

        self.element_count += 1;

        Some(T::parse_partial(&mut self.reader).unwrap())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod dynamic_length_array {

    use crate::array::DynamicLengthArray;
    use crate::parse::{ByteReader, Parse, ParseError, ParseExt};
    use crate::serialize::{Serialize, SerializeError, SerializeExt};

    #[test]
    fn create_valid() {
        const EXPECTED_ELEMENTS: [u32; 2] = [10, 30];

        let mut buffer = [0; 64];
        let array =
            DynamicLengthArray::<'_, u32, u16, 5>::create(EXPECTED_ELEMENTS.iter(), &mut buffer)
                .unwrap();

        assert!(array.iter().eq(EXPECTED_ELEMENTS.into_iter()));
    }

    #[test]
    fn create_too_many_elements() {
        const TEST_ELEMENTS: [u32; 3] = [10, 30, 50];

        let mut buffer = [0; 64];
        assert_eq!(
            DynamicLengthArray::<'_, u32, u16, 2>::create(TEST_ELEMENTS.iter(), &mut buffer),
            Err(SerializeError::DynamicTypeOverflow)
        );
    }

    #[test]
    fn create_buffer_too_small() {
        const TEST_ELEMENTS: [u32; 2] = [10, 30];

        let mut buffer = [0; 7];
        assert_eq!(
            DynamicLengthArray::<'_, u32, u16, 5>::create(TEST_ELEMENTS.iter(), &mut buffer),
            Err(SerializeError::BufferTooSmall)
        );
    }

    #[test]
    fn conversion() {
        const TEST_ELEMENTS: [u32; 2] = [10, 30];
        const EXPECTED_BYTES: &[u8] = &[
            0, 8, // Length
            0, 0, 0, 10, // Item 0
            0, 0, 0, 30, // Item 1
        ];

        let mut buffer = [0; 64];
        let array =
            DynamicLengthArray::<'_, u32, u16, 5>::create(TEST_ELEMENTS.iter(), &mut buffer)
                .unwrap();

        test_round_trip!(DynamicLengthArray::<'_, u32, u16, 5>, array, EXPECTED_BYTES);
    }

    #[test]
    fn parse_too_many_elements() {
        const TEST_DATA: &[u8] = &[
            0, 8, // Partial length field
            0, 0, 0, 23, // Element 0
            0, 0, 0, 34, // Element 1
        ];

        const EXPECTED_ELEMENTS: [u32; 1] = [23];

        assert!(
            DynamicLengthArray::<'_, u32, u16, 1>::parse(TEST_DATA)
                .unwrap()
                .iter()
                .eq(EXPECTED_ELEMENTS.into_iter())
        );
    }

    #[test]
    fn parse_element_fails() {
        const TEST_DATA: &[u8] = &[
            0, 1, // Partial length field
            0, 0, 0, 23, // Element 0
        ];

        #[derive(Debug)]
        struct Fails;

        impl<'a> Parse<'a> for Fails {
            fn parse_partial(_: &mut ByteReader<'a>) -> Result<Self, ParseError> {
                Err(ParseError::MalformedMessage { failed_at: "Fails" })
            }
        }

        assert!(matches!(
            DynamicLengthArray::<'_, Fails, u16, 2>::parse(TEST_DATA),
            Err(ParseError::MalformedMessage { .. }),
        ));
    }

    #[test]
    fn serialize_length_overflow() {
        const TEST_ELEMENTS: [u8; 256] = [0; 256];

        let mut buffer = [0; 512];
        let array =
            DynamicLengthArray::<'_, u8, u8, 512>::create(TEST_ELEMENTS.iter(), &mut buffer)
                .unwrap();

        assert_eq!(
            array.serialize(&mut [0; 512]),
            Err(SerializeError::LengthOverflow)
        );
    }

    #[test]
    fn clone_without_clone_bound() {
        #[derive(Debug, PartialEq, Parse, Serialize)]
        struct NotClone;

        let mut buffer = [0; 512];
        let array = DynamicLengthArray::<'_, NotClone, u16, 2>::create(
            [NotClone, NotClone].iter(),
            &mut buffer,
        )
        .unwrap();

        assert_eq!(array, array.clone());
    }

    #[test]
    fn eq() {
        const TEST_DATA_1: &[u8] = &[
            0, 4, // Length
            0, 0, 0, 23, // Element 0
        ];

        const TEST_DATA_2: &[u8] = &[
            0, 4, // Length
            0, 0, 0, 23, // Element 0
            1, 2, 3, 4, // Additional data not used by the array
        ];

        let mut reader = ByteReader::new(TEST_DATA_1);
        let array_1 = DynamicLengthArray::<'_, u32, u16, 2>::parse_partial(&mut reader).unwrap();

        let mut reader = ByteReader::new(TEST_DATA_2);
        let array_2 = DynamicLengthArray::<'_, u32, u16, 2>::parse_partial(&mut reader).unwrap();

        assert_eq!(array_1, array_2);
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod fixed_length_array {

    use crate::array::FixedLengthArray;
    use crate::length::NoLengthField;
    use crate::parse::{ByteReader, Parse, ParseError, ParseExt};
    use crate::serialize::{Serialize, SerializeError, SerializeExt};

    #[test]
    fn create_valid() {
        const EXPECTED_ELEMENTS: [u32; 2] = [10, 30];

        let mut buffer = [0; 64];
        let array =
            FixedLengthArray::<'_, u32, u16, 2>::create(EXPECTED_ELEMENTS.iter(), &mut buffer)
                .unwrap();

        assert!(array.iter().eq(EXPECTED_ELEMENTS.into_iter()));
    }

    #[test]
    fn create_too_many_elements() {
        const TEST_ELEMENTS: [u32; 3] = [10, 30, 50];

        let mut buffer = [0; 64];
        assert_eq!(
            FixedLengthArray::<'_, u32, u16, 2>::create(TEST_ELEMENTS.iter(), &mut buffer),
            Err(SerializeError::DynamicTypeOverflow)
        );
    }

    #[test]
    fn create_too_few_elements() {
        const TEST_ELEMENTS: [u32; 1] = [10];

        let mut buffer = [0; 64];
        assert_eq!(
            FixedLengthArray::<'_, u32, u16, 2>::create(TEST_ELEMENTS.iter(), &mut buffer),
            Err(SerializeError::DynamicTypeUnderflow)
        );
    }

    #[test]
    fn create_buffer_too_small() {
        const TEST_ELEMENTS: [u32; 2] = [10, 30];

        let mut buffer = [0; 7];
        assert_eq!(
            FixedLengthArray::<'_, u32, u16, 2>::create(TEST_ELEMENTS.iter(), &mut buffer),
            Err(SerializeError::BufferTooSmall)
        );
    }

    #[test]
    fn conversion_without_length() {
        const TEST_ELEMENTS: [u32; 2] = [10, 30];
        const EXPECTED_BYTES: &[u8] = &[
            0, 0, 0, 10, // Item 0
            0, 0, 0, 30, // Item 1
        ];

        let mut buffer = [0; 64];
        let array = FixedLengthArray::<'_, u32, NoLengthField, 2>::create(
            TEST_ELEMENTS.iter(),
            &mut buffer,
        )
        .unwrap();

        test_round_trip!(
            FixedLengthArray::<'_, u32, NoLengthField, 2>,
            array,
            EXPECTED_BYTES
        );
    }

    #[test]
    fn conversion_with_length() {
        const TEST_ELEMENTS: [u32; 2] = [10, 30];
        const EXPECTED_BYTES: &[u8] = &[
            0, 2, // Length
            0, 0, 0, 10, // Item 0
            0, 0, 0, 30, // Item 1
        ];

        let mut buffer = [0; 64];
        let array =
            FixedLengthArray::<'_, u32, u16, 2>::create(TEST_ELEMENTS.iter(), &mut buffer).unwrap();

        test_round_trip!(FixedLengthArray::<'_, u32, u16, 2>, array, EXPECTED_BYTES);
    }

    // TODO: This test currently does not test for correct behaviour according to SOME/IP but rather the current
    // implementation. It will need to be adjusted when more properly implementing SOME/IP.
    #[test]
    fn parse_too_few_elements() {
        const TEST_DATA: &[u8] = &[
            0, 1, // Partial length field
            0, 0, 0, 23, // Element 0
        ];

        assert!(matches!(
            FixedLengthArray::<'_, u32, u16, 2>::parse(TEST_DATA),
            Err(ParseError::MalformedMessage { .. }),
        ));
    }

    // TODO: This test currently does not test for correct behaviour according to SOME/IP but rather the current
    // implementation. It will need to be adjusted when more properly implementing SOME/IP.
    #[test]
    fn parse_too_many_elements() {
        const TEST_DATA: &[u8] = &[
            0, 3, // Partial length field
            0, 0, 0, 23, // Element 0
            0, 0, 0, 34, // Element 1
            0, 0, 0, 45, // Element 2
        ];

        assert!(matches!(
            FixedLengthArray::<'_, u32, u16, 2>::parse(TEST_DATA),
            Err(ParseError::MalformedMessage { .. }),
        ));
    }

    #[test]
    fn serialize_length_overflow() {
        const TEST_ELEMENTS: [u8; 256] = [0; 256];

        let mut buffer = [0; 512];
        let array =
            FixedLengthArray::<'_, u8, u8, 256>::create(TEST_ELEMENTS.iter(), &mut buffer).unwrap();

        assert_eq!(
            array.serialize(&mut [0; 512]),
            Err(SerializeError::LengthOverflow)
        );
    }

    #[test]
    fn clone_without_clone_bound() {
        #[derive(Debug, PartialEq, Parse, Serialize)]
        struct NotClone;

        let mut buffer = [0; 512];
        let array = FixedLengthArray::<'_, NotClone, NoLengthField, 2>::create(
            [NotClone, NotClone].iter(),
            &mut buffer,
        )
        .unwrap();

        assert_eq!(array, array.clone());
    }

    #[test]
    fn eq() {
        const TEST_DATA_1: &[u8] = &[
            0, 1, // Length
            0, 0, 0, 23, // Element 0
        ];

        const TEST_DATA_2: &[u8] = &[
            0, 1, // Length
            0, 0, 0, 23, // Element 0
            1, 2, 3, 4, // Additional data not used by the array
        ];

        let mut reader = ByteReader::new(TEST_DATA_1);
        let array_1 = FixedLengthArray::<'_, u32, u16, 1>::parse_partial(&mut reader).unwrap();

        let mut reader = ByteReader::new(TEST_DATA_2);
        let array_2 = FixedLengthArray::<'_, u32, u16, 1>::parse_partial(&mut reader).unwrap();

        assert_eq!(array_1, array_2);
    }
}
