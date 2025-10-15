//! Bit-level helpers for the macro generated code to read/write signals within a frame.

/// Takes a length-bit 2's complement encoded value and sign extends it to a full `i64`.
///
/// Higher bits must be 0 to get the correct result.
///
/// For example, for `length: 3` this does:
///
///  00…00000 -> 00…00000  (0)
///  00…00001 -> 00…00001  (1)
///  00…00010 -> 00…00010  (2)
///  00…00011 -> 00…00011  (3)
///  00…00100 -> 11…11100  (-4)
///  00…00101 -> 11…11101  (-3)
///  00…00110 -> 11…11110  (-2)
///  00…00111 -> 11…11111  (-1)
fn sign_extend(length: usize, mut value: u64) -> i64 {
    if (value & (1 << (length - 1))) != 0 {
        value |= !((1 << (length - 1)) - 1);
    }
    value as i64
}

/// [`u64::from_le_bytes`] but zero-extends the slice to the right if it is too short.
fn u64_from_le_slice(bytes: &[u8]) -> u64 {
    let mut buffer = [0; 8];
    buffer[..bytes.len()].copy_from_slice(bytes);
    u64::from_le_bytes(buffer)
}

/// [`u64::from_be_bytes`] but zero-extends the slice to the right if it is too short.
fn u64_from_be_slice(bytes: &[u8]) -> u64 {
    let mut buffer = [0; 8];
    buffer[..bytes.len()].copy_from_slice(bytes);
    u64::from_be_bytes(buffer)
}

/// Reads a big-endian CAN value with specified length and offset from within a byte buffer.
///
/// Big endian is simple in that the bits are in the same order as the input string, they just need shifting to the
/// correct position in the output value (with masking to discard bits before the start).
///
/// ```text
/// offset: 7, length: 10
///                _______________
/// bytes: 0000 0001  1000 0001  1000 0000
///
/// result: 11 0000 0011
/// ```
///
/// ```text
/// offset: 1, length: 3
///         ___
/// bytes: 0010 0000
///
/// result: 010
/// ```
pub fn read_big_endian_unsigned(bytes: &[u8], offset: usize, length: usize) -> u64 {
    assert!(bytes.len() <= 8, "maximum supported input is 8 bytes");
    assert!(offset + length <= 64, "maximum supported input is 8 bytes");

    let shift = 64 - offset - length;
    let mut result = u64_from_be_slice(bytes) >> shift;
    if length != 64 {
        result &= (1 << length) - 1;
    }
    result
}

/// [`read_big_endian_unsigned`] but then sign-extends the result.
pub fn read_big_endian_signed(bytes: &[u8], offset: usize, length: usize) -> i64 {
    sign_extend(length, read_big_endian_unsigned(bytes, offset, length))
}

/// Writes a big-endian CAN value with specified length and offset into a byte buffer.
///
/// See [`read_big_endian_unsigned`] for some examples of the expected mapping between values and the byte buffer.
pub fn write_big_endian_unsigned(bytes: &mut [u8], offset: usize, length: usize, value: u64) {
    assert!(bytes.len() <= 8, "maximum supported input is 8 bytes");
    assert!(offset + length <= 64, "maximum supported input is 8 bytes");

    let shift = 64 - offset - length;
    let mask = u64::MAX >> (64 - length);

    let mut original = u64_from_be_slice(bytes);
    original &= !(mask << shift);
    original |= (value & mask) << shift;
    let result = original.to_be_bytes();

    bytes.copy_from_slice(&result[..bytes.len()]);
}

/// Writes a big-endian CAN value with specified length and offset into a byte buffer.
pub fn write_big_endian_signed(bytes: &mut [u8], offset: usize, length: usize, value: i64) {
    write_big_endian_unsigned(bytes, offset, length, value as u64)
}

/// Reads a little-endian CAN value with specified length and offset from within a byte buffer.
///
/// Little endian looks complicated because it's using least-significant-bit-zero indexing, but not actually reversing
/// the bits. In this first example you can see the `offset: 2` means we start at the third most insignificant bit
/// (sixth from the left), and take the rest of the first byte to the most significant bit (left-most). Then the
/// length is enough to take the full second byte, and finally three bits from the third byte, starting counting from
/// the least-significant (right-most).
///
/// ```text
/// offset: 2, length: 17
///
///        _______    ---------        ===
/// bytes: 0100 1000  0111 0001  0000 0110
///
///         =====-----------_______
/// result: 1  1001 1100  0101 0010
/// ```
///
/// ```text
/// offset: 2, length: 3
///           ____
/// bytes: 0000 0100
///
/// result: 001
/// ```
pub fn read_little_endian_unsigned(bytes: &[u8], offset: usize, length: usize) -> u64 {
    assert!(bytes.len() <= 8, "maximum supported input is 8 bytes");
    assert!(offset + length <= 64, "maximum supported input is 8 bytes");

    let mut result = u64_from_le_slice(bytes) >> offset;
    if length != 64 {
        result &= (1 << length) - 1;
    }
    result
}

/// [`read_little_endian_unsigned`] but then sign-extends the result.
pub fn read_little_endian_signed(bytes: &[u8], offset: usize, length: usize) -> i64 {
    sign_extend(length, read_little_endian_unsigned(bytes, offset, length))
}

/// Writes a little-endian CAN value with specified length and offset into a byte buffer.
///
/// See [`read_little_endian_unsigned`] for some examples of the expected mapping between values and the byte buffer.
pub fn write_little_endian_unsigned(bytes: &mut [u8], offset: usize, length: usize, value: u64) {
    assert!(bytes.len() <= 8, "maximum supported input is 8 bytes");
    assert!(offset + length <= 64, "maximum supported input is 8 bytes");

    let mask = u64::MAX >> (64 - length);

    let mut original = u64_from_le_slice(bytes);
    original &= !(mask << offset);
    original |= (value & mask) << offset;
    let result = original.to_le_bytes();

    bytes.copy_from_slice(&result[..bytes.len()]);
}

/// Writes a little-endian CAN value with specified length and offset into a byte buffer.
pub fn write_little_endian_signed(bytes: &mut [u8], offset: usize, length: usize, value: i64) {
    write_little_endian_unsigned(bytes, offset, length, value as u64)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use test_case::test_case;

    #[test_case("00007028efbfb523", 0, 64, 0x7028EFBFB523)]
    #[test_case("0759bb23302c7b00", 0, 32, 0x759BB23)]
    #[test_case("0759bb23302c7b00", 32, 16, 0x302C)]
    #[test_case("0759bb23302c7b00", 48, 8, 0x7B)]
    fn test_read_big_endian_unsigned(hex: &str, offset: usize, length: usize, expected: u64) {
        let bytes = hex::decode(hex).unwrap();
        let actual = super::read_big_endian_unsigned(&bytes, offset, length);
        assert_eq!(actual, expected);
    }

    #[test_case("00007028efbfb523", 0, 64, 0x7028EFBFB523)]
    #[test_case("0759bb23302c7b00", 0, 32, 0x759BB23)]
    #[test_case("0759bb23302c7b00", 32, 16, 0x302C)]
    #[test_case("0759bb23302c7b00", 48, 8, 0x7B)]
    #[test_case("b0b44a55870181f7", 7, 8, 90)] // s8big
    #[test_case("b0b44a55870181f7", 32, 3, -4)] // s3big
    #[test_case("b0b44a55870181f7", 47, 10, -253)] // s10big
    #[test_case("b0b44a55870181f7", 57, 7, -9)] // s7big
    fn test_read_big_endian_signed(hex: &str, offset: usize, length: usize, expected: i64) {
        let bytes = hex::decode(hex).unwrap();
        let actual = super::read_big_endian_signed(&bytes, offset, length);
        assert_eq!(actual, expected);
    }

    #[test_case("23b5bfef28700000", 0, 64, 0x7028EFBFB523)]
    #[test_case("23bb59072c307b00", 0, 32, 0x759BB23)]
    #[test_case("23bb59072c307b00", 32, 16, 0x302C)]
    #[test_case("23bb59072c307b00", 48, 8, 0x7B)]
    #[test_case("4871060000000000", 2, 17, 0b1_1001_1100_0101_0010)]
    fn test_read_little_endian_unsigned(hex: &str, offset: usize, length: usize, expected: u64) {
        let bytes = hex::decode(hex).unwrap();
        let actual = super::read_little_endian_unsigned(&bytes, offset, length);
        assert_eq!(actual, expected);
    }

    #[test_case("23b5bfef28700000", 0, 64, 0x7028EFBFB523)]
    #[test_case("23bb59072c307b00", 0, 32, 0x759BB23)]
    #[test_case("23bb59072c307b00", 32, 16, 0x302C)]
    #[test_case("23bb59072c307b00", 48, 8, 0x7B)]
    #[test_case("dd4a4010d78fffff", 0, 64, -0x7028efbfb523)]
    #[test_case("dd44a6f8d4cf8500", 0, 32, -0x759bb23)]
    #[test_case("dd44a6f8d4cf8500", 32, 16, -0x302c)]
    #[test_case("dd44a6f8d4cf8500", 48, 8, -0x7b)]
    #[test_case("b0b44a55870181f7", 1, 7, -40)] // s7
    #[test_case("b0b44a55870181f7", 17, 9, 165)] // s9
    #[test_case("b0b44a55870181f7", 26, 8, -43)] // s8
    #[test_case("b0b44a55870181f7", 34, 3, 1)] // s3
    fn test_read_little_endian_signed(hex: &str, offset: usize, length: usize, expected: i64) {
        let bytes = hex::decode(hex).unwrap();
        let actual = super::read_little_endian_signed(&bytes, offset, length);
        assert_eq!(actual, expected);
    }

    #[test_case("00007028efbfb523", &[(0, 64, 0x7028efbfb523)])]
    #[test_case("0759bb23302c7b00", &[(0, 32, 0x759bb23), (32, 16, 0x302c), (48, 8, 0x7b)])]
    fn test_write_big_endian_unsigned(expected: &str, values: &[(usize, usize, u64)]) {
        let expected = hex::decode(expected).unwrap();
        let mut bytes = [0; 8];
        for &(offset, length, value) in values {
            super::write_big_endian_unsigned(&mut bytes, offset, length, value);
        }
        assert_eq!(
            &bytes[..],
            &expected[..],
            "{:?}",
            (hex::encode(bytes), hex::encode(&expected))
        );
    }

    #[test_case("00007028efbfb523", &[(0, 64, 0x7028efbfb523)])]
    #[test_case("0759bb23302c7b00", &[(0, 32, 0x759bb23), (32, 16, 0x302c), (48, 8, 0x7b)])]
    #[test_case("00b4000000000000", &[(7, 8, 90)])]
    #[test_case("0000000080000000", &[(32, 3, -4)])]
    #[test_case("0000000000018180", &[(47, 10, -253)])]
    #[test_case("0000000000000077", &[(57, 7, -9)])]
    fn test_write_big_endian_signed(expected: &str, values: &[(usize, usize, i64)]) {
        let expected = hex::decode(expected).unwrap();
        let mut bytes = [0; 8];
        for &(offset, length, value) in values {
            super::write_big_endian_signed(&mut bytes, offset, length, value);
        }
        assert_eq!(
            &bytes[..],
            &expected[..],
            "{:?}",
            (hex::encode(bytes), hex::encode(&expected))
        );
    }

    #[test_case("23b5bfef28700000", &[(0, 64, 0x7028efbfb523)])]
    #[test_case("23bb59072c307b00", &[(0, 32, 0x759bb23), (32, 16, 0x302c), (48, 8, 0x7b)])]
    #[test_case("4871060000000000", &[(2, 17, 0b1_1001_1100_0101_0010)])]
    fn test_write_little_endian_unsigned(expected: &str, values: &[(usize, usize, u64)]) {
        let expected = hex::decode(expected).unwrap();
        let mut bytes = [0; 8];
        for &(offset, length, value) in values {
            super::write_little_endian_unsigned(&mut bytes, offset, length, value);
        }
        assert_eq!(
            &bytes[..],
            &expected[..],
            "{:?}",
            (hex::encode(bytes), hex::encode(&expected))
        );
    }

    #[test_case("23b5bfef28700000", &[(0, 64, 0x7028efbfb523)])]
    #[test_case("23bb59072c307b00", &[(0, 32, 0x759bb23), (32, 16, 0x302c), (48, 8, 0x7b)])]
    #[test_case("dd4a4010d78fffff", &[(0, 64, -0x7028efbfb523)])]
    #[test_case("dd44a6f8d4cf8500", &[(0, 32, -0x759bb23), (32, 16, -0x302c), (48, 8, -0x7b)])]
    #[test_case("b000000000000000", &[(1, 7, -40)])]
    #[test_case("00004a0100000000", &[(17, 9, 165)])]
    #[test_case("0000005403000000", &[(26, 8, -43)])]
    #[test_case("0000000004000000", &[(34, 3, 1)])]
    fn test_write_little_endian_signed(expected: &str, values: &[(usize, usize, i64)]) {
        let expected = hex::decode(expected).unwrap();
        let mut bytes = [0; 8];
        for &(offset, length, value) in values {
            super::write_little_endian_signed(&mut bytes, offset, length, value);
        }
        assert_eq!(
            &bytes[..],
            &expected[..],
            "{:?}",
            (hex::encode(bytes), hex::encode(&expected))
        );
    }

    #[test]
    fn test_write_mixed() {
        let expected = hex::decode("b0b44a55870181f7").unwrap();
        let mut bytes = [0; 8];

        super::write_little_endian_signed(&mut bytes, 1, 7, -40);
        super::write_big_endian_signed(&mut bytes, 7, 8, 90);
        super::write_little_endian_signed(&mut bytes, 17, 9, 165);
        super::write_little_endian_signed(&mut bytes, 26, 8, -43);
        super::write_big_endian_signed(&mut bytes, 32, 3, -4);
        super::write_little_endian_signed(&mut bytes, 34, 3, 1);
        super::write_big_endian_signed(&mut bytes, 47, 10, -253);
        super::write_big_endian_signed(&mut bytes, 57, 7, -9);

        assert_eq!(
            &bytes[..],
            &expected[..],
            "{:?}",
            (hex::encode(bytes), hex::encode(&expected))
        );
    }
}
