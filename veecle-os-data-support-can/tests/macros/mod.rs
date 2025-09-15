//! Generates a set of serialization tests.
//!
//! Expects input like
//!
//! ```ignore
//! make_tests! {
//!     some_mod_name {                             // Arbitrary module name for the tests associated with this DBC file.
//!         dbc: "…A CAN-DBC file contents…",       // The DBC file the tests in this module use generated types from.
//!
//!         some_message {                          // The generated module name of a message to test (can be multiple).
//!             "00000000" => SomeMessage {         // The expected serialized hex value mapped to the deserialized value to test against (can be multiple).
//!                 some_signal: SomeSignal(0),     // The fields of the deserialized value, field name mapped to signal type and value (can be multiple).
//!             }
//!
//!             errors {                            // A special section to put values that should error while deserializing.
//!                 SomeMessage {                   // The message type name.
//!                     "ffffffff"                  // A serialized hex value that should error (can be multiple).
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```
macro_rules! make_tests {
    // Start with matching just a series of the module names containing separate DBC files,
    // leaving the contents opaque here.
    ($(
        $db_name:ident {
            $($db_tt:tt)*
        }
    )+) => {
        // Use the module names twice,
        // once to generate the module defining the test cases,
        $( make_tests!(@db $db_name { $($db_tt)* }); )*
        // then again to generate a main function consuming all those modules.
        make_tests!(@main $($db_name)*);
    };

    // Matches the full form of a single test module and generates a module containing the
    // generated code for the DBC file and a function to get all the test cases for it.
    (@db $db_name:ident {
        dbc: $dbc:literal,
        $(
            $module:ident {
                $(
                    $hex:literal => $message_ty:ident { $($fields:tt)* }
                )*

                $(
                    errors {
                        $(
                            $error_ty:ident {
                                $($error_hex:literal: $expected_error_variant:ident { $($expected_error_fields:tt)* })*
                            }
                        )*
                    }
                )?
            }
        )+
    }) => {
        mod $db_name {
            use libtest_mimic::Trial;

            veecle_os_data_support_can::generate!(mod generated { #![dbc = $dbc] });

            pub fn trials() -> Vec<Vec<Trial>> {
                Vec::from([

                    // For the dbc create a test that runs codegen at runtime for code coverage to see.
                    #[cfg(not(miri))] // It's way too slow under Miri.
                    vec![
                        Trial::test(format!("{}::codegen", stringify!($db_name)), move || {
                            use veecle_os_data_support_can_codegen::{Options, Generator};

                            let options = Options {
                                veecle_os_runtime: syn::parse_str("veecle_os_runtime")?,
                                veecle_os_data_support_can: syn::parse_str("veecle_os_data_support_can")?,
                                arbitrary: Some(veecle_os_data_support_can_codegen::ArbitraryOptions {
                                    path: syn::parse_str("arbitrary")?,
                                    cfg: Some(syn::parse_str(r#"all()"#)?),
                                }),
                                serde: syn::parse_str("serde")?,
                                message_frame_validations: Box::new(|_| None),
                            };

                            Generator::new(stringify!($db_name), options, $dbc).into_string();

                            Ok(())
                        }),
                    ],

                    // For each message module, and each hex/value pair within that module, create
                    // a serialization and deserialization test case.
                    $($(
                        {
                            use self::generated::{$module, $message_ty};

                            let value = make_tests!(@message $module::$message_ty { $($fields)* });

                            let bytes: [u8; $message_ty::FRAME_LENGTH] = hex::decode($hex)
                                .unwrap()
                                .try_into()
                                .unwrap();

                            let base = format!(
                                "{}::{}",
                                stringify!($db_name),
                                stringify!($module),
                            );

                            vec![
                                Trial::test(format!("{base}::serialize({value:?})"), move || {
                                    let serialized = hex::encode(veecle_os_data_support_can::Frame::from(&value).data());
                                    pretty_assertions::assert_eq!($hex, serialized, "expected right, but got left");
                                    Ok(())
                                }),

                                Trial::test(format!("{base}::deserialize({value:?})"), move || {
                                    let frame = veecle_os_data_support_can::Frame::new($message_ty::FRAME_ID, bytes);
                                    let deserialized = $message_ty::try_from(&frame).unwrap();
                                    pretty_assertions::assert_eq!(value, deserialized, "expected right, but got left");
                                    Ok(())
                                }),
                            ]
                        },
                    )*)+

                    // For each message module that has an `errors` section, generate a
                    // deserialization test case that it does actually error.
                    $($($($(
                        {
                            use self::generated::$error_ty;
                            let error_hex = $error_hex;
                            let bytes: [u8; $error_ty::FRAME_LENGTH] = hex::decode(error_hex)
                                .unwrap()
                                .try_into()
                                .unwrap();

                            let base = format!(
                                "{}::{}::errors",
                                stringify!($db_name),
                                stringify!($module),
                            );

                            vec![
                                Trial::test(format!("{base}::deserialize({error_hex})"), move || {
                                    let frame = veecle_os_data_support_can::Frame::new($error_ty::FRAME_ID, bytes);
                                    let error = $error_ty::try_from(frame);
                                    assert!(
                                        matches!(
                                            error.as_ref().unwrap_err(),
                                            veecle_os_data_support_can::CanDecodeError::$expected_error_variant { $($expected_error_fields)* },
                                        ),
                                        "expected right, but got left\n\n{}",
                                        pretty_assertions::Comparison::new(
                                            &error.as_ref().unwrap_err(),
                                            &veecle_os_data_support_can::CanDecodeError::$expected_error_variant { $($expected_error_fields)* },
                                        ),
                                    );
                                    Ok(())
                                }),
                            ]
                        },
                    )*)*)?)+

                ])
            }
        }
    };

    // We use a TT muncher and push-down accumulation in the next few arms to allow parsing different syntaxes
    // per-field, and accumulating the resulting list of `field: expr,` fragments to put in the constructor at the end.
    (@message $module:ident :: $message_ty:ident {
        $($fields:tt)*
    }) => {
        make_tests! {
            @next field $module :: $message_ty
            [ $($fields)* ]
            []
        }
    };

    (
        @next field $module:ident :: $message_ty:ident
        [
            $field:ident : $field_ty:ident ( $value:expr ),
            $($rest:tt)*
        ]
        [ $($fields:tt)* ]
    ) => {
        make_tests! {
            @next field $module :: $message_ty
            [ $($rest)* ]
            [
                $($fields)*
                $field: $module::$field_ty::try_from($value).expect(concat!(
                    "failed to parse expected value ", stringify!($value), " for ",
                    stringify!($db_name), "::", stringify!($module), "::", stringify!($field_ty),
                )),
            ]
        }
    };

    (
        @next field $module:ident :: $message_ty:ident
        [
            $field:ident : $field_ty:ident :: $choice:ident,
            $($rest:tt)*
        ]
        [ $($fields:tt)* ]
    ) => {
        make_tests! {
            @next field $module :: $message_ty
            [ $($rest)* ]
            [
                $($fields)*
                $field: $module::$field_ty::$choice,
            ]
        }
    };

    (@next field $module:ident :: $message_ty:ident [] [ $($fields:tt)* ]) => {
        $message_ty {
            $($fields)*
        }
    };

    (@main $($db_name:ident)*) => {
        fn main() -> std::process::ExitCode {
            let args = libtest_mimic::Arguments::from_args();

            let trials = [
                $(
                    $db_name::trials(),
                )*
            ].into_iter().flatten().flatten().collect();

            libtest_mimic::run(&args, trials).exit_code()
        }
    };
}
