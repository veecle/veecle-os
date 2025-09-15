#![expect(missing_docs)]

use veecle_os_data_support_can::{CanDecodeError, Frame, generate};

#[test]
fn simple_validation() {
    generate!(
        mod generated {
            #![dbc = r#"
                VERSION ""

                NS_ :

                BO_ 1 SomeMessage: 8 Vector__XXX
                 SG_ Signal1 : 0|16@1+ (1,0) [0|0] "" Vector__XXX
                 SG_ Signal2 : 16|32@1+ (1,0) [0|0] "" Vector__XXX
                 SG_ CRC : 52|8@1+ (1,0) [0|0] "" Vector__XXX
            "#]

            use veecle_os_data_support_can::CanDecodeError;

            impl SomeMessage {
                #[validate_frame]
                fn check_crc(data: &[u8; Self::FRAME_LENGTH]) -> Result<(), CanDecodeError> {
                    let expected = data[7];
                    let actual = data[0..7].iter().fold(0, |acc, byte| acc ^ byte);
                    if expected != actual {
                        return Err(CanDecodeError::invalid("wrong crc"));
                    }
                    Ok(())
                }
            }
        }
    );

    let bytes: [u8; 8] = hex::decode("0000000000000005").unwrap().try_into().unwrap();
    let frame = Frame::new(generated::SomeMessage::FRAME_ID, bytes);

    let error = generated::SomeMessage::try_from(frame).unwrap_err();
    assert!(
        matches!(
            error,
            CanDecodeError::Invalid {
                message: "wrong crc"
            },
        ),
        "expected right, but got left {}",
        pretty_assertions::Comparison::new(
            &error,
            &CanDecodeError::Invalid {
                message: "wrong crc"
            },
        ),
    );
}
