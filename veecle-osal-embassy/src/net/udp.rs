//! UDP socket implementation for Embassy.

use crate::IntoOsalError;
use core::net::IpAddr;
use core::net::SocketAddr;
use embassy_net::IpAddress;
use embassy_net::udp::{BindError, RecvError, SendError};
use veecle_osal_api::net::udp::Error;

/// UDP socket for sending and receiving datagrams.
pub struct UdpSocket<'a> {
    socket: embassy_net::udp::UdpSocket<'a>,
    /// Whether the socket is bound.
    is_bound: bool,
}

impl<'a> core::fmt::Debug for UdpSocket<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UdpSocket").finish()
    }
}

impl<'a> UdpSocket<'a> {
    /// Creates a new `UdpSocket`.
    ///
    /// The socket must be closed.
    pub fn new(socket: embassy_net::udp::UdpSocket<'a>) -> Result<Self, Error> {
        if socket.is_open() {
            return Err(Error::InvalidState);
        }
        Ok(Self {
            socket,
            is_bound: false,
        })
    }
}

impl<'a> veecle_osal_api::net::udp::UdpSocket for UdpSocket<'a> {
    async fn bind(&mut self, address: SocketAddr) -> Result<(), Error> {
        self.socket
            .bind(address)
            .map_err(IntoOsalError::into_osal_error)?;
        self.is_bound = true;
        Ok(())
    }

    fn local_addr(&self) -> Result<SocketAddr, Error> {
        self.socket
            .endpoint()
            .addr
            .ok_or(Error::SocketNotBound)
            .map(|address| {
                let address = match address {
                    IpAddress::Ipv4(address) => IpAddr::V4(address),
                    IpAddress::Ipv6(address) => IpAddr::V6(address),
                };
                SocketAddr::new(address, self.socket.endpoint().port)
            })
    }

    async fn recv_from(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), Error> {
        if !self.is_bound {
            return Err(Error::SocketNotBound);
        }
        let (read, metadata) = self
            .socket
            .recv_from(buffer)
            .await
            .map_err(IntoOsalError::into_osal_error)?;
        let address = match metadata.endpoint.addr {
            IpAddress::Ipv4(address) => IpAddr::V4(address),
            IpAddress::Ipv6(address) => IpAddr::V6(address),
        };
        let address: SocketAddr = SocketAddr::new(address, metadata.endpoint.port);
        Ok((read, address))
    }

    async fn send_to(&self, buffer: &[u8], address: SocketAddr) -> Result<usize, Error> {
        if !self.is_bound {
            return Err(Error::SocketNotBound);
        }
        self.socket
            .send_to(buffer, address)
            .await
            .map_err(IntoOsalError::into_osal_error)?;
        Ok(buffer.len())
    }

    fn close(&mut self) {
        self.socket.close();
        self.is_bound = false;
    }
}

impl IntoOsalError<Error> for BindError {
    fn into_osal_error(self) -> Error {
        match self {
            BindError::InvalidState => Error::InvalidState,
            BindError::NoRoute => Error::NoRoute,
        }
    }
}

impl IntoOsalError<Error> for RecvError {
    fn into_osal_error(self) -> Error {
        match self {
            RecvError::Truncated => Error::BufferTooSmall,
        }
    }
}

impl IntoOsalError<Error> for SendError {
    fn into_osal_error(self) -> Error {
        match self {
            SendError::NoRoute => Error::NoRoute,
            SendError::SocketNotBound => Error::SocketNotBound,
            SendError::PacketTooLarge => Error::BufferTooLarge,
        }
    }
}
