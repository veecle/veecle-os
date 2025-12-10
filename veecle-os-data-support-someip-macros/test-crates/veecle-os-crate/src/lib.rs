//! Test that derive macros can be used while depending only on `veecle-os`.

use veecle_os::data_support::someip;

#[derive(Debug, Clone, Copy, PartialEq, Eq, someip::parse::Parse, someip::serialize::Serialize)]
pub struct TestStruct {
    inner: u8,
}

#[cfg(test)]
mod tests {
    use veecle_os::data_support::someip::parse::ParseExt;
    use veecle_os::data_support::someip::serialize::SerializeExt;

    use crate::TestStruct;

    #[test]
    fn roundtrip() {
        let mut buffer = [0; 512];

        let value = TestStruct { inner: 5 };
        let written = value.serialize(&mut buffer).unwrap();
        assert_eq!(TestStruct::parse(&buffer[..written]).unwrap(), value);
    }
}
