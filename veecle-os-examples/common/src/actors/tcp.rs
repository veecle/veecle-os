use core::convert::Infallible;
use core::net::SocketAddr;
use embedded_io_async::{Read, Write};
use veecle_os::osal::api::net::tcp::TcpConnection;

#[veecle_os::runtime::actor]
pub async fn tcp_server_actor<S, L>(#[init_context] input: (S, SocketAddr)) -> Infallible
where
    S: veecle_os::osal::api::net::tcp::TcpSocket,
    L: veecle_os::osal::api::log::LogTarget,
{
    const RESPONSE_PREFIX: &[u8; 6] = b"ECHO: ";

    // TODO(DEV-660): We cannot destructure in the function signature because of the `actor` macro.
    let (mut socket, address) = input;
    L::println(format_args!("TCP server actor started"));

    let mut response = [0u8; 1024 + RESPONSE_PREFIX.len()];
    response[..RESPONSE_PREFIX.len()].copy_from_slice(RESPONSE_PREFIX);
    loop {
        L::println(format_args!("Waiting for connections"));

        let mut connection = match socket.accept(address).await {
            Ok((connection, remote_address)) => {
                L::println(format_args!(
                    "Accepted connection from: {:?}",
                    remote_address
                ));
                connection
            }
            Err(error) => {
                L::println(format_args!("Error accepting connection: {:?}", error));
                continue;
            }
        };

        let read = match connection
            .read(&mut response[RESPONSE_PREFIX.len()..])
            .await
        {
            Ok(0) => {
                L::println(format_args!("Read EOF, closing connection"));
                connection.close().await;
                continue;
            }
            Ok(read) => {
                L::println(format_args!(
                    "Read request from {:?}: {:?}",
                    address,
                    str::from_utf8(&response[RESPONSE_PREFIX.len()..][..read])
                        .unwrap_or("<invalid utf8>"),
                ));
                read
            }
            Err(error) => {
                L::println(format_args!("Error reading from connection: {:?}", error));
                connection.close().await;
                continue;
            }
        };

        L::println(format_args!("Sending message to client"));
        match connection
            .write_all(&response[..read + RESPONSE_PREFIX.len()])
            .await
        {
            Ok(()) => {
                L::println(format_args!("Sent message to client"));
            }
            Err(error) => {
                L::println(format_args!("Error sending over connection: {:?}", error));
            }
        }

        connection.close().await;
    }
}

#[veecle_os::runtime::actor]
pub async fn tcp_client_actor<S, L>(#[init_context] input: (S, SocketAddr)) -> Infallible
where
    S: veecle_os::osal::api::net::tcp::TcpSocket,
    L: veecle_os::osal::api::log::LogTarget,
{
    // TODO(DEV-660): We cannot destructure in the function signature because of the `actor` macro.
    let (mut socket, address) = input;
    L::println(format_args!("TCP client actor started"));
    loop {
        let buffer = b"Hi from the client!";
        let mut response = [0u8; 1024];

        L::println(format_args!("Connecting to server"));
        let mut connection = match socket.connect(address).await {
            Ok(connection) => {
                L::println(format_args!("Successfully connected to server"));
                connection
            }
            Err(error) => {
                L::println(format_args!("Error connecting to server: {:?}", error));
                continue;
            }
        };

        L::println(format_args!("Sending message to server"));
        if let Err(error) = connection.write_all(buffer).await {
            L::println(format_args!("Error sending over connection: {:?}", error));
            continue;
        };
        L::println(format_args!("Sent message to server"));

        // We assume that the message fits into `response`.
        match read_to_end::<L>(&mut connection, &mut response).await {
            Ok(read) => {
                if read == 0 {
                    L::println(format_args!("Read EOF, closing connection"));
                    continue;
                }
                L::println(format_args!(
                    "Received response from {:?}: {:?}",
                    address,
                    str::from_utf8(&response[..read]).unwrap_or("<invalid utf8>"),
                ));
            }
            Err(error) => {
                L::println(format_args!("Error reading from connection: {:?}", error));
            }
        }

        connection.close().await;
    }
}

/// Reads from `connection` until EOF is reached or `buffer` is full.
async fn read_to_end<L: veecle_os::osal::api::log::LogTarget>(
    connection: &mut impl TcpConnection,
    buffer: &mut [u8],
) -> Result<usize, veecle_os::osal::api::net::tcp::Error> {
    let mut bytes_read = 0;

    loop {
        match connection.read(&mut buffer[bytes_read..]).await {
            Ok(read) => {
                if read == 0 {
                    L::println(format_args!("0"));
                    break;
                }
                L::println(format_args!("read: {}", read));
                bytes_read += read;
            }
            Err(error) => {
                L::println(format_args!("error: {:?}", error));
                return Err(error);
            }
        }
    }
    Ok(bytes_read)
}
