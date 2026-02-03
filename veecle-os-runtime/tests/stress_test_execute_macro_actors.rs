#![recursion_limit = "256"]
#![allow(unused_variables, non_snake_case, clippy::too_many_arguments)]

//! For stress testing `veecle_os_runtime::execute!` with a large number of actors (but few data types).
//!
//! Performance can be measured with the following commands:
//!
//! ```
//! cargo +nightly rustc --test stress_test_execute_macro_actors -- -Ztime-passes
//! nix develop .#nightly --command cargo rustc --test stress_test_execute_macro_actors -- -Ztime-passes
//! ```

macro_rules! make_test {
    ($($ident:ident)*) => {
        #[derive(Copy, Clone, Debug, veecle_os_runtime::Storable)]
        struct Data;

        $(
            #[veecle_os_runtime::actor]
            async fn $ident(
                reader: veecle_os_runtime::single_writer::Reader<'_, Data>,
            ) -> veecle_os_runtime::Never {
                panic!("test completed");
            }
        )*

        #[veecle_os_runtime::actor]
        async fn writer(
            writer: veecle_os_runtime::single_writer::Writer<'_, Data>,
        ) -> veecle_os_runtime::Never {
            panic!("test completed");
        }

        #[test]
        #[should_panic(expected = "test completed")]
        fn stress_test_execute_macro_store() {
            futures::executor::block_on(
                veecle_os_runtime::execute! {
                    actors: [
                        $(
                            $ident,
                        )*
                        Writer,
                    ],
                }
            );
        }
    }
}

make_test!(A B C D E F G H I J K L M N O P Q R S T U V W X Y Z);
