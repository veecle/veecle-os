//! Socket address types and parsing logic.
//!
//! Generic socket addressing that can represent either Unix domain sockets or TCP sockets with
//! hostname resolution support.

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::net::{SocketAddr, ToSocketAddrs};
use std::os::unix::net::SocketAddr as UnixSocketAddr;
use std::str::FromStr;

/// A parsed-but-not-resolved [`SocketAddr`]-like to support hostnames.
///
/// See tests for what is supported.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnresolvedSocketAddress {
    /// Either a hostname or an IP address (v4 or v6).
    pub(crate) host: String,

    // IPv6 is the only case where [`SocketAddr`] syntax is not just `{host}:{port}`, it has extra
    // `[]` around the IP to distinguish from the port number.
    pub(crate) is_v6: bool,

    pub(crate) port: u16,
}

impl ToSocketAddrs for UnresolvedSocketAddress {
    // Can't use `(&str, u16)` here because of the lifetime.
    type Iter = <(String, u16) as ToSocketAddrs>::Iter;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        (self.host.as_str(), self.port).to_socket_addrs()
    }
}

impl Display for UnresolvedSocketAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let UnresolvedSocketAddress { host, is_v6, port } = self;
        if *is_v6 {
            write!(f, "[{host}]:{port}")?;
        } else {
            write!(f, "{host}:{port}")?;
        }
        Ok(())
    }
}

/// Errors that can occur when parsing unresolved socket addresses.
#[derive(thiserror::Error, Debug, Clone)]
pub enum UnresolvedSocketAddressParseError {
    /// Missing ':' separator between host and port.
    #[error("missing ':' separator")]
    MissingSeparator,

    /// Invalid port number format.
    #[error("invalid port number")]
    InvalidPortNumber(#[source] std::num::ParseIntError),

    /// Invalid hostname format.
    #[error("invalid hostname: {0}")]
    InvalidHostname(&'static str),
}

impl FromStr for UnresolvedSocketAddress {
    type Err = UnresolvedSocketAddressParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let (host, port) = string
            .rsplit_once(":")
            .ok_or(UnresolvedSocketAddressParseError::MissingSeparator)?;

        let port = port
            .parse()
            .map_err(UnresolvedSocketAddressParseError::InvalidPortNumber)?;

        fn is_ipv4(host: &str) -> bool {
            std::net::Ipv4Addr::from_str(host).is_ok()
        }

        fn is_ipv6(host: &str) -> Option<&str> {
            let host = host.strip_prefix('[')?.strip_suffix(']')?;
            std::net::Ipv6Addr::from_str(host).is_ok().then_some(host)
        }

        /// Validates hostname according to [RFC 1123 §2.1] + [RFC 952] syntax rules:
        ///
        /// - Total length <= 253 characters
        /// - Each label (part between dots) <= 63 characters
        /// - Labels contain only alphanumeric characters and hyphens
        /// - Labels cannot start or end with hyphens
        /// - Trailing dot allowed for Fully Qualified Domain Name (FQDN)
        ///
        /// [RFC 1123 §2.1]: https://datatracker.ietf.org/doc/html/rfc1123#section-2
        /// [RFC 952]: https://datatracker.ietf.org/doc/html/rfc952
        fn validate_hostname(host: &str) -> Result<(), UnresolvedSocketAddressParseError> {
            if host.is_empty() {
                return Err(UnresolvedSocketAddressParseError::InvalidHostname(
                    "is empty",
                ));
            }

            if host.len() > 253 {
                return Err(UnresolvedSocketAddressParseError::InvalidHostname(
                    "is too long",
                ));
            }

            if host.starts_with('.') {
                return Err(UnresolvedSocketAddressParseError::InvalidHostname(
                    "starts with period",
                ));
            }

            // Strip a trailing `.` to allow for FQDN.
            for label in host.strip_suffix('.').unwrap_or(host).split('.') {
                if label.is_empty() {
                    return Err(UnresolvedSocketAddressParseError::InvalidHostname(
                        "contains empty label",
                    ));
                }

                if label.len() > 63 {
                    return Err(UnresolvedSocketAddressParseError::InvalidHostname(
                        "label too long",
                    ));
                }

                if label.starts_with('-') {
                    return Err(UnresolvedSocketAddressParseError::InvalidHostname(
                        "label starts with dash",
                    ));
                }

                if label.ends_with('-') {
                    return Err(UnresolvedSocketAddressParseError::InvalidHostname(
                        "label ends with dash",
                    ));
                }

                if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                    return Err(UnresolvedSocketAddressParseError::InvalidHostname(
                        "label contains disallowed character",
                    ));
                }
            }

            Ok(())
        }

        if let Some(host) = is_ipv6(host) {
            Ok(Self {
                host: host.to_owned(),
                is_v6: true,
                port,
            })
        } else {
            if !is_ipv4(host) {
                validate_hostname(host)?;
            }
            Ok(Self {
                host: host.to_owned(),
                is_v6: false,
                port,
            })
        }
    }
}

/// Generic socket address that can be either Unix or TCP.
#[derive(Debug, Clone)]
pub enum MultiSocketAddress {
    /// Unix domain socket path.
    Unix(UnixSocketAddr),

    /// TCP socket with IP and port.
    Tcp(SocketAddr),
}

impl Display for MultiSocketAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unix(address) => fmt::Debug::fmt(address, f),
            Self::Tcp(address) => address.fmt(f),
        }
    }
}

/// A parsed-but-not-resolved generic socket address that can be either Unix or TCP.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum UnresolvedMultiSocketAddress {
    /// Unix domain socket path.
    Unix(Utf8PathBuf),

    /// TCP socket with hostname/IP and port.
    Tcp(UnresolvedSocketAddress),
}

impl Display for UnresolvedMultiSocketAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unix(address) => address.fmt(f),
            Self::Tcp(address) => address.fmt(f),
        }
    }
}

/// Errors that can occur when parsing socket addresses.
#[derive(thiserror::Error, Debug)]
pub enum UnresolvedMultiSocketAddressParseError {
    /// Invalid TCP address format. Unix paths must start with '/', './', or '../'.
    #[error("invalid TCP address (Unix paths must start with '/', './', or '../')")]
    Tcp(#[from] UnresolvedSocketAddressParseError),
}

impl FromStr for UnresolvedMultiSocketAddress {
    type Err = UnresolvedMultiSocketAddressParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // Unix socket paths must be absolute or explicit relative paths.
        if string.starts_with('/') || string.starts_with("./") || string.starts_with("../") {
            Ok(UnresolvedMultiSocketAddress::Unix(string.into()))
        } else {
            let socket = UnresolvedSocketAddress::from_str(string)?;
            Ok(UnresolvedMultiSocketAddress::Tcp(socket))
        }
    }
}

/// Errors that can occur when converting a [`MultiSocketAddress`] to an
/// [`UnresolvedMultiSocketAddress`].
#[derive(thiserror::Error, Debug)]
pub enum UnresolvedMultiSocketAddressTryFromMultiSocketAddressError {
    /// The Unix socket is unnamed.
    #[error("the unix socket is unnamed")]
    UnnamedUnixSocket,

    /// The Unix socket path is non-utf8.
    #[error("the unix socket path is non-utf8")]
    NonUtf8(#[from] camino::FromPathError),
}

impl TryFrom<MultiSocketAddress> for UnresolvedMultiSocketAddress {
    type Error = UnresolvedMultiSocketAddressTryFromMultiSocketAddressError;

    fn try_from(address: MultiSocketAddress) -> Result<Self, Self::Error> {
        match address {
            MultiSocketAddress::Unix(address) => Ok(UnresolvedMultiSocketAddress::Unix(
                <&camino::Utf8Path>::try_from(address.as_pathname().ok_or(
                    UnresolvedMultiSocketAddressTryFromMultiSocketAddressError::UnnamedUnixSocket,
                )?)?
                .to_path_buf(),
            )),
            MultiSocketAddress::Tcp(address) => {
                Ok(UnresolvedMultiSocketAddress::Tcp(UnresolvedSocketAddress {
                    host: address.ip().to_string(),
                    port: address.port(),
                    is_v6: address.is_ipv6(),
                }))
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{
        UnresolvedMultiSocketAddress, UnresolvedSocketAddress, UnresolvedSocketAddressParseError,
    };
    use std::any::type_name;
    use std::net::{SocketAddr, ToSocketAddrs};
    use std::str::FromStr;

    trait ParseHelper: FromStr<Err: std::fmt::Display> {
        fn parse(string: &str) -> Self {
            match Self::from_str(string) {
                Ok(value) => value,
                Err(error) => {
                    panic!(
                        "failed to parse {string} as {}: {error}",
                        type_name::<Self>()
                    );
                }
            }
        }
    }

    impl<T: FromStr<Err: std::fmt::Display>> ParseHelper for T {}

    fn contains(unresolved: UnresolvedSocketAddress, resolved: SocketAddr) -> bool {
        match unresolved.to_socket_addrs() {
            Ok(mut addresses) => addresses.any(|address| address == resolved),
            Err(err) => panic!("failed to resolve {unresolved}: {err}"),
        }
    }

    #[test]
    fn unresolved_socket_address_parsing() {
        assert!(contains(
            UnresolvedSocketAddress::parse("one.one.one.one:8888"),
            SocketAddr::parse("1.1.1.1:8888"),
        ));

        assert!(contains(
            UnresolvedSocketAddress::parse("one.one.one.one.:8888"),
            SocketAddr::parse("1.1.1.1:8888"),
        ));

        assert!(contains(
            UnresolvedSocketAddress::parse("198.51.100.124:8888"),
            SocketAddr::parse("198.51.100.124:8888"),
        ));

        assert!(contains(
            UnresolvedSocketAddress::parse("[2001:db8::cafe]:8888"),
            SocketAddr::parse("[2001:db8::cafe]:8888"),
        ));

        assert!(matches!(
            UnresolvedSocketAddress::from_str(""),
            Err(UnresolvedSocketAddressParseError::MissingSeparator),
        ));
        assert!(matches!(
            UnresolvedSocketAddress::from_str(":80"),
            Err(UnresolvedSocketAddressParseError::InvalidHostname(_)),
        ));
        assert!(matches!(
            UnresolvedSocketAddress::from_str("example.com"),
            Err(UnresolvedSocketAddressParseError::MissingSeparator),
        ));
        assert!(matches!(
            UnresolvedSocketAddress::from_str("example.com:http"),
            Err(UnresolvedSocketAddressParseError::InvalidPortNumber(_)),
        ));
        assert!(matches!(
            UnresolvedSocketAddress::from_str("-example.com:80"),
            Err(UnresolvedSocketAddressParseError::InvalidHostname(_)),
        ));
        assert!(matches!(
            UnresolvedSocketAddress::from_str("example:com:80"),
            Err(UnresolvedSocketAddressParseError::InvalidHostname(_)),
        ));
        assert!(matches!(
            UnresolvedSocketAddress::from_str("2001:db8::cafe:80"),
            Err(UnresolvedSocketAddressParseError::InvalidHostname(_)),
        ));
        assert!(matches!(
            UnresolvedSocketAddress::from_str("foo-.example.com:80"),
            Err(UnresolvedSocketAddressParseError::InvalidHostname(_)),
        ));
        assert!(matches!(
            UnresolvedSocketAddress::from_str(".example.com:80"),
            Err(UnresolvedSocketAddressParseError::InvalidHostname(_)),
        ));
        assert!(matches!(
            UnresolvedSocketAddress::from_str("foo..example.com:80"),
            Err(UnresolvedSocketAddressParseError::InvalidHostname(_)),
        ));
        assert!(matches!(
            UnresolvedSocketAddress::from_str(
                "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz.example.com:80"
            ),
            Err(UnresolvedSocketAddressParseError::InvalidHostname(_)),
        ));
    }

    #[test]
    fn unresolved_multi_socket_address_parsing() {
        // Absolute Unix socket paths:
        let unix_addr = UnresolvedMultiSocketAddress::parse("/tmp/test.sock");
        assert!(matches!(unix_addr, UnresolvedMultiSocketAddress::Unix(_)));

        // Explicit relative Unix socket paths:
        let relative_path = UnresolvedMultiSocketAddress::parse("./relative/path");
        assert!(matches!(
            relative_path,
            UnresolvedMultiSocketAddress::Unix(_)
        ));

        let parent_relative_path = UnresolvedMultiSocketAddress::parse("../parent/path");
        assert!(matches!(
            parent_relative_path,
            UnresolvedMultiSocketAddress::Unix(_)
        ));

        // TCP addresses:
        let tcp_addr = UnresolvedMultiSocketAddress::parse("localhost:8080");
        assert!(matches!(tcp_addr, UnresolvedMultiSocketAddress::Tcp(_)));

        let ipv4_addr = UnresolvedMultiSocketAddress::parse("127.0.0.1:8080");
        assert!(matches!(ipv4_addr, UnresolvedMultiSocketAddress::Tcp(_)));

        let ipv6_addr = UnresolvedMultiSocketAddress::parse("[::1]:8080");
        assert!(matches!(ipv6_addr, UnresolvedMultiSocketAddress::Tcp(_)));

        // Error cases:
        assert!(UnresolvedMultiSocketAddress::from_str("invalid:tcp:address:format").is_err());
        assert!(UnresolvedMultiSocketAddress::from_str("relative/path").is_err());
        assert!(UnresolvedMultiSocketAddress::from_str("just-a-filename").is_err());
    }
}
