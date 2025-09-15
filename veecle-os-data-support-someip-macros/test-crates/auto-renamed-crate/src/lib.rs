//! Test that derive macros can be used while depending on a renamed `veecle_os_data_support_someip`.

use my_veecle_os_data_support_someip::parse::Parse;
use my_veecle_os_data_support_someip::serialize::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Parse, Serialize)]
pub struct TestStruct {
    inner: u8,
}

#[cfg(test)]
mod tests {
    use my_veecle_os_data_support_someip::parse::ParseExt;
    use my_veecle_os_data_support_someip::serialize::SerializeExt;

    use crate::TestStruct;

    #[test]
    fn roundtrip() {
        let mut buffer = [0; 512];

        let value = TestStruct { inner: 5 };
        let serialized = value.serialize(&mut buffer).unwrap();
        assert_eq!(TestStruct::parse(serialized).unwrap(), value);
    }
}
