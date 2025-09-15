use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use veecle_os::osal::std::log::Log;
use veecle_os_examples_common::actors::udp::UdpClientActor;

pub const SERVER_ADDRESS: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 1, 2, 6), 8000));
pub const CLIENT_ADDRESS: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 1, 2, 7), 8000));

#[veecle_os::osal::std::main(telemetry = true)]
async fn main() {
    veecle_os::runtime::execute! {
        store: [],
        actors: [
            UdpClientActor<_, Log>: (
                veecle_os::osal::std::net::udp::UdpSocket::new(),
                CLIENT_ADDRESS,
                SERVER_ADDRESS
            ),
        ],
    }
    .await;
}
