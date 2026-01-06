//! TCP socket implementation for the std platform.

use crate::IntoOsalError;
use embedded_io_adapters::tokio_1::FromTokio;
use std::io::ErrorKind;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use veecle_osal_api::net::tcp::Error;

/// TCP socket for establishing connections.
///
/// This socket can handle one connection at a time.
/// Create multiple instances for concurrent connections.
#[derive(Default)]
pub struct TcpSocket;

impl TcpSocket {
    /// Creates a new `TcpSocket`.
    pub fn new() -> Self {
        Self
    }
}

impl core::fmt::Debug for TcpSocket {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TcpSocket").finish()
    }
}

/// Active TCP connection for reading and writing data.
///
/// Implements async I/O operations through `embedded_io_async` traits.
/// The connection is automatically closed when dropped.
pub struct TcpConnection<'s> {
    stream: FromTokio<tokio::net::TcpStream>,
    // Prevents multiple concurrent connections from the same socket to fulfill trait contract.
    _socket: &'s mut TcpSocket,
}

impl<'s> core::fmt::Debug for TcpConnection<'s> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TcpConnection").finish()
    }
}

impl<'s> veecle_osal_api::net::tcp::TcpConnection for TcpConnection<'s> {
    async fn close(self) {
        // Any error isn't actionable here.
        let _ = self.stream.into_inner().shutdown().await;
    }
}

impl<'s> embedded_io_async::Read for TcpConnection<'s> {
    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        self.stream
            .read(buffer)
            .await
            .map_err(IntoOsalError::into_osal_error)
    }
}

impl<'s> embedded_io::ErrorType for TcpConnection<'s> {
    type Error = Error;
}

impl<'s> embedded_io_async::Write for TcpConnection<'s> {
    async fn write(&mut self, buffer: &[u8]) -> Result<usize, Self::Error> {
        self.stream
            .write(buffer)
            .await
            .map_err(IntoOsalError::into_osal_error)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        embedded_io_async::Write::flush(&mut self.stream)
            .await
            .map_err(IntoOsalError::into_osal_error)
    }
}

impl veecle_osal_api::net::tcp::TcpSocket for TcpSocket {
    async fn connect(
        &mut self,
        address: SocketAddr,
    ) -> Result<impl veecle_osal_api::net::tcp::TcpConnection, Error> {
        let stream = tokio::net::TcpStream::connect(address)
            .await
            .map_err(IntoOsalError::into_osal_error)?;
        Ok(TcpConnection {
            stream: FromTokio::new(stream),
            _socket: self,
        })
    }

    async fn accept(
        &mut self,
        address: SocketAddr,
    ) -> Result<(impl veecle_osal_api::net::tcp::TcpConnection, SocketAddr), Error> {
        // Required to match the trait contract.
        if address.port() == 0 {
            return Err(Error::InvalidPort);
        }

        let socket = if address.ip().is_ipv4() {
            tokio::net::TcpSocket::new_v4().map_err(IntoOsalError::into_osal_error)?
        } else {
            tokio::net::TcpSocket::new_v6().map_err(IntoOsalError::into_osal_error)?
        };

        // The trait contract requires allowing multiple sockets to accept connections on the same address and port.
        socket
            .set_reuseaddr(true)
            .map_err(IntoOsalError::into_osal_error)?;
        socket
            .set_reuseport(true)
            .map_err(IntoOsalError::into_osal_error)?;

        socket
            .bind(address)
            .map_err(IntoOsalError::into_osal_error)?;

        let listener = socket.listen(1).map_err(IntoOsalError::into_osal_error)?;

        let (stream, address) = listener
            .accept()
            .await
            .map_err(IntoOsalError::into_osal_error)?;
        Ok((
            TcpConnection {
                stream: FromTokio::new(stream),
                _socket: self,
            },
            address,
        ))
    }
}

impl IntoOsalError<Error> for std::io::Error {
    fn into_osal_error(self) -> Error {
        match self.kind() {
            ErrorKind::PermissionDenied => Error::PermissionDenied,
            ErrorKind::ConnectionRefused => Error::ConnectionReset,
            ErrorKind::ConnectionReset => Error::ConnectionReset,
            ErrorKind::HostUnreachable => Error::NoRoute,
            ErrorKind::NetworkUnreachable => Error::NoRoute,
            ErrorKind::ConnectionAborted => Error::ConnectionReset,
            ErrorKind::AddrInUse => Error::InvalidAddress,
            ErrorKind::AddrNotAvailable => Error::InvalidAddress,
            ErrorKind::NetworkDown => Error::NetworkDown,
            ErrorKind::TimedOut => Error::TimedOut,
            _ => Error::Other,
        }
    }
}
