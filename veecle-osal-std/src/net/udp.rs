//! UDP socket implementation for the std platform.

use crate::IntoOsalError;
use core::net::SocketAddr;
use socket2::{Protocol, SockAddr, Type};
use std::io::ErrorKind;
use veecle_osal_api::net::udp::Error;

/// UDP socket for sending and receiving datagrams.
#[derive(Default)]
pub struct UdpSocket {
    /// The underlying socket.
    ///
    /// If this is `Some`, the socket is in use and cannot be used to `bind` or `send`.
    socket: Option<tokio::net::UdpSocket>,
}

impl core::fmt::Debug for UdpSocket {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UdpSocket").finish()
    }
}

impl UdpSocket {
    /// Creates a new `UdpSocket`.
    pub fn new() -> Self {
        Self { socket: None }
    }
}

impl veecle_osal_api::net::udp::UdpSocket for UdpSocket {
    async fn bind(&mut self, address: SocketAddr) -> Result<(), Error> {
        if self.socket.is_some() {
            return Err(Error::InvalidState);
        }

        // We need to use `socket2´ to set `SO_REUSEADDR` and `SO_REUSEPORT`,
        // because neither Tokio nor the standard library support this
        // for UDP sockets.

        let socket2_socket = socket2::Socket::new(
            socket2::Domain::for_address(address),
            Type::DGRAM,
            Some(Protocol::UDP),
        )
        .map_err(IntoOsalError::into_osal_error)?;

        socket2_socket
            .set_reuse_address(true)
            .map_err(IntoOsalError::into_osal_error)?;
        socket2_socket
            .set_reuse_port(true)
            .map_err(IntoOsalError::into_osal_error)?;
        socket2_socket
            .set_nonblocking(true)
            .map_err(IntoOsalError::into_osal_error)?;

        let socket2_addr: SockAddr = address.into();
        socket2_socket
            .bind(&socket2_addr)
            .map_err(IntoOsalError::into_osal_error)?;

        let tokio_socket = tokio::net::UdpSocket::from_std(socket2_socket.into())
            .map_err(IntoOsalError::into_osal_error)?;

        self.socket = Some(tokio_socket);
        Ok(())
    }

    fn local_addr(&self) -> Result<SocketAddr, Error> {
        self.socket
            .as_ref()
            .ok_or(Error::SocketNotBound)?
            .local_addr()
            .map_err(IntoOsalError::into_osal_error)
    }

    async fn recv_from(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), Error> {
        let Some(socket) = self.socket.as_ref() else {
            return Err(Error::SocketNotBound);
        };

        let (read, address) = socket
            .recv_from(buffer)
            .await
            .map_err(IntoOsalError::into_osal_error)?;

        Ok((read, address))
    }

    async fn send_to(&self, buffer: &[u8], address: SocketAddr) -> Result<usize, Error> {
        let Some(socket) = self.socket.as_ref() else {
            return Err(Error::SocketNotBound);
        };

        let read = socket
            .send_to(buffer, address)
            .await
            .map_err(IntoOsalError::into_osal_error)?;

        Ok(read)
    }

    fn close(&mut self) {
        self.socket = None;
    }
}

impl IntoOsalError<Error> for std::io::Error {
    fn into_osal_error(self) -> Error {
        match self.kind() {
            ErrorKind::PermissionDenied => Error::PermissionDenied,
            ErrorKind::HostUnreachable => Error::NoRoute,
            ErrorKind::NetworkUnreachable => Error::NoRoute,
            ErrorKind::AddrInUse => Error::InvalidAddress,
            ErrorKind::AddrNotAvailable => Error::InvalidAddress,
            ErrorKind::NetworkDown => Error::NetworkDown,
            _ => Error::Other,
        }
    }
}
