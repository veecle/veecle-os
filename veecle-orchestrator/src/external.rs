use std::net::SocketAddr;

use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use veecle_ipc_protocol::EncodedStorable;

#[tracing::instrument(skip(input, output))]
pub async fn run(
    address: crate::UnresolvedSocketAddr,
    input: mpsc::Sender<EncodedStorable>,
    mut output: mpsc::Receiver<(SocketAddr, EncodedStorable)>,
) -> eyre::Result<()> {
    let socket = UdpSocket::bind(address.as_to_socket_addrs()).await?;

    // Arbitrary message size limit also used in `veecle-ipc-protocol`.
    let mut buffer = [0; 2048];

    tracing::info!("listening");
    loop {
        tokio::select! {
            received = socket.recv(&mut buffer) => {
                match received {
                    Ok(length) => {
                        match serde_json::from_slice(&buffer[..length]) {
                            Ok(storable) => {
                                input.send(storable).await?;
                            }
                            Err(error) => {
                                tracing::error!(?error, "failed to parse external input");
                            }
                        }
                    }
                    Err(error) => {
                        tracing::error!(?error, "failed to parse external input");
                    }
                }
            }
            outgoing = output.recv() => {
                let Some((address, storable)) = outgoing else { continue };
                match serde_json::to_vec(&storable) {
                    Ok(bytes) => {
                        let length = socket.send_to(&bytes, address).await?;
                        if length != bytes.len() {
                            tracing::error!("failed to send all external output");
                        }
                    }
                    Err(error) => {
                        tracing::error!(?error, "failed to serialize external output");
                    }
                }
            }
        }
    }
}
