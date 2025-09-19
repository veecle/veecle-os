//! Veecle OS Orchestrator.
//!
//! Some small librarified but internal components to make them testable.

#![doc(hidden)]

/// A parsed-but-not-resolved `SocketAddr`-like to support hostnames.
///
/// See tests for what is supported.
#[derive(Debug, Clone)]
pub struct UnresolvedSocketAddr {
    /// Either a hostname, IPv4 or IPv6 address.
    host: String,

    // IPv6 is the only case where `SocketAddr` syntax is not just `{host}:{port}`, it has extra
    // `[]`` around the IP to distinguish from the port number.
    is_v6: bool,

    port: u16,
}

impl UnresolvedSocketAddr {
    /// Returns this as something that implements `tokio::net::ToSocketAddrs`.
    ///
    /// This type would directly implement the trait, but it's sealed.
    pub fn as_to_socket_addrs(&self) -> impl tokio::net::ToSocketAddrs {
        (self.host.as_str(), self.port)
    }
}

impl std::net::ToSocketAddrs for UnresolvedSocketAddr {
    // Can't use `(&str, u16)` here because of the lifetime.
    type Iter = <(String, u16) as std::net::ToSocketAddrs>::Iter;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        (self.host.as_str(), self.port).to_socket_addrs()
    }
}

impl std::fmt::Display for UnresolvedSocketAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let UnresolvedSocketAddr { host, is_v6, port } = self;
        if *is_v6 {
            write!(f, "[{host}]:{port}")?;
        } else {
            write!(f, "{host}:{port}")?;
        }
        Ok(())
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum UnresolvedSocketAddrError {
    #[error("missing ':' separator")]
    MissingSeparator,

    #[error("invalid port number")]
    InvalidPortNumber(#[source] std::num::ParseIntError),

    #[error("invalid hostname: {0}")]
    InvalidHostname(&'static str),
}

impl std::str::FromStr for UnresolvedSocketAddr {
    type Err = UnresolvedSocketAddrError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let (host, port) = string
            .rsplit_once(":")
            .ok_or(UnresolvedSocketAddrError::MissingSeparator)?;

        let port = port
            .parse()
            .map_err(UnresolvedSocketAddrError::InvalidPortNumber)?;

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
        fn validate_hostname(host: &str) -> Result<(), UnresolvedSocketAddrError> {
            if host.is_empty() {
                return Err(UnresolvedSocketAddrError::InvalidHostname("is empty"));
            }

            if host.len() > 253 {
                return Err(UnresolvedSocketAddrError::InvalidHostname("is too long"));
            }

            if host.starts_with('.') {
                return Err(UnresolvedSocketAddrError::InvalidHostname(
                    "starts with period",
                ));
            }

            // Strip a trailing `.` to allow for FQDN.
            for label in host.strip_suffix('.').unwrap_or(host).split('.') {
                if label.is_empty() {
                    return Err(UnresolvedSocketAddrError::InvalidHostname(
                        "contains empty label",
                    ));
                }

                if label.len() > 63 {
                    return Err(UnresolvedSocketAddrError::InvalidHostname("label too long"));
                }

                if label.starts_with('-') {
                    return Err(UnresolvedSocketAddrError::InvalidHostname(
                        "label starts with dash",
                    ));
                }

                if label.ends_with('-') {
                    return Err(UnresolvedSocketAddrError::InvalidHostname(
                        "label ends with dash",
                    ));
                }

                if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                    return Err(UnresolvedSocketAddrError::InvalidHostname(
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

#[cfg(test)]
mod tests {
    use super::{UnresolvedSocketAddr, UnresolvedSocketAddrError};
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

    fn contains(unresolved: UnresolvedSocketAddr, resolved: SocketAddr) -> bool {
        match unresolved.to_socket_addrs() {
            Ok(mut addresses) => addresses.any(|address| address == resolved),
            Err(err) => panic!("failed to resolve {unresolved}: {err}"),
        }
    }

    #[test]
    fn smoke_test() {
        assert!(contains(
            UnresolvedSocketAddr::parse("one.one.one.one:8888"),
            SocketAddr::parse("1.1.1.1:8888"),
        ));

        assert!(contains(
            UnresolvedSocketAddr::parse("one.one.one.one.:8888"),
            SocketAddr::parse("1.1.1.1:8888"),
        ));

        assert!(contains(
            UnresolvedSocketAddr::parse("198.51.100.124:8888"),
            SocketAddr::parse("198.51.100.124:8888"),
        ));

        assert!(contains(
            UnresolvedSocketAddr::parse("[2001:db8::cafe]:8888"),
            SocketAddr::parse("[2001:db8::cafe]:8888"),
        ));

        assert!(matches!(
            UnresolvedSocketAddr::from_str(""),
            Err(UnresolvedSocketAddrError::MissingSeparator),
        ));
        assert!(matches!(
            UnresolvedSocketAddr::from_str(":80"),
            Err(UnresolvedSocketAddrError::InvalidHostname(_)),
        ));
        assert!(matches!(
            UnresolvedSocketAddr::from_str("example.com"),
            Err(UnresolvedSocketAddrError::MissingSeparator),
        ));
        assert!(matches!(
            UnresolvedSocketAddr::from_str("example.com:http"),
            Err(UnresolvedSocketAddrError::InvalidPortNumber(_)),
        ));
        assert!(matches!(
            UnresolvedSocketAddr::from_str("-example.com:80"),
            Err(UnresolvedSocketAddrError::InvalidHostname(_)),
        ));
        assert!(matches!(
            UnresolvedSocketAddr::from_str("example:com:80"),
            Err(UnresolvedSocketAddrError::InvalidHostname(_)),
        ));
        assert!(matches!(
            UnresolvedSocketAddr::from_str("2001:db8::cafe:80"),
            Err(UnresolvedSocketAddrError::InvalidHostname(_)),
        ));
    }
}
