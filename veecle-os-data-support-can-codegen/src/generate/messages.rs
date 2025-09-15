use std::collections::HashSet;

use anyhow::{Result, bail, ensure};
use can_dbc::{Comment, DBC, Message, Signal, SignalExtendedValueType, ValueType};
use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};

struct GeneratedSignal {
    name: syn::Ident,
    snake_case_name: syn::Ident,
    definition: TokenStream,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum FloatOrInt {
    Float(f64),
    Int(i128),
}

impl FloatOrInt {
    fn as_int(self) -> Option<i128> {
        match self {
            FloatOrInt::Float(_) => None,
            FloatOrInt::Int(value) => Some(value),
        }
    }

    fn is_int(self) -> bool {
        self.as_int().is_some()
    }

    fn make_int_lit(self) -> syn::Lit {
        let value = match self {
            Self::Float(value) => value.round() as i128,
            Self::Int(value) => value,
        };
        syn::Lit::Int(proc_macro2::Literal::i128_unsuffixed(value).into())
    }

    fn make_f32_lit(self) -> syn::Lit {
        let value = match self {
            Self::Float(value) => value as f32,
            Self::Int(value) => value as f32,
        };
        syn::Lit::Float(proc_macro2::Literal::f32_unsuffixed(value).into())
    }

    fn make_f64_lit(self) -> syn::Lit {
        let value = match self {
            Self::Float(value) => value,
            Self::Int(value) => value as f64,
        };
        syn::Lit::Float(proc_macro2::Literal::f64_unsuffixed(value).into())
    }

    fn make_f32_bits_lit(self) -> syn::Lit {
        let value = match self {
            Self::Float(value) => value as f32,
            Self::Int(value) => value as f32,
        };
        syn::Lit::Int(proc_macro2::Literal::u32_unsuffixed(value.to_bits()).into())
    }

    fn make_f64_bits_lit(self) -> syn::Lit {
        let value = match self {
            Self::Float(value) => value,
            Self::Int(value) => value as f64,
        };
        syn::Lit::Int(proc_macro2::Literal::u64_unsuffixed(value.to_bits()).into())
    }
}

impl From<f32> for FloatOrInt {
    fn from(value: f32) -> Self {
        Self::from(value as f64)
    }
}

impl From<f64> for FloatOrInt {
    fn from(value: f64) -> Self {
        if value.fract() == 0.0 {
            Self::Int(value as i128)
        } else {
            Self::Float(value)
        }
    }
}

impl From<&f64> for FloatOrInt {
    fn from(&value: &f64) -> Self {
        Self::from(value)
    }
}

impl From<i128> for FloatOrInt {
    fn from(value: i128) -> Self {
        Self::Int(value)
    }
}

impl core::ops::Add for FloatOrInt {
    type Output = Self;

    fn add(self, right: Self) -> Self::Output {
        match (self, right) {
            (Self::Float(left), Self::Float(right)) => Self::Float(left + right),
            (Self::Float(left), Self::Int(right)) => Self::Float(left + right as f64),
            (Self::Int(left), Self::Float(right)) => Self::Float(left as f64 + right),
            (Self::Int(left), Self::Int(right)) => Self::Int(left + right),
        }
    }
}

impl core::ops::Mul for FloatOrInt {
    type Output = Self;

    fn mul(self, right: Self) -> Self::Output {
        match (self, right) {
            (Self::Float(left), Self::Float(right)) => Self::Float(left * right),
            (Self::Float(left), Self::Int(right)) => Self::Float(left * right as f64),
            (Self::Int(left), Self::Float(right)) => Self::Float(left as f64 * right),
            (Self::Int(left), Self::Int(right)) => Self::Int(left * right),
        }
    }
}

/// Includes code fragments for the type to be used with a particular signal in a message.
struct SignalType {
    /// The type of the signal exposed to external rust code.
    ty: syn::Ident,

    /// The type of the signal when encoded into a bytestream.
    raw_ty: syn::Ident,

    /// Called with `(raw_ty, expr: ty)` args and returns an expression converting it into a `raw_ty` for
    /// serialization. (The `raw_ty` is passed as an implementation detail to allow avoiding closure captures in some
    /// variations).
    to_raw: fn(&syn::Ident, TokenStream) -> TokenStream,

    /// Called with a `(ty, expr: raw_ty)` args and returns an expression converting it into a `ty` for
    /// deserialization. (The `ty` is passed as an implementation detail to allow avoiding closure captures in some
    /// variations).
    from_raw: fn(&syn::Ident, TokenStream) -> TokenStream,

    /// Turns a value into a `ty` literal.
    make_lit: fn(FloatOrInt) -> syn::Lit,

    /// Turns a value into a `raw_ty` literal.
    make_raw_lit: fn(FloatOrInt) -> syn::Lit,

    /// The max value for `ty`.
    ty_max: FloatOrInt,

    /// The min value for `ty`.
    ty_min: FloatOrInt,
}

fn signal_type(
    dbc: &DBC,
    message: &Message,
    signal: &Signal,
    factor: FloatOrInt,
    offset: FloatOrInt,
    mut max: FloatOrInt,
    mut min: FloatOrInt,
) -> Result<SignalType> {
    let extended_type = dbc
        .signal_extended_value_type_list()
        .iter()
        .find(|info| {
            info.message_id() == message.message_id() && info.signal_name() == signal.name()
        })
        .map(|info| info.signal_extended_value_type());

    let raw_ty: syn::Ident = syn::parse_str(match (signal.value_type(), signal.signal_size) {
        (ValueType::Signed, 1..=8) => "i8",
        (ValueType::Signed, 9..=16) => "i16",
        (ValueType::Signed, 17..=32) => "i32",
        (ValueType::Signed, 33..=64) => "i64",
        (ValueType::Unsigned, 1..=8) => "u8",
        (ValueType::Unsigned, 9..=16) => "u16",
        (ValueType::Unsigned, 17..=32) => "u32",
        (ValueType::Unsigned, 33..=64) => "u64",
        (_, size) => bail!("unsupported signal size {size} for {}", signal.name()),
    })?;

    let (ty_max, ty_min) = match signal.value_type() {
        ValueType::Signed => (
            (2i128.pow(signal.signal_size as u32 - 1) - 1).into(),
            (-2i128.pow(signal.signal_size as u32 - 1)).into(),
        ),
        ValueType::Unsigned => ((2i128.pow(signal.signal_size as u32) - 1).into(), 0.into()),
    };

    match extended_type {
        Some(
            SignalExtendedValueType::IEEEfloat32Bit | SignalExtendedValueType::IEEEdouble64bit,
        ) => {
            assert_eq!(signal.value_type(), &ValueType::Signed);

            match extended_type {
                Some(SignalExtendedValueType::IEEEfloat32Bit) => {
                    assert_eq!(signal.signal_size, 32);
                    let ty = syn::parse_str("f32")?;
                    let raw_ty = syn::parse_str("u32")?;
                    Ok(SignalType {
                        to_raw: |_, value| quote!(f32::to_bits(#value)),
                        from_raw: |_, raw| quote!(f32::from_bits(#raw)),
                        ty,
                        raw_ty,
                        make_lit: FloatOrInt::make_f32_lit,
                        make_raw_lit: FloatOrInt::make_f32_bits_lit,
                        ty_max: f32::MAX.into(),
                        ty_min: f32::MIN.into(),
                    })
                }
                Some(SignalExtendedValueType::IEEEdouble64bit) => {
                    assert_eq!(signal.signal_size, 64);
                    let ty = syn::parse_str("f64")?;
                    let raw_ty = syn::parse_str("u64")?;
                    Ok(SignalType {
                        to_raw: |_, value| quote!(f64::to_bits(#value)),
                        from_raw: |_, raw| quote!(f64::from_bits(#raw)),
                        ty,
                        raw_ty,
                        make_lit: FloatOrInt::make_f64_lit,
                        make_raw_lit: FloatOrInt::make_f64_bits_lit,
                        ty_max: f64::MAX.into(),
                        ty_min: f64::MIN.into(),
                    })
                }
                _ => unreachable!(),
            }
        }

        Some(SignalExtendedValueType::SignedOrUnsignedInteger) => {
            assert!(factor.is_int() && offset.is_int());
            let ty = raw_ty.clone();
            Ok(SignalType {
                to_raw: |_, value| value,
                from_raw: |_, raw| raw,
                ty,
                raw_ty,
                make_lit: FloatOrInt::make_int_lit,
                make_raw_lit: FloatOrInt::make_int_lit,
                ty_max,
                ty_min,
            })
        }

        None => {
            if factor.is_int() && offset.is_int() {
                if max == FloatOrInt::Int(0) && min == FloatOrInt::Int(0) {
                    (max, min) = (ty_max * factor + offset, ty_min * factor + offset);
                }

                let Some((min, max)) = Option::zip(min.as_int(), max.as_int()) else {
                    panic!("signal has integer factor but non-integer min/max")
                };

                mod c {
                    pub mod u8 {
                        pub const MIN: i128 = u8::MIN as i128;
                        pub const MAX: i128 = u8::MAX as i128;
                    }
                    pub mod i8 {
                        pub const MIN: i128 = i8::MIN as i128;
                        pub const MAX: i128 = i8::MAX as i128;
                    }
                    pub mod u16 {
                        pub const MIN: i128 = u16::MIN as i128;
                        pub const MAX: i128 = u16::MAX as i128;
                    }
                    pub mod i16 {
                        pub const MIN: i128 = i16::MIN as i128;
                        pub const MAX: i128 = i16::MAX as i128;
                    }
                    pub mod u32 {
                        pub const MIN: i128 = u32::MIN as i128;
                        pub const MAX: i128 = u32::MAX as i128;
                    }
                    pub mod i32 {
                        pub const MIN: i128 = i32::MIN as i128;
                        pub const MAX: i128 = i32::MAX as i128;
                    }
                    pub mod u64 {
                        pub const MIN: i128 = u64::MIN as i128;
                        pub const MAX: i128 = u64::MAX as i128;
                    }
                    pub mod i64 {
                        pub const MIN: i128 = i64::MIN as i128;
                        pub const MAX: i128 = i64::MAX as i128;
                    }
                }

                let ty = syn::parse_str(match (min, max) {
                    (c::u8::MIN..=c::u8::MAX, c::u8::MIN..=c::u8::MAX) => "u8",
                    (c::i8::MIN..=c::i8::MAX, c::i8::MIN..=c::i8::MAX) => "i8",
                    (c::u16::MIN..=c::u16::MAX, c::u16::MIN..=c::u16::MAX) => "u16",
                    (c::i16::MIN..=c::i16::MAX, c::i16::MIN..=c::i16::MAX) => "i16",
                    (c::u32::MIN..=c::u32::MAX, c::u32::MIN..=c::u32::MAX) => "u32",
                    (c::i32::MIN..=c::i32::MAX, c::i32::MIN..=c::i32::MAX) => "i32",
                    (c::u64::MIN..=c::u64::MAX, c::u64::MIN..=c::u64::MAX) => "u64",
                    (c::i64::MIN..=c::i64::MAX, c::i64::MIN..=c::i64::MAX) => "i64",
                    (_, _) => {
                        panic!("signal value out of supported range (min = {min}, max = {max})")
                    }
                })
                .unwrap();

                Ok(SignalType {
                    to_raw: if ty == raw_ty {
                        |_, value| value
                    } else {
                        |raw_ty, value| {
                            quote!(
                                #raw_ty::try_from(#value).expect("the range was checked on the value before scaling")
                            )
                        }
                    },
                    from_raw: if ty == raw_ty {
                        |_, value| value
                    } else {
                        |ty, value| quote!(#ty::from(#value))
                    },
                    ty,
                    raw_ty,
                    make_lit: FloatOrInt::make_int_lit,
                    make_raw_lit: FloatOrInt::make_int_lit,
                    ty_max,
                    ty_min,
                })
            } else {
                let ty = syn::parse_str("f64")?;
                Ok(SignalType {
                    // `+/- 0.5)` is used as an alternative to `.round()` because that's not available on no-std,
                    // the value should be very close to an integer anyway so this should be the same.
                    to_raw: match signal.value_type() {
                        // The sign check is necessary because `as {integer}` truncates towards 0, so we need to turn
                        // -0.9999 into -1.4999 so it gets truncated to -1 (and `signum()` is not available on no-std
                        // either).
                        ValueType::Signed => |raw_ty, value| {
                            if value.clone().into_iter().count() == 1 {
                                quote!((value + if value > 0.0 { 0.5 } else { -0.5 }) as #raw_ty)
                            } else {
                                quote!({
                                    let value = #value;
                                    (value + if value > 0.0 { 0.5 } else { -0.5 }) as #raw_ty
                                })
                            }
                        },
                        ValueType::Unsigned => |raw_ty, value| quote!(((#value + 0.5) as #raw_ty)),
                    },
                    from_raw: |ty, value| quote!(#value as #ty),
                    ty,
                    raw_ty,
                    make_lit: FloatOrInt::make_f64_lit,
                    make_raw_lit: FloatOrInt::make_int_lit,
                    ty_max,
                    ty_min,
                })
            }
        }
    }
}

fn translate_be_signal_start(start_bit: usize) -> usize {
    // CAN-DBC appears to use `Lsb0` indexing of the bits even for BE values, so we have to invert the bit-offset within
    // the target byte to get the `Msb0` index.
    let (byte_index, bit_offset) = (start_bit / 8, start_bit % 8);
    byte_index * 8 + (7 - bit_offset)
}

#[test]
fn test_translate_be_signal_start() {
    assert_eq!(translate_be_signal_start(55), 48);
    assert_eq!(translate_be_signal_start(39), 32);
    assert_eq!(translate_be_signal_start(7), 0);

    assert_eq!(translate_be_signal_start(61), 58);
    assert_eq!(translate_be_signal_start(60), 59);
    assert_eq!(translate_be_signal_start(31), 24);
    assert_eq!(translate_be_signal_start(7), 0);
}

/// Generates a data type for `signal` along with conversion functions.
fn generate_signal(
    options: &crate::Options,
    dbc: &DBC,
    message: &Message,
    signal: &Signal,
) -> Result<GeneratedSignal> {
    let crate::Options {
        veecle_os_runtime,
        veecle_os_data_support_can,
        ..
    } = options;

    let name_str = signal.name().to_pascal_case();
    let name: syn::Ident = syn::parse_str(&name_str)?;
    let snake_case_name_str = signal.name().to_snake_case();
    let snake_case_name: syn::Ident = syn::parse_str(&snake_case_name_str)?;

    let comments = dbc
        .comments()
        .iter()
        .filter_map(|comment| match comment {
            Comment::Signal {
                message_id,
                signal_name,
                comment,
            } if message_id == message.message_id() && signal_name == signal.name() => {
                Some(comment)
            }
            _ => None,
        })
        .map(|comment| format!(" ```text\n{comment}\n```"))
        .collect::<Vec<_>>();

    let SignalType {
        ty,
        raw_ty,
        to_raw,
        from_raw,
        make_lit,
        make_raw_lit,
        ty_max,
        ty_min,
    } = signal_type(
        dbc,
        message,
        signal,
        signal.factor().into(),
        signal.offset().into(),
        signal.max.into(),
        signal.min.into(),
    )?;

    let factor = make_lit(signal.factor().into());
    let offset = make_lit(signal.offset().into());

    let (full_range, max, min, raw_max, raw_min) = if signal.max == 0.0 && signal.min == 0.0 {
        // Assume the full range
        let max = ty_max * signal.factor().into() + signal.offset().into();
        let min = ty_min * signal.factor().into() + signal.offset().into();
        (
            true,
            make_lit(max),
            make_lit(min),
            make_raw_lit(ty_max),
            make_raw_lit(ty_min),
        )
    } else {
        let raw_max = ((signal.max - signal.offset()) / signal.factor()).into();
        let raw_min = ((signal.min - signal.offset()) / signal.factor()).into();
        (
            FloatOrInt::from(signal.max) == ty_max && FloatOrInt::from(signal.min) == ty_min,
            make_lit(signal.max.into()),
            make_lit(signal.min.into()),
            make_raw_lit(raw_max),
            make_raw_lit(raw_min),
        )
    };

    let start_bit = usize::try_from(signal.start_bit)?;
    let signal_size = usize::try_from(signal.signal_size)?;

    let (start_bit, read_bits, write_bits) =
        match (signal.byte_order(), raw_ty.to_string().starts_with("u")) {
            (can_dbc::ByteOrder::LittleEndian, true) => (
                start_bit,
                quote!(read_little_endian_unsigned),
                quote!(write_little_endian_unsigned),
            ),
            (can_dbc::ByteOrder::LittleEndian, false) => (
                start_bit,
                quote!(read_little_endian_signed),
                quote!(write_little_endian_signed),
            ),
            (can_dbc::ByteOrder::BigEndian, true) => (
                translate_be_signal_start(start_bit),
                quote!(read_big_endian_unsigned),
                quote!(write_big_endian_unsigned),
            ),
            (can_dbc::ByteOrder::BigEndian, false) => (
                translate_be_signal_start(start_bit),
                quote!(read_big_endian_signed),
                quote!(write_big_endian_signed),
            ),
        };

    ensure!(
        start_bit + signal_size <= 64,
        "invalid start-bit/signal-size {start_bit}/{signal_size} for signal {:?} of message {:?} [id {:?}]",
        signal.name(),
        message.message_name(),
        message.message_id()
    );

    let start_bit = proc_macro2::Literal::usize_unsuffixed(start_bit);
    let signal_size = proc_macro2::Literal::usize_unsuffixed(signal_size);

    let out_of_range_error = format!(
        "out of range {}..={}",
        min.to_token_stream(),
        max.to_token_stream()
    );

    let arbitrary_impl = options.arbitrary.as_ref().map(|a| {
        let arbitrary = &a.path;
        let cfg = a.to_cfg();
        // `arbitrary` has no method to generate a float in a range, but there's a direct relationship with the
        // range of valid raw values, so we can generate those instead.
        let value = quote! {
            let min = Self::MIN.raw();
            let max = Self::MAX.raw();
            Ok(Self::try_from_raw(u.int_in_range(min..=max)?).expect("we generate in range"))
        };
        quote! {
            #cfg
            impl<'a> #arbitrary::Arbitrary<'a> for #name {
                fn arbitrary(u: &mut #arbitrary::Unstructured<'a>) -> #arbitrary::Result<Self> {
                    #value
                }
            }
        }
    });

    let choices = {
        let mut choices = Vec::from_iter(
            dbc.value_descriptions_for_signal(*message.message_id(), signal.name())
                .into_iter()
                .flatten()
                .map(|description| (description.a(), description.b())),
        );

        // Ensure that with duplicate choices the lowest value gets the original description.
        choices.sort_by(|(value1, _), (value2, _)| value1.total_cmp(value2));

        // In case there's conflicts with our builtin `MAX` and `MIN` consts, add them as pre-known values to
        // deduplicate.
        let mut seen: HashSet<String> = HashSet::from(["MAX".to_owned(), "MIN".to_owned()]);

        Vec::from_iter(choices.into_iter().map(move |(value, description)| {
            // Turn the description into a valid Rust identifier.
            let mut first = true;
            let mut name = String::from_iter(description.chars().map(|c| {
                let valid = if std::mem::take(&mut first) {
                    c == '_' || unicode_ident::is_xid_start(c)
                } else {
                    unicode_ident::is_xid_continue(c)
                };
                if valid { c } else { '_' }
            }));

            // Multiple values can have the same description, add `_` suffixes to ensure these are unique.
            while seen.contains(&name) {
                name = format!("{name}_");
            }
            seen.insert(name.clone());

            (quote::format_ident!("{name}"), value, description)
        }))
    };

    let consts = Vec::from_iter({
        choices.iter().map(move |(name, value, description)| {
            let description = format!(" {description}");
            let raw = make_raw_lit(((*value - signal.offset()) / signal.factor()).into());
            quote! {
                #[doc = #description]
                #[allow(non_upper_case_globals)]
                pub const #name: Self = Self { raw: #raw };
            }
        })
    });

    let debug_impl = {
        let basic_body = quote! {
            f.debug_struct(#name_str)
                .field("raw", &self.raw)
                .field("value", &self.value())
                .finish()
        };

        let body = if choices.is_empty() {
            basic_body
        } else {
            let names = choices.iter().map(|(name, _, _)| name);
            let full_names = choices
                .iter()
                .map(|(choice_name, _, _)| format!("{name}::{choice_name}"));
            quote! {
                match *self {
                    #(
                        Self::#names => {
                            f.debug_struct(#full_names)
                                .field("raw", &self.raw)
                                .field("value", &self.value())
                                .finish()
                        }
                    )*
                    _ => {
                        #basic_body
                    }
                }
            }
        };

        quote! {
            impl core::fmt::Debug for #name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    #body
                }
            }
        }
    };

    let make_from_raw_with_factor = |raw| {
        let mut value = from_raw(&ty, raw);
        if *signal.factor() != 1.0 {
            value = quote!(#value * #factor);
        }
        if *signal.offset() != 0.0 {
            value = quote!(#value + #offset);
        }
        value
    };

    let from_raw_with_factor = make_from_raw_with_factor(quote!(raw));
    let from_self_raw_with_factor = make_from_raw_with_factor(quote!(self.raw));

    let to_raw_with_factor = {
        let mut value = quote!(value);
        if *signal.offset() != 0.0 {
            value = quote!(#value - #offset);
        }
        if *signal.factor() != 1.0 {
            value = quote!(#value / #factor);
        }
        to_raw(&raw_ty, value)
    };

    let try_from_body = if full_range {
        quote! {
            Ok(Self { raw: #to_raw_with_factor })
        }
    } else {
        quote! {
            if (#min..=#max).contains(&value) {
                Ok(Self { raw: #to_raw_with_factor })
            } else {
                Err(Self::Error::OutOfRange { name: stringify!(#name), ty: stringify!(#ty), message: #out_of_range_error })
            }
        }
    };

    Ok(GeneratedSignal {
        definition: quote! {
            #(#[doc = #comments])*
            #[derive(Clone, Copy, PartialEq, PartialOrd, _serde::Serialize)]
            #[serde(crate = "_serde")]
            pub struct #name {
                raw: #raw_ty,
            }

            impl #name {
                pub const MAX: Self = Self { raw: #raw_max };
                pub const MIN: Self = Self { raw: #raw_min };

                #(#consts)*

                fn try_from_raw(raw: #raw_ty) -> Result<Self, #veecle_os_data_support_can::CanDecodeError> {
                    Self::try_from(#from_raw_with_factor)
                }

                fn raw(&self) -> #raw_ty {
                    self.raw
                }

                pub(super) fn read_bits(bytes: &[u8]) -> Result<Self, #veecle_os_data_support_can::CanDecodeError> {
                    Self::try_from_raw(#raw_ty::try_from(bits::#read_bits(bytes, #start_bit, #signal_size)).unwrap())
                }

                pub(super) fn write_bits(&self, bytes: &mut [u8]) {
                    bits::#write_bits(bytes, #start_bit, #signal_size, self.raw().into())
                }

                pub fn value(&self) -> #ty {
                    #from_self_raw_with_factor
                }
            }

            impl Default for #name {
                fn default() -> Self {
                    Self::MIN
                }
            }

            impl TryFrom<#ty> for #name {
                type Error = #veecle_os_data_support_can::CanDecodeError;

                fn try_from(value: #ty) -> Result<Self, Self::Error> {
                    #try_from_body
                }
            }

            impl #veecle_os_runtime::Storable for #name {
                type DataType = Self;
            }

            #debug_impl

            #arbitrary_impl
        },
        name,
        snake_case_name,
    })
}

/// Generates a module for data types and conversions related to `message`.
fn generate_message(options: &crate::Options, dbc: &DBC, message: &Message) -> Result<TokenStream> {
    let crate::Options {
        veecle_os_runtime,
        veecle_os_data_support_can,
        serde,
        message_frame_validations,
        ..
    } = options;

    let name = syn::parse_str::<syn::Ident>(&message.message_name().to_pascal_case())?;
    let snake_case_name = syn::parse_str::<syn::Ident>(&message.message_name().to_snake_case())?;

    let comments = dbc
        .comments()
        .iter()
        .filter_map(|comment| match comment {
            Comment::Message {
                message_id,
                comment,
            } if message_id == message.message_id() => Some(comment),
            _ => None,
        })
        .map(|comment| format!(" ```text\n{comment}\n```"))
        .collect::<Vec<_>>();

    let validation =
        message_frame_validations(&name).map(|validation| quote!(let () = #validation(&bytes)?;));

    let message_size = usize::try_from(*message.message_size())?;
    ensure!(
        message_size <= 8,
        "invalid size {message_size} for message {:?} [id {:?}]",
        message.message_name(),
        message.message_id()
    );

    let frame_id = match message.message_id() {
        can_dbc::MessageId::Standard(id) => {
            let id = syn::LitInt::new(&format!("{id:#x}"), Span::call_site());
            quote! {
                #veecle_os_data_support_can::Id::Standard(#veecle_os_data_support_can::StandardId::new_unwrap(#id))
            }
        }
        can_dbc::MessageId::Extended(id) => {
            let id = syn::LitInt::new(&format!("{id:#x}"), Span::call_site());
            quote! {
                #veecle_os_data_support_can::Id::Extended(#veecle_os_data_support_can::ExtendedId::new_unwrap(#id))
            }
        }
    };

    let signals = Result::<Vec<_>>::from_iter(
        message
            .signals()
            .iter()
            .map(|signal| generate_signal(options, dbc, message, signal)),
    )?;

    let signal_definitions = Vec::from_iter(signals.iter().map(|signal| &signal.definition));
    let signal_names = Vec::from_iter(signals.iter().map(|signal| &signal.name));
    let signal_snake_case_names =
        Vec::from_iter(signals.iter().map(|signal| &signal.snake_case_name));

    let arbitrary_impl = options.arbitrary.as_ref().map(|a| {
        let arbitrary = &a.path;
        let cfg = a.to_cfg();
        quote! {
            #cfg
            impl<'a> #arbitrary::Arbitrary<'a> for #name {
                fn arbitrary(u: &mut #arbitrary::Unstructured<'a>) -> #arbitrary::Result<Self> {
                    Ok(Self {
                        #(#signal_snake_case_names: u.arbitrary()?,)*
                    })
                }
            }
        }
    });

    Ok(quote! {
        pub mod #snake_case_name {
            use #veecle_os_data_support_can::reëxports::bits;
            use #serde as _serde;

            #(#signal_definitions)*
        }

        #(#[doc = #comments])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, _serde::Serialize)]
        #[serde(crate = "_serde")]
        pub struct #name {
            #(pub #signal_snake_case_names: #snake_case_name::#signal_names,)*
        }

        impl #name {
            pub const FRAME_ID: #veecle_os_data_support_can::Id = #frame_id;
            pub const FRAME_LENGTH: usize = #message_size;
        }

        impl TryFrom<&#veecle_os_data_support_can::Frame> for #name {
            type Error = #veecle_os_data_support_can::CanDecodeError;
            fn try_from(frame: &#veecle_os_data_support_can::Frame) -> Result<Self, Self::Error> {
                if frame.id() != Self::FRAME_ID {
                    return Err(#veecle_os_data_support_can::CanDecodeError::IncorrectId);
                }

                let bytes: [u8; Self::FRAME_LENGTH] = frame.data()
                    .try_into()
                    .map_err(|_| #veecle_os_data_support_can::CanDecodeError::IncorrectBufferSize)?;

                #validation

                Ok(Self {
                    #(#signal_snake_case_names: #snake_case_name::#signal_names::read_bits(&bytes)?,)*
                })
            }
        }

        impl TryFrom<#veecle_os_data_support_can::Frame> for #name {
            type Error = #veecle_os_data_support_can::CanDecodeError;
            fn try_from(frame: #veecle_os_data_support_can::Frame) -> Result<Self, Self::Error> {
                Self::try_from(&frame)
            }
        }

        impl From<&#name> for #veecle_os_data_support_can::Frame {
            fn from(value: &#name) -> Self {
                let mut bytes = [0u8; #name::FRAME_LENGTH];
                #(
                    value.#signal_snake_case_names.write_bits(&mut bytes);
                )*
                Frame::new(#name::FRAME_ID, bytes)
            }
        }

        impl From<#name> for #veecle_os_data_support_can::Frame {
            fn from(value: #name) -> Self {
                Self::from(&value)
            }
        }

        impl #veecle_os_runtime::Storable for #name {
            type DataType = Self;
        }

        #arbitrary_impl
    })
}

pub(super) fn generate(options: &crate::Options, dbc: &DBC) -> Result<TokenStream> {
    let serde = &options.serde;

    let messages = Result::<Vec<_>>::from_iter(
        dbc.messages()
            .iter()
            .map(|message| generate_message(options, dbc, message)),
    )?;

    Ok(quote! {
        use #serde as _serde;

        #(#messages)*
    })
}
