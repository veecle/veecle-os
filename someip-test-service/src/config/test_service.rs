//! Types to configure the test service.

use strum::EnumString;

/// Configuration for the test service.
#[derive(Clone, Debug)]
pub struct Config {
    /// Unicast address for sending RPC requests.
    ///
    /// Mapped to the `unicast` field of the vsomeip configuration.
    ///
    /// # Note
    ///
    /// If you want to run on the loopback interface, please use `127.0.0.1`
    /// because `vsomeip` won't open a socket on any other address.
    pub unicast_address: String,

    /// Unicast port for sending RPC requests.
    /// Must be unique for each service instance.
    ///
    /// Mapped to the `services[].unreliable` field of the vsomeip configuration.
    pub unicast_port: u16,

    /// Enable service discovery.
    ///
    /// Mapped to the `service-discovery.enable` field of the vsomeip configuration.
    pub service_discovery: bool,

    /// Multicast address for service discovery requests.
    ///
    /// Mapped to the `service-discovery.multicast` field of the vsomeip configuration.
    /// Makes effect only if [`Self::service_discovery`] is `true`.
    pub multicast_address: String,

    /// Multicast port for service discovery requests.
    ///
    /// Mapped to the `service-discovery.port` field of the vsomeip configuration.
    /// Makes effect only if [`Self::service_discovery`] is `true`.
    pub multicast_port: u16,

    /// Logging level of the service.
    ///
    /// Mapped to the `logging.level` field of the vsomeip configuration
    /// and the `[logging].level` field of the Common API configuration.
    pub logging_level: LoggingLevel,
}

impl Default for Config {
    /// Creates default instance of the [`Config`].
    ///
    /// # Defaults
    ///
    /// - [`Config::unicast_address`] - `127.0.0.1`
    /// - [`Config::unicast_port`] - Random free UDP port.
    /// - [`Config::service_discovery`] - `false`.
    /// - [`Config::multicast_address`] - `224.244.224.245`.
    /// - [`Config::multicast_port`] - `30490`.
    /// - [`Config::logging_level`] - [`LoggingLevel::Info`].
    ///
    /// # Panics
    ///
    /// - When it is not possible to obtain a random free UDP port of the `127.0.0.1`.
    fn default() -> Self {
        let unicast_address = String::from("127.0.0.1");
        let unicast_port = get_free_udp_port(&unicast_address);
        Self {
            unicast_address,
            unicast_port,
            // TODO: Figure out why service discovery is not working on a CI.
            service_discovery: false,
            multicast_address: String::from("224.244.224.245"),
            multicast_port: 30490,
            logging_level: LoggingLevel::Info,
        }
    }
}

/// Returns a random free UDP port number of the provided interface.
fn get_free_udp_port(interface_address: &str) -> u16 {
    let socket =
        std::net::UdpSocket::bind((interface_address, 0)).expect("failed to get free UDP port");
    let address = socket.local_addr().expect("failed to get socket address");
    address.port()
}

/// Logging level of the test service.
///
/// See [`Self::to_common_api`] to understand how it is mapped to the CommonAPI.
/// See [`Self::to_vsomeip`] to understand how it is mapped to vsomeip.
#[derive(Clone, Debug, EnumString)]
pub enum LoggingLevel {
    /// Print only fatal errors.
    Fatal,

    /// Print log messages of the error level and above.
    Error,

    /// Print log messages of the warning level and above.
    Warning,

    /// Print log messages of the info level and above.
    Info,

    /// Print log messages of the debug level and above.
    Debug,

    /// Print all log messages.
    Trace,
}

impl LoggingLevel {
    /// Converts to a Common API log level.
    /// See: <https://github.com/COVESA/capicxx-core-tools/blob/master/docx/CommonAPICppUserGuide>
    pub fn to_common_api(&self) -> &'static str {
        match self {
            Self::Fatal | Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "verbose",
        }
    }

    /// Converts to a vsomeip log level.
    /// See: <https://github.com/COVESA/vsomeip/blob/master/documentation/vsomeipConfiguration.md#1-logging>
    pub fn to_vsomeip(&self) -> &'static str {
        match self {
            Self::Fatal => "fatal",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}
