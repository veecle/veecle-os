//! TCP socket implementation for Embassy.

use crate::IntoOsalError;
use core::net::IpAddr;
use core::net::SocketAddr;
use embassy_net::tcp::{AcceptError, ConnectError, State};
use embassy_net::{IpAddress, IpEndpoint, IpListenEndpoint};
use veecle_osal_api::net::tcp::Error;

/// TCP socket for establishing connections.
///
/// This socket can handle one connection at a time.
/// Create multiple instances for concurrent connections.
pub struct TcpSocket<'a> {
    socket: embassy_net::tcp::TcpSocket<'a>,
}

impl<'a> core::fmt::Debug for TcpSocket<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TcpSocket").finish()
    }
}

impl<'a> TcpSocket<'a> {
    /// Creates a new `TcpSocket`.
    ///
    /// The socket must be closed.
    pub fn new(socket: embassy_net::tcp::TcpSocket<'a>) -> Result<Self, Error> {
        if socket.state() != State::Closed {
            return Err(Error::InvalidState);
        }
        Ok(Self { socket })
    }
}

/// Active TCP connection for reading and writing data.
///
/// Implements async I/O operations through `embedded_io_async` traits.
///
/// The connection is automatically closed when dropped.
/// On drop, the remote endpoint may or may not be informed about the connection terminating.
/// To cleanly close a connection, use [`veecle_osal_api::net::tcp::TcpConnection::close`].
pub struct TcpConnection<'a, 's> {
    socket: &'s mut embassy_net::tcp::TcpSocket<'a>,
}

impl<'a, 's> core::fmt::Debug for TcpConnection<'a, 's> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TcpConnection").finish()
    }
}

impl Drop for TcpConnection<'_, '_> {
    fn drop(&mut self) {
        if self.socket.state() != State::Closed {
            self.socket.close();
            self.socket.abort();
        }
    }
}

impl<'a, 's> embedded_io_async::Read for TcpConnection<'a, 's> {
    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        self.socket
            .read(buffer)
            .await
            .map_err(IntoOsalError::into_osal_error)
    }
}

impl<'a, 's> embedded_io::ErrorType for TcpConnection<'a, 's> {
    type Error = Error;
}

impl<'a, 's> embedded_io_async::Write for TcpConnection<'a, 's> {
    async fn write(&mut self, buffer: &[u8]) -> Result<usize, Self::Error> {
        self.socket
            .write(buffer)
            .await
            .map_err(IntoOsalError::into_osal_error)
    }
}

impl<'a, 's> veecle_osal_api::net::tcp::TcpConnection for TcpConnection<'a, 's> {
    async fn close(self) {
        self.socket.close();
        // We need to wait until the socket has been flushed to be able to reuse it.
        // The only error that can occur is `ConnectionReset`, which isn't actionable in `close`.
        let _ = self.socket.flush().await;
    }
}

impl<'a> veecle_osal_api::net::tcp::TcpSocket for TcpSocket<'a> {
    async fn connect(
        &mut self,
        address: SocketAddr,
    ) -> Result<impl veecle_osal_api::net::tcp::TcpConnection, Error> {
        self.socket
            .connect(address)
            .await
            .map_err(IntoOsalError::into_osal_error)?;
        Ok(TcpConnection {
            socket: &mut self.socket,
        })
    }

    async fn accept(
        &mut self,
        address: SocketAddr,
    ) -> Result<(impl veecle_osal_api::net::tcp::TcpConnection, SocketAddr), Error> {
        // smoltcp treats an all-zero address as invalid, so we need to convert it to `None`.
        let listen_endpoint = if address.ip().is_unspecified() {
            IpListenEndpoint {
                addr: None,
                port: address.port(),
            }
        } else {
            address.into()
        };

        self.socket
            .accept(listen_endpoint)
            .await
            .map_err(IntoOsalError::into_osal_error)?;
        let IpEndpoint {
            addr: address,
            port,
        } = self
            .socket
            .remote_endpoint()
            .expect("The endpoint should be set after accepting a connection.");

        let address = match address {
            IpAddress::Ipv4(address) => IpAddr::V4(address),
            IpAddress::Ipv6(address) => IpAddr::V6(address),
        };
        let address: SocketAddr = SocketAddr::new(address, port);
        Ok((
            TcpConnection {
                socket: &mut self.socket,
            },
            address,
        ))
    }
}

impl IntoOsalError<Error> for AcceptError {
    fn into_osal_error(self) -> Error {
        match self {
            AcceptError::ConnectionReset => Error::ConnectionReset,
            AcceptError::InvalidState => Error::InvalidState,
            AcceptError::InvalidPort => Error::InvalidPort,
        }
    }
}

impl IntoOsalError<Error> for ConnectError {
    fn into_osal_error(self) -> Error {
        match self {
            ConnectError::InvalidState => Error::InvalidState,
            ConnectError::ConnectionReset => Error::ConnectionReset,
            ConnectError::TimedOut => Error::TimedOut,
            ConnectError::NoRoute => Error::NoRoute,
        }
    }
}

impl IntoOsalError<Error> for embassy_net::tcp::Error {
    fn into_osal_error(self) -> Error {
        match self {
            embassy_net::tcp::Error::ConnectionReset => Error::ConnectionReset,
        }
    }
}
