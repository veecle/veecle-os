use examples_common::actors::udp::UdpServerActor;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use veecle_os::osal::std::log::Log;
use veecle_os::osal::std::net::udp::UdpSocket;

pub const SERVER_ADDRESS: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 1, 2, 6), 8000));

#[veecle_os::osal::std::main(telemetry = true)]
async fn main() {
    veecle_os::runtime::execute! {
        actors: [
            UdpServerActor<UdpSocket, Log>: (
                UdpSocket::new(),
                SERVER_ADDRESS
            ),
        ],
    }
    .await;
}
