//! UDP socket abstractions.

use core::fmt::Display;
use core::fmt::Formatter;
use core::net::SocketAddr;

/// UDP socket for sending and receiving datagrams.
///
/// UDP is connectionless - each send/receive operation can target different addresses.
///
/// # Example
///
/// ```no_run
/// use veecle_osal_api::net::udp::UdpSocket;
/// use core::net::SocketAddr;
///
/// async fn udp_echo<S>(mut socket: S)
/// where
///     S: UdpSocket
/// {
///     let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
///     socket.bind(addr).await.unwrap();
///
///     let mut buffer = [0u8; 1500];
///     loop {
///         let (size, peer) = socket.recv_from(&mut buffer).await.unwrap();
///         socket.send_to(&buffer[..size], peer).await.unwrap();
///     }
/// }
/// ```
#[expect(async_fn_in_trait)]
pub trait UdpSocket {
    /// Binds the socket to a local address.
    ///
    /// If the specified port is `0`, the port is assigned automatically and can be queried with [`Self::local_addr`].
    async fn bind(&mut self, address: SocketAddr) -> Result<(), Error>;

    /// Returns the local address this socket is bound to.
    fn local_addr(&self) -> Result<SocketAddr, Error>;

    /// Receives a datagram.
    ///
    /// Returns the number of bytes received and the sender's address.
    /// If the datagram is larger than the buffer, excess bytes may be discarded.
    async fn recv_from(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), Error>;

    /// Sends a datagram to the specified address.
    ///
    /// Returns the number of bytes sent.
    async fn send_to(&self, buffer: &[u8], address: SocketAddr) -> Result<usize, Error>;

    /// Closes the UDP socket.
    fn close(&mut self);
}

/// Errors that can occur when using UDP sockets.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Error {
    /// The provided buffer was too small for the received datagram.
    ///
    /// The datagram may have been dropped.
    BufferTooSmall,
    /// The provided buffer was too large for internal buffer.
    BufferTooLarge,
    /// The socket is in an invalid state.
    InvalidState,
    /// No permission to access the resource.
    PermissionDenied,
    /// No route to host.
    NoRoute,
    /// The provided port is invalid.
    InvalidPort,
    /// The provided address is invalid.
    ///
    /// This can occur if the address is already in use or doesn't exist.
    InvalidAddress,
    /// The network stack is down.
    NetworkDown,
    /// The socket is not bound to an outgoing address and port.
    SocketNotBound,
    /// Currently unhandled error occurred.
    /// Please open a bug report if you encounter this error.
    Other,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidState => {
                write!(f, "The socket is in an invalid state.")
            }
            Error::BufferTooSmall => {
                write!(
                    f,
                    "The provided buffer was too small for the received datagram."
                )
            }
            Error::BufferTooLarge => {
                write!(f, "The provided buffer was too large for internal buffer.")
            }
            Error::NoRoute => {
                write!(f, "No route to host.")
            }
            Error::PermissionDenied => {
                write!(f, "No permission to access the resource.")
            }
            Error::InvalidPort => {
                write!(f, "The provided port is invalid.")
            }
            Error::InvalidAddress => {
                write!(f, "The provided address is invalid.")
            }
            Error::NetworkDown => {
                write!(f, "The network stack is down.")
            }
            Error::Other => {
                write!(
                    f,
                    "Unspecified error, please open a bug report if you encounter this error."
                )
            }
            Error::SocketNotBound => {
                write!(
                    f,
                    "The socket is not bound to an outgoing address and port."
                )
            }
        }
    }
}

impl core::error::Error for Error {}

#[doc(hidden)]
#[cfg(feature = "test-suites")]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod test_suite {
    #![expect(missing_docs, reason = "tests")]
    //! Test suite for UDP sockets.

    use crate::net::udp::{Error, UdpSocket};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

    pub async fn test_bind_all_zero_address_v4(mut socket: impl UdpSocket) {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

        socket.bind(address).await.unwrap();

        let bound_addr = socket.local_addr().unwrap();

        assert_eq!(bound_addr.ip(), IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));

        assert_ne!(
            bound_addr.port(),
            0,
            "port should be automatically assigned"
        );

        socket.close();
    }

    pub async fn test_bind_all_zero_address_v6(mut socket: impl UdpSocket) {
        let address = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 0);

        socket.bind(address).await.unwrap();

        let bound_addr = socket.local_addr().unwrap();

        assert_eq!(bound_addr.ip(), IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED));

        assert_ne!(
            bound_addr.port(),
            0,
            "port should be automatically assigned"
        );

        socket.close();
    }

    pub async fn test_bind_specific_port(mut socket: impl UdpSocket, ip_address: &str) {
        let ip_address = ip_address.parse().unwrap();
        let address = SocketAddr::new(ip_address, 58080);

        socket.bind(address).await.unwrap();

        let bound_addr = socket.local_addr().unwrap();

        assert_eq!(bound_addr.ip(), ip_address);
        assert_eq!(bound_addr.port(), 58080, "port should match requested port");

        socket.close();
    }

    pub async fn test_send_recv(
        mut socket1: impl UdpSocket,
        mut socket2: impl UdpSocket,
        ip_address: &str,
    ) {
        let ip_address = ip_address.parse().unwrap();
        let addr1 = SocketAddr::new(ip_address, 58081);
        let addr2 = SocketAddr::new(ip_address, 58082);

        socket1.bind(addr1).await.unwrap();
        socket2.bind(addr2).await.unwrap();

        let send_data = b"Hello, UDP!";
        let mut recv_buffer = [0u8; 64];

        let sent = socket1.send_to(send_data, addr2).await.unwrap();
        assert_eq!(sent, send_data.len());

        let (received, sender_addr) = socket2.recv_from(&mut recv_buffer).await.unwrap();
        assert_eq!(received, send_data.len());
        assert_eq!(&recv_buffer[..received], send_data);
        assert_eq!(sender_addr, addr1);

        let response = b"Hello back!";
        let sent = socket2.send_to(response, addr1).await.unwrap();
        assert_eq!(sent, response.len());

        let (received, sender_addr) = socket1.recv_from(&mut recv_buffer).await.unwrap();
        assert_eq!(received, response.len());
        assert_eq!(&recv_buffer[..received], response);
        assert_eq!(sender_addr, addr2);

        socket1.close();
        socket2.close();
    }

    pub async fn test_local_addr_before_bind(socket: impl UdpSocket) {
        assert_eq!(socket.local_addr(), Err(Error::SocketNotBound));
    }

    pub async fn test_close_socket(mut socket: impl UdpSocket, ip_address: &str) {
        let ip_address = ip_address.parse().unwrap();
        let address = SocketAddr::new(ip_address, 58085);

        socket.bind(address).await.unwrap();
        let bound_addr = socket.local_addr().unwrap();
        assert_eq!(bound_addr, address);

        socket.close();

        assert_eq!(socket.local_addr(), Err(Error::SocketNotBound));
    }

    pub async fn test_recv_without_bind(socket: impl UdpSocket) {
        let mut buffer = [0u8; 64];
        assert_eq!(
            socket.recv_from(&mut buffer).await,
            Err(Error::SocketNotBound)
        );
    }

    pub async fn test_send_without_bind(socket: impl UdpSocket, ip_address: &str) {
        let ip_address = ip_address.parse().unwrap();
        let target_addr = SocketAddr::new(ip_address, 58086);
        let data = b"test data";

        assert_eq!(
            socket.send_to(data, target_addr).await,
            Err(Error::SocketNotBound)
        );
    }

    pub async fn test_bind_multiple_sockets_same_ip(
        mut socket1: impl UdpSocket,
        mut socket2: impl UdpSocket,
        ip_address: &str,
    ) {
        let ip_address = ip_address.parse().unwrap();
        let address = SocketAddr::new(ip_address, 58087);

        socket1.bind(address).await.unwrap();
        socket2.bind(address).await.unwrap();
    }

    pub async fn test_zero_length_datagram(
        mut socket1: impl UdpSocket,
        mut socket2: impl UdpSocket,
        ip_address: &str,
    ) {
        let ip_address = ip_address.parse().unwrap();
        let addr1 = SocketAddr::new(ip_address, 58088);
        let addr2 = SocketAddr::new(ip_address, 58089);

        socket1.bind(addr1).await.unwrap();
        socket2.bind(addr2).await.unwrap();

        let empty_data = &[];
        let sent = socket1.send_to(empty_data, addr2).await.unwrap();
        assert_eq!(sent, 0);

        let mut recv_buffer = [0u8; 64];
        let (received, sender_addr) = socket2.recv_from(&mut recv_buffer).await.unwrap();
        assert_eq!(received, 0);
        assert_eq!(sender_addr, addr1);

        socket1.close();
        socket2.close();
    }

    pub async fn test_max_datagram_size(
        mut socket1: impl UdpSocket,
        mut socket2: impl UdpSocket,
        ip_address: &str,
    ) {
        let ip_address = ip_address.parse().unwrap();
        let addr1 = SocketAddr::new(ip_address, 58090);
        let addr2 = SocketAddr::new(ip_address, 58091);

        socket1.bind(addr1).await.unwrap();
        socket2.bind(addr2).await.unwrap();

        // Test maximum practical UDP payload size.
        // 65507 = 65535 - 8 (UDP header) - 20 (IPv4 header).
        const MAX_SIZE: usize = 65507;
        let mut send_data = [0u8; MAX_SIZE];
        for (i, byte) in send_data.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }

        let sent = socket1.send_to(&send_data, addr2).await.unwrap();
        assert_eq!(sent, MAX_SIZE);

        let mut recv_buffer = [0u8; MAX_SIZE];
        let (received, sender_addr) = socket2.recv_from(&mut recv_buffer).await.unwrap();
        assert_eq!(received, MAX_SIZE);
        assert_eq!(&recv_buffer[..], &send_data[..]);
        assert_eq!(sender_addr, addr1);

        socket1.close();
        socket2.close();
    }

    pub async fn test_multiple_binds(mut socket: impl UdpSocket, ip_address: &str) {
        let ip_address = ip_address.parse().unwrap();
        let addr1 = SocketAddr::new(ip_address, 58092);
        let addr2 = SocketAddr::new(ip_address, 58093);

        socket.bind(addr1).await.unwrap();
        assert_eq!(socket.local_addr().unwrap(), addr1);

        assert_eq!(socket.bind(addr2).await, Err(Error::InvalidState));

        socket.close();
    }

    pub async fn test_rebind_after_close(mut socket: impl UdpSocket, ip_address: &str) {
        let ip_address = ip_address.parse().unwrap();
        let addr1 = SocketAddr::new(ip_address, 58094);
        let addr2 = SocketAddr::new(ip_address, 58095);

        socket.bind(addr1).await.unwrap();
        assert_eq!(socket.local_addr().unwrap(), addr1);

        socket.close();

        assert_eq!(socket.local_addr(), Err(Error::SocketNotBound));

        socket.bind(addr2).await.unwrap();
        assert_eq!(socket.local_addr().unwrap(), addr2);

        socket.close();
    }
}
