//! The Veecle OS framework.

#![forbid(unsafe_code)]
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc(inline)]
pub use veecle_os_runtime as runtime;

/// The Veecle OS operating system abstraction layer.
pub mod osal {
    #[doc(inline)]
    pub use veecle_osal_api as api;
    #[doc(inline)]
    #[cfg(feature = "osal-embassy")]
    pub use veecle_osal_embassy as embassy;
    #[doc(inline)]
    #[cfg(feature = "osal-freertos")]
    pub use veecle_osal_freertos as freertos;
    #[doc(inline)]
    #[cfg(feature = "osal-std")]
    pub use veecle_osal_std as std;
}

#[doc(inline)]
#[cfg(feature = "telemetry")]
pub use veecle_telemetry as telemetry;
#[doc(inline)]
#[cfg(feature = "telemetry")]
pub use veecle_telemetry::{debug, error, event, fatal, info, log, span, trace};

/// Support modules for working with various data formats.
pub mod data_support {
    #[doc(inline)]
    #[cfg(feature = "data-support-can")]
    pub use veecle_os_data_support_can as can;
    #[doc(inline)]
    #[cfg(feature = "data-support-someip")]
    pub use veecle_os_data_support_someip as someip;
}
