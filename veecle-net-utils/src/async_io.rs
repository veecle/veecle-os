//! Async socket I/O operations.
//!
//! Async socket streams, listeners, and connection methods for both Unix domain sockets and TCP
//! sockets using Tokio.

use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};

use camino::{Utf8Path, Utf8PathBuf};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs, UnixListener, UnixStream};

use crate::{MultiSocketAddress, UnresolvedMultiSocketAddress, UnresolvedSocketAddress};

/// A wrapper for [`UnixListener`] that handles removing the socket from the filesystem
/// on drop.
///
/// See [`UnixListener`] for API docs.
#[derive(Debug)]
pub struct AsyncUnixListener {
    path: Utf8PathBuf,
    inner: UnixListener,
}

impl Drop for AsyncUnixListener {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_file(&self.path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::error!(?error, ?self.path, "error removing unix socket");
        }
    }
}

impl AsyncUnixListener {
    /// Binds to a Unix domain socket path.
    pub fn bind(path: impl AsRef<Utf8Path>) -> Result<Self> {
        let path = path.as_ref();
        let inner = UnixListener::bind(path.as_std_path())?;
        Ok(Self {
            path: path.to_owned(),
            inner,
        })
    }

    /// Accepts an incoming connection.
    pub async fn accept(&self) -> Result<(UnixStream, tokio::net::unix::SocketAddr)> {
        self.inner.accept().await
    }
}

/// Async socket listener that can be either Unix or TCP.
#[derive(Debug)]
pub enum AsyncSocketListener {
    /// Unix domain socket listener.
    Unix(AsyncUnixListener),

    /// TCP socket listener.
    Tcp(TcpListener),
}

/// Async socket stream that can be either Unix or TCP.
#[derive(Debug)]
pub enum AsyncSocketStream {
    /// Unix domain socket stream.
    Unix(UnixStream),

    /// TCP socket stream.
    Tcp(TcpStream),
}

impl UnresolvedSocketAddress {
    /// Returns this as something that can be used with Tokio's [`ToSocketAddrs`].
    ///
    /// `ToSocketAddrs` is sealed so we must return a helper instead of implementing it directly.
    pub fn as_to_socket_addrs(&self) -> impl ToSocketAddrs {
        (self.host.as_str(), self.port)
    }
}

impl UnresolvedMultiSocketAddress {
    /// Binds this address as an async listener.
    pub async fn bind_async(&self) -> Result<AsyncSocketListener> {
        match self {
            UnresolvedMultiSocketAddress::Unix(path) => {
                let listener = AsyncUnixListener::bind(path)?;
                Ok(AsyncSocketListener::Unix(listener))
            }
            UnresolvedMultiSocketAddress::Tcp(socket) => {
                let listener = TcpListener::bind(socket.as_to_socket_addrs()).await?;
                Ok(AsyncSocketListener::Tcp(listener))
            }
        }
    }

    /// Connects to this address as an async socket.
    pub async fn connect_async(&self) -> Result<AsyncSocketStream> {
        match self {
            UnresolvedMultiSocketAddress::Unix(path) => {
                let listener = UnixStream::connect(path).await?;
                Ok(AsyncSocketStream::Unix(listener))
            }
            UnresolvedMultiSocketAddress::Tcp(socket) => {
                let listener = TcpStream::connect(socket.as_to_socket_addrs()).await?;
                Ok(AsyncSocketStream::Tcp(listener))
            }
        }
    }
}

impl AsyncSocketListener {
    /// Accepts an incoming connection.
    pub async fn accept(&self) -> Result<(AsyncSocketStream, MultiSocketAddress)> {
        match self {
            AsyncSocketListener::Unix(listener) => {
                let (stream, address) = listener.accept().await?;
                Ok((
                    AsyncSocketStream::Unix(stream),
                    MultiSocketAddress::Unix(address.into()),
                ))
            }
            AsyncSocketListener::Tcp(listener) => {
                let (stream, address) = listener.accept().await?;
                Ok((
                    AsyncSocketStream::Tcp(stream),
                    MultiSocketAddress::Tcp(address),
                ))
            }
        }
    }

    /// Returns the local address this listener is bound to.
    pub fn local_address(&self) -> Result<MultiSocketAddress> {
        match self {
            AsyncSocketListener::Unix(listener) => Ok(MultiSocketAddress::Unix(
                listener.inner.local_addr()?.into(),
            )),
            AsyncSocketListener::Tcp(listener) => {
                Ok(MultiSocketAddress::Tcp(listener.local_addr()?))
            }
        }
    }
}

impl AsyncRead for AsyncSocketStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        match &mut *self {
            AsyncSocketStream::Unix(stream) => Pin::new(stream).poll_read(context, buffer),
            AsyncSocketStream::Tcp(stream) => Pin::new(stream).poll_read(context, buffer),
        }
    }
}

impl AsyncWrite for AsyncSocketStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<Result<usize>> {
        match &mut *self {
            AsyncSocketStream::Unix(stream) => Pin::new(stream).poll_write(context, buffer),
            AsyncSocketStream::Tcp(stream) => Pin::new(stream).poll_write(context, buffer),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Result<()>> {
        match &mut *self {
            AsyncSocketStream::Unix(stream) => Pin::new(stream).poll_flush(context),
            AsyncSocketStream::Tcp(stream) => Pin::new(stream).poll_flush(context),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Result<()>> {
        match &mut *self {
            AsyncSocketStream::Unix(stream) => Pin::new(stream).poll_shutdown(context),
            AsyncSocketStream::Tcp(stream) => Pin::new(stream).poll_shutdown(context),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use camino::Utf8PathBuf;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::UnresolvedMultiSocketAddress;

    #[tokio::test]
    async fn async_socket_stream_io_traits_tcp() {
        let listener = UnresolvedMultiSocketAddress::from_str("127.0.0.1:0")
            .unwrap()
            .bind_async()
            .await
            .unwrap();
        let address =
            UnresolvedMultiSocketAddress::try_from(listener.local_address().unwrap()).unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buffer = [0u8; 5];
            stream.read_exact(&mut buffer).await.unwrap();
            assert_eq!(&buffer, b"hello");
            stream.write_all(b"world").await.unwrap();
            stream.shutdown().await.unwrap();
        });

        let mut stream = address.connect_async().await.unwrap();

        stream.write_all(b"hello").await.unwrap();
        stream.flush().await.unwrap();
        let mut buffer = [0u8; 5];
        stream.read_exact(&mut buffer).await.unwrap();
        assert_eq!(&buffer, b"world");
        stream.shutdown().await.unwrap();

        server.await.unwrap();
    }

    #[tokio::test]
    async fn async_socket_stream_io_traits_unix() {
        let listener = tokio::task::spawn_blocking(|| {
            tempfile::Builder::new()
                .prefix("test_async_socket_stream_unix")
                .suffix(".sock")
                .make(|path| {
                    let socket_path = Utf8PathBuf::from_path_buf(path.to_path_buf()).unwrap();
                    let address = UnresolvedMultiSocketAddress::Unix(socket_path);
                    tokio::runtime::Handle::current().block_on(address.bind_async())
                })
        })
        .await
        .unwrap()
        .unwrap();

        let address =
            UnresolvedMultiSocketAddress::try_from(listener.as_file().local_address().unwrap())
                .unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.as_file().accept().await.unwrap();
            let mut buffer = [0u8; 5];
            stream.read_exact(&mut buffer).await.unwrap();
            assert_eq!(&buffer, b"hello");
            stream.write_all(b"world").await.unwrap();
            stream.shutdown().await.unwrap();
        });

        let mut stream = address.connect_async().await.unwrap();

        stream.write_all(b"hello").await.unwrap();
        stream.flush().await.unwrap();
        let mut buffer = [0u8; 5];
        stream.read_exact(&mut buffer).await.unwrap();
        assert_eq!(&buffer, b"world");
        stream.shutdown().await.unwrap();

        server.await.unwrap();
    }
}
