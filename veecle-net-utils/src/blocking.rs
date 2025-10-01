//! Blocking socket I/O operations.
//!
//! Blocking socket streams and connection methods for both Unix domain sockets and TCP sockets.

use std::io::{Read, Result, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;

use crate::UnresolvedMultiSocketAddress;

/// Blocking socket stream that can be either Unix or TCP.
#[derive(Debug)]
pub enum BlockingSocketStream {
    /// Unix domain socket stream.
    Unix(UnixStream),

    /// TCP socket stream.
    Tcp(TcpStream),
}

impl UnresolvedMultiSocketAddress {
    /// Connects to this address using a blocking socket.
    pub fn connect_blocking(&self) -> Result<BlockingSocketStream> {
        match self {
            UnresolvedMultiSocketAddress::Unix(path) => {
                let stream = UnixStream::connect(path)?;
                Ok(BlockingSocketStream::Unix(stream))
            }
            UnresolvedMultiSocketAddress::Tcp(socket) => {
                let stream = TcpStream::connect(socket)?;
                Ok(BlockingSocketStream::Tcp(stream))
            }
        }
    }
}

impl Read for BlockingSocketStream {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        match self {
            BlockingSocketStream::Unix(stream) => stream.read(buffer),
            BlockingSocketStream::Tcp(stream) => stream.read(buffer),
        }
    }
}

impl Write for BlockingSocketStream {
    fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        match self {
            BlockingSocketStream::Unix(stream) => stream.write(buffer),
            BlockingSocketStream::Tcp(stream) => stream.write(buffer),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            BlockingSocketStream::Unix(stream) => stream.flush(),
            BlockingSocketStream::Tcp(stream) => stream.flush(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::os::unix::net::UnixListener;

    use crate::{MultiSocketAddress, UnresolvedMultiSocketAddress};

    #[test]
    fn blocking_socket_stream_io_traits_tcp() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = UnresolvedMultiSocketAddress::try_from(MultiSocketAddress::Tcp(
            listener.local_addr().unwrap(),
        ))
        .unwrap();

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 5];
            stream.read_exact(&mut buffer).unwrap();
            assert_eq!(&buffer, b"hello");
            stream.write_all(b"world").unwrap();
        });

        let mut stream = address.connect_blocking().unwrap();

        stream.write_all(b"hello").unwrap();
        let mut buffer = [0u8; 5];
        stream.read_exact(&mut buffer).unwrap();
        assert_eq!(&buffer, b"world");

        server.join().unwrap();
    }

    #[test]
    fn blocking_socket_stream_io_traits_unix() {
        let listener = tempfile::Builder::new()
            .prefix("test_blocking_socket_stream_unix")
            .suffix(".sock")
            .make(|path| UnixListener::bind(path))
            .unwrap();

        let address = UnresolvedMultiSocketAddress::try_from(MultiSocketAddress::Unix(
            listener.as_file().local_addr().unwrap(),
        ))
        .unwrap();

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.as_file().accept().unwrap();
            let mut buffer = [0u8; 5];
            stream.read_exact(&mut buffer).unwrap();
            assert_eq!(&buffer, b"hello");
            stream.write_all(b"world").unwrap();
            stream.flush().unwrap();
        });

        let mut stream = address.connect_blocking().unwrap();

        stream.write_all(b"hello").unwrap();
        stream.flush().unwrap();
        let mut buffer = [0u8; 5];
        stream.read_exact(&mut buffer).unwrap();
        assert_eq!(&buffer, b"world");

        server.join().unwrap();
    }
}
