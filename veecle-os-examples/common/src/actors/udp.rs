use core::convert::Infallible;
use core::net::SocketAddr;

#[veecle_os::runtime::actor]
pub async fn udp_server_actor<S, L>(#[init_context] input: (S, SocketAddr)) -> Infallible
where
    S: veecle_os::osal::api::net::udp::UdpSocket,
    L: veecle_os::osal::api::log::LogTarget,
{
    const RESPONSE_PREFIX: &[u8; 6] = b"ECHO: ";

    // TODO(DEV-660): We cannot destructure in the function signature because of the `actor` macro.
    let (mut socket, address) = input;
    L::println(format_args!("UDP server actor started"));

    if let Err(error) = socket.bind(address).await {
        L::println(format_args!("Error binding socket: {:?}", error));
        panic!("Failed to bind UDP socket");
    }

    L::println(format_args!("UDP server bound to {:?}", address));

    let mut response = [0u8; 1024 + RESPONSE_PREFIX.len()];
    response[..RESPONSE_PREFIX.len()].copy_from_slice(RESPONSE_PREFIX);
    loop {
        L::println(format_args!("Waiting for datagrams"));

        let (read, client_address) = match socket
            .recv_from(&mut response[RESPONSE_PREFIX.len()..])
            .await
        {
            Ok(result) => {
                L::println(format_args!(
                    "Received datagram from {:?}: {:?}",
                    result.1,
                    core::str::from_utf8(&response[RESPONSE_PREFIX.len()..][..result.0])
                        .unwrap_or("<invalid utf8>")
                ));
                result
            }
            Err(error) => {
                L::println(format_args!("Error receiving datagram: {:?}", error));
                continue;
            }
        };

        L::println(format_args!(
            "Sending response to client {:?}",
            client_address
        ));
        match socket
            .send_to(&response[..read + RESPONSE_PREFIX.len()], client_address)
            .await
        {
            Ok(sent) => {
                L::println(format_args!("Sent {} bytes to client", sent));
            }
            Err(error) => {
                L::println(format_args!("Error sending response: {:?}", error));
                continue;
            }
        }
    }
}

#[veecle_os::runtime::actor]
pub async fn udp_client_actor<S, L>(
    #[init_context] input: (S, SocketAddr, SocketAddr),
) -> Infallible
where
    S: veecle_os::osal::api::net::udp::UdpSocket,
    L: veecle_os::osal::api::log::LogTarget,
{
    // TODO(DEV-660): We cannot destructure in the function signature because of the `actor` macro.
    // First address is local bind address, second is server address.
    let (mut socket, local_address, server_address) = input;
    L::println(format_args!("UDP client actor started"));

    if let Err(error) = socket.bind(local_address).await {
        L::println(format_args!("Error binding socket: {:?}", error));
        panic!("Failed to bind UDP socket");
    }

    L::println(format_args!("UDP client bound to {:?}", local_address));

    loop {
        let message = b"Hi from the UDP client!";
        let mut response = [0u8; 1024];

        L::println(format_args!("Sending message to: {:?}", server_address));
        match socket.send_to(message, server_address).await {
            Ok(sent) => {
                L::println(format_args!("Sent {} bytes to server", sent));
            }
            Err(error) => {
                L::println(format_args!("Error sending to server: {:?}", error));
                continue;
            }
        }

        match socket.recv_from(&mut response).await {
            Ok((read, sender_address)) => {
                L::println(format_args!(
                    "Received response from {:?}: {:?}",
                    sender_address,
                    core::str::from_utf8(&response[..read]).unwrap_or("<invalid utf8>")
                ));

                if sender_address != server_address {
                    L::println(format_args!(
                        "Warning: Response from unexpected address {:?} (expected {:?})",
                        sender_address, server_address
                    ));
                }
            }
            Err(error) => {
                L::println(format_args!("Error receiving response: {:?}", error));
            }
        }
    }
}
