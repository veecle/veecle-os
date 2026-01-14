#![expect(missing_docs)]

//! This test ensures that slots are distinguished by the `Storable` type rather than by the
//! `DataType`. Changes to how slots are created or assigned could introduce bugs where data from
//! one Storable overwrites another.

use veecle_os_runtime::{Reader, Storable, Writer};

#[derive(Debug, PartialEq, Clone)]
pub struct Temperature;

impl Storable for Temperature {
    type DataType = f32;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Humidity;

impl Storable for Humidity {
    type DataType = f32;
}

#[test]
fn test_two_storables_with_same_datatype() {
    veecle_os_test::block_on_future(veecle_os_test::execute! {
        actors: [],
        validation: async |
            mut temp_reader: Reader<'a, Temperature>,
            mut temp_writer: Writer<'a, Temperature>,
            mut humidity_reader: Reader<'a, Humidity>,
            mut humidity_writer: Writer<'a, Humidity>,
        | {
            temp_writer.write(20.5).await;
            humidity_writer.write(65.0).await;

            assert_eq!(temp_reader.read_cloned(), Some(20.5));
            assert_eq!(humidity_reader.read_cloned(), Some(65.0));

            temp_writer.write(21.0).await;
            humidity_writer.write(70.0).await;

            assert_eq!(temp_reader.read_cloned(), Some(21.0));
            assert_eq!(humidity_reader.read_cloned(), Some(70.0));
        }
    });
}
