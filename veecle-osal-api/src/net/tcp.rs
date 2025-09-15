//! TCP socket abstractions.
//!
//! To get started, see [`TcpSocket`].

use core::fmt::Display;
use core::fmt::Formatter;
use core::net::SocketAddr;
use embedded_io_async::ErrorKind;

/// TCP socket for establishing connections.
///
/// A socket can only handle one connection at a time.
/// For multiple connections, use multiple socket instances.
/// While no socket is actively listening, incoming connections are rejected via `RST` packet.
/// This differs from the typical behavior of TCP sockets on Linux.
///
/// # Example
///
/// ```no_run
/// use veecle_osal_api::net::tcp::{TcpSocket, TcpConnection};
/// use core::net::SocketAddr;
///
/// async fn connect_example(mut socket: impl TcpSocket)
/// {
///     let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
///     let connection = socket.connect(addr).await.unwrap();
///     // Use connection for reading/writing.
///     connection.close().await;
/// }
/// ```
#[expect(async_fn_in_trait)]
pub trait TcpSocket {
    /// Connects to a remote TCP server.
    async fn connect(&mut self, address: SocketAddr) -> Result<impl TcpConnection, Error>;

    /// Accepts an incoming TCP connection.
    ///
    /// Binds to the specified address and waits for an incoming connection.
    /// Returns the connection and the remote peer's address.
    ///
    /// Listens on all devices if an all-zero IP is provided.
    ///
    /// Does not support using the `0` port to listen on an automatically assigned port.
    ///
    /// # Loopback
    ///
    /// Depending on the platform, the remote peer's address might be set to `127.0.0.1`,
    /// regardless of the actual IPv4 used.
    async fn accept(
        &mut self,
        address: SocketAddr,
    ) -> Result<(impl TcpConnection, SocketAddr), Error>;
}

/// TCP connection for reading and writing data.
///
/// Implements async read and write operations through `embedded_io_async` traits.
///
/// # Example
///
/// ```no_run
/// use embedded_io_async::{Read, Write};
/// use veecle_osal_api::net::tcp::TcpConnection;
///
/// async fn echo_server(mut connection: impl TcpConnection)
/// {
///     let mut buffer = [0u8; 1024];
///     loop {
///         match connection.read(&mut buffer).await {
///             Ok(0) => break, // Connection closed.
///             Ok(read) => {
///                 connection.write_all(&buffer[..read]).await.unwrap();
///                 connection.flush().await.unwrap();
///             }
///             Err(_) => break,
///         }
///     }
///     connection.close().await;
/// }
/// ```
#[expect(async_fn_in_trait)]
pub trait TcpConnection:
    core::fmt::Debug
    + embedded_io_async::Read
    + embedded_io_async::Write
    + embedded_io_async::ErrorType<Error = Error>
{
    /// Closes the write-half of the TCP connection and flushes all unsent data.
    async fn close(self);
}

/// Errors that can occur when using TCP sockets.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Error {
    /// The connection was refused or reset by timeout or RST packet.
    ConnectionReset,
    /// The socket is in an invalid state.
    InvalidState,
    /// The provided port is invalid.
    InvalidPort,
    /// The provided address is invalid.
    ///
    /// This can occur if the address is already in use or doesn't exist.
    InvalidAddress,
    /// The connection timed out.
    TimedOut,
    /// No route to host.
    NoRoute,
    /// No permission to access the resource.
    PermissionDenied,
    /// The network stack is down.
    NetworkDown,
    /// Currently unhandled error occurred.
    /// Please open a bug report if you encounter this error.
    Other,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::ConnectionReset => {
                write!(f, "The connection was reset by timeout of RST packet.")
            }
            Error::InvalidState => {
                write!(f, "The socket is in an invalid state.")
            }
            Error::InvalidPort => {
                write!(f, "The provided port is invalid.")
            }
            Error::TimedOut => {
                write!(f, "The connection timed out.")
            }
            Error::NoRoute => {
                write!(f, "No route to host.")
            }
            Error::Other => {
                write!(
                    f,
                    "Unspecified error, please open a bug report if you encounter this error."
                )
            }
            Error::InvalidAddress => {
                write!(f, "The provided address is invalid.")
            }
            Error::PermissionDenied => {
                write!(f, "No permission to access the resource.")
            }
            Error::NetworkDown => {
                write!(f, "The network stack is down.")
            }
        }
    }
}

impl core::error::Error for Error {}

impl embedded_io_async::ErrorType for Error {
    type Error = Error;
}

impl embedded_io_async::Error for Error {
    fn kind(&self) -> ErrorKind {
        match self {
            Error::ConnectionReset => ErrorKind::ConnectionReset,
            Error::InvalidState => ErrorKind::InvalidInput,
            Error::InvalidPort => ErrorKind::InvalidInput,
            Error::InvalidAddress => ErrorKind::InvalidInput,
            Error::TimedOut => ErrorKind::TimedOut,
            Error::NoRoute => ErrorKind::Other,
            Error::PermissionDenied => ErrorKind::PermissionDenied,
            Error::NetworkDown => ErrorKind::NotConnected,
            Error::Other => ErrorKind::Other,
        }
    }
}

#[doc(hidden)]
#[cfg(feature = "test-suites")]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod test_suite {
    #![expect(missing_docs, reason = "tests")]
    //! Test suite for TCP sockets.

    use crate::net::tcp::{Error, TcpConnection, TcpSocket};
    use embedded_io_async::{Read, Write};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

    pub async fn test_connect(
        mut client: impl TcpSocket,
        mut server: impl TcpSocket,
        ip_address: &str,
    ) {
        let ip_address = ip_address.parse().unwrap();
        let server_addr = SocketAddr::new(ip_address, 59001);

        let server_task = async {
            let (connection, remote_addr) = server.accept(server_addr).await.unwrap();

            // The remote address port should be different from server port.
            // The IP might be normalized (e.g., 127.3.0.1 -> 127.0.0.1) on some platforms.
            assert!(remote_addr.ip().is_loopback() || remote_addr.ip() == ip_address);
            assert_ne!(remote_addr.port(), server_addr.port());
            assert_ne!(remote_addr.port(), 0);

            connection.close().await;
        };

        let client_task = async {
            let connection = loop {
                if let Ok(connection) = client.connect(server_addr).await {
                    break connection;
                }
            };

            connection.close().await;
        };

        futures::join!(server_task, client_task);
    }

    pub async fn test_send_recv(
        mut client: impl TcpSocket,
        mut server: impl TcpSocket,
        ip_address: &str,
    ) {
        let ip_address = ip_address.parse().unwrap();
        let server_addr = SocketAddr::new(ip_address, 59003);

        let server_task = async {
            let (mut connection, _) = server.accept(server_addr).await.unwrap();

            let mut buffer = [0u8; 256];
            let read = connection.read(&mut buffer).await.unwrap();
            assert_eq!(&buffer[..read], b"Test message from client");

            connection.write_all(&buffer[..read]).await.unwrap();
            connection.flush().await.unwrap();

            let read = connection.read(&mut buffer).await.unwrap();
            assert_eq!(&buffer[..read], b"Second message");

            connection.write_all(b"Acknowledged").await.unwrap();
            connection.flush().await.unwrap();

            connection.close().await;
        };

        let client_task = async {
            let mut connection = loop {
                if let Ok(connection) = client.connect(server_addr).await {
                    break connection;
                }
            };

            connection
                .write_all(b"Test message from client")
                .await
                .unwrap();
            connection.flush().await.unwrap();

            let mut buffer = [0u8; 256];
            let read = connection.read(&mut buffer).await.unwrap();
            assert_eq!(&buffer[..read], b"Test message from client");

            connection.write_all(b"Second message").await.unwrap();
            connection.flush().await.unwrap();

            let read = connection.read(&mut buffer).await.unwrap();
            assert_eq!(&buffer[..read], b"Acknowledged");

            connection.close().await;
        };

        futures::join!(server_task, client_task);
    }

    pub async fn test_connect_refused(mut client: impl TcpSocket, ip_address: &str) {
        let ip_address = ip_address.parse().unwrap();
        let server_addr = SocketAddr::new(ip_address, 59900);

        assert_eq!(
            client.connect(server_addr).await.unwrap_err(),
            Error::ConnectionReset
        );
    }

    pub async fn test_accept_with_zero_port(mut server: impl TcpSocket, ip_address: &str) {
        let ip_address = ip_address.parse().unwrap();
        let server_addr = SocketAddr::new(ip_address, 0);

        assert_eq!(
            server.accept(server_addr).await.unwrap_err(),
            Error::InvalidPort
        );
    }

    pub async fn test_accept_all_zero_ip(
        mut client: impl TcpSocket,
        mut server: impl TcpSocket,
        ip_address: &str,
    ) {
        let port = 59910;
        let ip_address: IpAddr = ip_address.parse().unwrap();
        let ip_address = SocketAddr::new(ip_address, port);
        let all_zero_address = if ip_address.is_ipv4() {
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port)
        } else {
            SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), port)
        };

        let server_task = async {
            let (mut connection, _) = server.accept(all_zero_address).await.unwrap();

            let mut buffer = [0u8; 256];
            let read = connection.read(&mut buffer).await.unwrap();
            assert_eq!(&buffer[..read], b"Test message from client");

            connection.write_all(&buffer[..read]).await.unwrap();
            connection.flush().await.unwrap();
        };

        let client_task = async {
            let mut connection = loop {
                if let Ok(connection) = client.connect(ip_address).await {
                    break connection;
                }
            };

            connection
                .write_all(b"Test message from client")
                .await
                .unwrap();
            connection.flush().await.unwrap();

            connection.close().await;
        };

        futures::join!(server_task, client_task);
    }

    pub async fn test_close_connection(
        mut client: impl TcpSocket,
        mut server: impl TcpSocket,
        ip_address: &str,
    ) {
        let ip_address = ip_address.parse().unwrap();
        let server_addr = SocketAddr::new(ip_address, 59004);

        let server_task = async {
            let (mut connection, _) = server.accept(server_addr).await.unwrap();

            connection.write_all(b"Hello").await.unwrap();
            connection.flush().await.unwrap();

            connection.close().await;
        };

        let client_task = async {
            let mut connection = loop {
                if let Ok(connection) = client.connect(server_addr).await {
                    break connection;
                }
            };

            let mut buffer = [0u8; 32];
            let read = connection.read(&mut buffer).await.unwrap();
            assert_eq!(&buffer[..read], b"Hello");

            let read = connection.read(&mut buffer).await.unwrap();
            assert_eq!(
                read, 0,
                "Expected EOF (0 bytes) after server closed connection"
            );

            connection.close().await;
        };

        futures::join!(server_task, client_task);
    }
}
