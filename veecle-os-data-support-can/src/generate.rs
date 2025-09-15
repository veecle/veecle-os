//! Macros to generate data types and Veecle OS actors from CAN-DBC

// This is a wrapper macro instead of re-export so that we can pass the `$crate` token in
// referring to this crate. This makes the macro resilient to dependency renaming and
// re-exporting which a hardcoded `::veecle_os_data_support_can` path can't be.

/// Parse a CAN-DBC file or source string and generate data types and Veecle OS actors to use with it.
///
/// ```rust
/// veecle_os_data_support_can::generate!(
///     mod from_file {
///         #![dbc = include_str!("../../veecle-os-data-support-can-codegen/tests/cases/CSS-Electronics-SAE-J1939-DEMO.dbc")]
///     }
/// );
///
/// veecle_os_data_support_can::generate!(
///     mod from_str {
///         #![dbc = r#"
///             VERSION ""
///
///             NS_ :
///
///             BO_ 2364540158 EEC1: 8 Vector__XXX
///              SG_ EngineSpeed : 24|16@1+ (0.125,0) [0|8031.875] "rpm" Vector__XXX
///         "#]
///     }
/// );
///
/// let _ = from_file::Eec1 {
///     engine_speed: from_file::eec1::EngineSpeed::try_from(0.5).unwrap(),
/// };
///
/// let _ = from_str::Eec1 {
///     engine_speed: from_str::eec1::EngineSpeed::try_from(0.5).unwrap(),
/// };
/// ```
#[macro_export]
macro_rules! generate {
    ($vis:vis mod $name:ident { #![dbc = include_str!($file:literal)] $($extra:tt)* }) => {
        $crate::reëxports::veecle_os_data_support_can_macros::from_file!($crate; $vis mod $name; $file; $($extra)* );
    };

    ($vis:vis mod $name:ident { #![dbc = $str:literal] $($extra:tt)* }) => {
        $crate::reëxports::veecle_os_data_support_can_macros::from_str!($crate; $vis mod $name; $str; $($extra)* );
    };
}
