#![allow(missing_docs)] // This is a test crate.

#[macro_use]
mod macros;

// Using `cantools` python package to generate test cases:
//
// ```python
// >>> import cantools
// >>> db = cantools.database.load_file("CSS-Electronics-SAE-J1939-DEMO.dbc")
// >>> eec1 = db.get_message_by_name('EEC1')
// >>> eec1.encode({'EngineSpeed': 2.5}).hex()
// '0000001400000000'
// ```

make_tests! {
    j1939 {
        dbc: r#"
            VERSION ""

            NS_ :

            BO_ 2364540158 EEC1: 8 Vector__XXX
             SG_ EngineSpeed : 24|16@1+ (0.125,0) [0|8031.875] "rpm" Vector__XXX

            BO_ 2566844926 CCVS1: 8 Vector__XXX
             SG_ WheelBasedVehicleSpeed : 8|16@1+ (0.00390625,0) [0|250.996] "km/h" Vector__XXX
        "#,

        eec1 {
            "0000000000000000" => Eec1 {
                engine_speed: EngineSpeed(0.0),
            }

            "0000001400000000" => Eec1 {
                engine_speed: EngineSpeed(2.5),
            }

            "000000fffa000000" => Eec1 {
                engine_speed: EngineSpeed(8031.875),
            }

            errors {
                Eec1 {
                    "000000fffb000000": OutOfRange { name: "EngineSpeed", ty: "f64", message: "out of range 0.0..=8031.875" }
                    "000000ffff000000": OutOfRange { name: "EngineSpeed", ty: "f64", message: "out of range 0.0..=8031.875" }
                }
            }
        }

        ccvs1 {
            "0024030000000000" => Ccvs1 {
                wheel_based_vehicle_speed: WheelBasedVehicleSpeed(3.140625),
            }
        }
    }

    floating_point {
        dbc: r#"
            VERSION ""

            NS_ :

            BU_: TestNode

            BO_ 1024 Message1: 8 TestNode
             SG_ Signal1 : 0|64@1- (1,0) [0|0] "" Vector__XXX

            BO_ 1025 Message2: 8 TestNode
             SG_ Signal2 : 32|32@1- (1,0) [0|0] "" Vector__XXX
             SG_ Signal1 : 0|32@1- (1,0) [0|0] "" Vector__XXX

            SIG_VALTYPE_ 1024 Signal1 : 2;
            SIG_VALTYPE_ 1025 Signal2 : 1;
            SIG_VALTYPE_ 1025 Signal1 : 1;
        "#,

        message1 {
            "75931804562e60c0" => Message1 {
                signal1: Signal1(-129.448),
            }
        }

        message2 {
            "0080014324b29649" => Message2 {
                signal1: Signal1(129.5),
                signal2: Signal2(1234500.5),
            }
        }
    }

    integer {
        dbc: r#"
            VERSION ""

            NS_ :

            BO_ 1024 Message1: 8 Vector__XXX
             SG_ u64 : 0|64@1+ (1,0) [0|0] "" Vector__XXX

            BO_ 1025 Message2: 8 Vector_XXX
             SG_ u32 : 0|32@1+ (1,0) [0|0] "" Vector__XXX
             SG_ u16 : 32|16@1+ (1,0) [0|0] "" Vector__XXX
             SG_ u8 : 48|8@1+ (1,0) [0|0] "" Vector__XXX

            BO_ 1026 Message3: 8 Vector__XXX
             SG_ i64 : 0|64@1- (1,0) [0|0] "" Vector__XXX

            BO_ 1027 Message4: 8 Vector_XXX
             SG_ i32 : 0|32@1- (1,0) [0|0] "" Vector__XXX
             SG_ i16 : 32|16@1- (1,0) [0|0] "" Vector__XXX
             SG_ i8 : 48|8@1- (1,0) [0|0] "" Vector__XXX

            BO_ 2024 Message1BE: 8 Vector__XXX
             SG_ u64 : 7|64@0+ (1,0) [0|200000000000000] "" Vector__XXX

            BO_ 2025 Message2BE: 8 Vector_XXX
             SG_ u32 : 7|32@0+ (1,0) [0|4294967295] "" Vector__XXX
             SG_ u16 : 39|16@0+ (1,0) [0|65535] "" Vector__XXX
             SG_ u8 : 55|8@0+ (1,0) [0|255] "" Vector__XXX

            BO_ 2026 Message3BE: 8 Vector__XXX
             SG_ i64 : 7|64@0- (1,0) [-200000000000000|200000000000000] "" Vector__XXX

            BO_ 2027 Message4BE: 8 Vector_XXX
             SG_ i32 : 7|32@0- (1,0) [-2147483648|2147483647] "" Vector__XXX
             SG_ i16 : 39|16@0- (1,0) [-32768|32767] "" Vector__XXX
             SG_ i8 : 55|8@0- (1,0) [-128|127] "" Vector__XXX

            SIG_VALTYPE_ 1024 u64 : 0;
            SIG_VALTYPE_ 1025 u32 : 0;
            SIG_VALTYPE_ 1025 u16 : 0;
            SIG_VALTYPE_ 1025 u8 : 0;
            SIG_VALTYPE_ 1026 i64 : 0;
            SIG_VALTYPE_ 1027 i32 : 0;
            SIG_VALTYPE_ 1027 i16 : 0;
            SIG_VALTYPE_ 1027 i8 : 0;

            SIG_VALTYPE_ 2024 u64 : 0;
            SIG_VALTYPE_ 2025 u32 : 0;
            SIG_VALTYPE_ 2025 u16 : 0;
            SIG_VALTYPE_ 2025 u8 : 0;
            SIG_VALTYPE_ 2026 i64 : 0;
            SIG_VALTYPE_ 2027 i32 : 0;
            SIG_VALTYPE_ 2027 i16 : 0;
            SIG_VALTYPE_ 2027 i8 : 0;
        "#,

        message1 {
            "23b5bfef28700000" => Message1 {
                u64: U64(123321123321123),
            }
        }

        message2 {
            "23bb59072c307b00" => Message2 {
                u32: U32(123321123),
                u16: U16(12332),
                u8: U8(123),
            }
        }

        message3 {
            "23b5bfef28700000" => Message3 {
                i64: I64(123321123321123),
            }

            "dd4a4010d78fffff" => Message3 {
                i64: I64(-123321123321123),
            }
        }

        message4 {
            "23bb59072c307b00" => Message4 {
                i32: I32(123321123),
                i16: I16(12332),
                i8: I8(123),
            }

            "dd44a6f8d4cf8500" => Message4 {
                i32: I32(-123321123),
                i16: I16(-12332),
                i8: I8(-123),
            }
        }

        message1_be {
            "00007028efbfb523" => Message1Be {
                u64: U64(123321123321123),
            }
        }

        message2_be {
            "0759bb23302c7b00" => Message2Be {
                u32: U32(123321123),
                u16: U16(12332),
                u8: U8(123),
            }
        }

        message3_be {
            "00007028efbfb523" => Message3Be {
                i64: I64(123321123321123),
            }

            "ffff8fd710404add" => Message3Be {
                i64: I64(-123321123321123),
            }
        }

        message4_be {
            "0759bb23302c7b00" => Message4Be {
                i32: I32(123321123),
                i16: I16(12332),
                i8: I8(123),
            }

            "f8a644ddcfd48500" => Message4Be {
                i32: I32(-123321123),
                i16: I16(-12332),
                i8: I8(-123),
            }
        }
    }

    integer_be {
        dbc: r#"
            VERSION ""

            NS_ :

            BO_ 2024 Message1BE: 8 Vector__XXX
             SG_ u64 : 7|64@0+ (1,0) [0|200000000000000] "" Vector__XXX

            BO_ 2025 Message2BE: 8 Vector_XXX
             SG_ u32 : 7|32@0+ (1,0) [0|4294967295] "" Vector__XXX
             SG_ u16 : 39|16@0+ (1,0) [0|65535] "" Vector__XXX
             SG_ u8 : 55|8@0+ (1,0) [0|255] "" Vector__XXX

            BO_ 2026 Message3BE: 8 Vector__XXX
             SG_ i64 : 7|64@0- (1,0) [-200000000000000|200000000000000] "" Vector__XXX

            BO_ 2027 Message4BE: 8 Vector_XXX
             SG_ i32 : 7|32@0- (1,0) [-2147483648|2147483647] "" Vector__XXX
             SG_ i16 : 39|16@0- (1,0) [-32768|32767] "" Vector__XXX
             SG_ i8 : 55|8@0- (1,0) [-128|127] "" Vector__XXX

            BO_ 10 Message378910: 8 Vector__XXX
             SG_ s3big : 39|3@0- (1,0) [0|0] "" Vector__XXX
             SG_ s3 : 34|3@1- (1,0) [0|0] "" Vector__XXX
             SG_ s10big : 40|10@0- (1,0) [0|0] "" Vector__XXX
             SG_ s8big : 0|8@0- (1,0) [0|0] "" Vector__XXX
             SG_ s7big : 62|7@0- (1,0) [0|0] "" Vector__XXX
             SG_ s9 : 17|9@1- (1,0) [0|0] "" Vector__XXX
             SG_ s8 : 26|8@1- (1,0) [0|0] "" Vector__XXX
             SG_ s7 : 1|7@1- (1,0) [0|0] "" Vector__XXX

            SIG_VALTYPE_ 2024 u64 : 0;
            SIG_VALTYPE_ 2025 u32 : 0;
            SIG_VALTYPE_ 2025 u16 : 0;
            SIG_VALTYPE_ 2025 u8 : 0;
            SIG_VALTYPE_ 2026 i64 : 0;
            SIG_VALTYPE_ 2027 i32 : 0;
            SIG_VALTYPE_ 2027 i16 : 0;
            SIG_VALTYPE_ 2027 i8 : 0;
        "#,

        message1_be {
            "00007028efbfb523" => Message1Be {
                u64: U64(123321123321123),
            }
        }

        message2_be {
            "0759bb23302c7b00" => Message2Be {
                u32: U32(123321123),
                u16: U16(12332),
                u8: U8(123),
            }
        }

        message3_be {
            "00007028efbfb523" => Message3Be {
                i64: I64(123321123321123),
            }

            "ffff8fd710404add" => Message3Be {
                i64: I64(-123321123321123),
            }
        }

        message4_be {
            "0759bb23302c7b00" => Message4Be {
                i32: I32(123321123),
                i16: I16(12332),
                i8: I8(123),
            }

            "f8a644ddcfd48500" => Message4Be {
                i32: I32(-123321123),
                i16: I16(-12332),
                i8: I8(-123),
            }
        }

        message378910 {
            "b0b44a55870181f7" => Message378910 {
                s7: S7(-40),
                s8big: S8big(90),
                s9: S9(165),
                s8: S8(-43),
                s3big: S3big(-4),
                s3: S3(1),
                s10big: S10big(-253),
                s7big: S7big(-9),
            }
        }
    }

    signed_message {
        dbc: r#"
            VERSION ""

            NS_ :

            BO_ 8 Message631: 8 Vector__XXX
             SG_ s63 : 1|63@1- (1,0) [0|0] "" Vector__XXX

            BO_ 6 Message63: 8 Vector__XXX
             SG_ s63 : 0|63@1- (1,0) [0|0] "" Vector__XXX

            BO_ 2 Message64: 8 Vector__XXX
             SG_ s64 : 0|64@1- (1,0) [0|0] "" Vector__XXX

            BO_ 1 Message33: 8 Vector__XXX
             SG_ s33 : 0|33@1- (1,0) [0|0] "" Vector__XXX

            BO_ 0 Message32: 8 Vector__XXX
             SG_ s32 : 0|32@1- (1,0) [0|0] "" Vector__XXX
        "#,

        message631 {
             "0a00000000000000" => Message631 {
                 s63: S63(5),
             }
             "f6ffffffffffffff" => Message631 {
                 s63: S63(-5),
             }
        }

        message63 {
             "0500000000000000" => Message63 {
                 s63: S63(5),
             }
             "fbffffffffffff7f" => Message63 {
                 s63: S63(-5),
             }
        }

        message64 {
             "0500000000000000" => Message64 {
                 s64: S64(5),
             }
             "fbffffffffffffff" => Message64 {
                 s64: S64(-5),
             }
        }

        message33 {
             "0500000000000000" => Message33 {
                 s33: S33(5),
             }
             "fbffffff01000000" => Message33 {
                 s33: S33(-5),
             }
        }

        message32 {
             "0500000000000000" => Message32 {
                 s32: S32(5),
             }
             "fbffffff00000000" => Message32 {
                 s32: S32(-5),
             }
        }
    }

    nonbinary {
        dbc: r#"
            VERSION ""

            NS_ :

            BO_ 1 Message: 1 Vector__XXX
             SG_ value : 0|8@1+ (0.1,0) [0|0] "" Vector__XXX
        "#,

        message {
            "00" => Message {
                value: Value(0.0),
            }
            "01" => Message {
                value: Value(0.1),
            }
            "51" => Message {
                value: Value(8.1),
            }
            "fe" => Message {
                value: Value(25.4),
            }
            "ff" => Message {
                value: Value(25.5),
            }
        }
    }

    choices {
        dbc: r##"
            VERSION ""

            NS_ :

            BO_ 0 Foo: 1 Vector__XXX
             SG_ Foo : 0|8@1- (1,0) [-128|127] "" Vector__XXX

            VAL_ 0 Foo 6 "reserved" 5 "reserved" 4 "unused 2" 3 "unused" 2 "unused" 1 "#%=*ä'" 0 "With space" -5 "A negative value" ;
        "##,

        foo {
            "00" => Foo {
                foo: Foo::With_space,
            }
            "01" => Foo {
                foo: Foo::____ä_,
            }
            "02" => Foo {
                foo: Foo::unused,
            }
            "03" => Foo {
                foo: Foo::unused_,
            }
            "04" => Foo {
                foo: Foo::unused_2,
            }
            "05" => Foo {
                foo: Foo::reserved,
            }
            "06" => Foo {
                foo: Foo::reserved_,
            }
            "07" => Foo {
                foo: Foo(7),
            }
            "fb" => Foo {
                foo: Foo::A_negative_value,
            }
        }
    }
}
