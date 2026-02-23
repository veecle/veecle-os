#![no_std]
#![no_main]

use core::format_args;
use core::net::SocketAddr;
use core::net::{Ipv4Addr, SocketAddrV4};
use embassy_executor::Spawner;
use embassy_net::udp::PacketMetadata;
use embassy_net::{EthernetAddress, Ipv4Address, Ipv4Cidr};
use examples_common::actors::udp::UdpServerActor;
use heapless::Vec;
use panic_halt as _;
use veecle_os::osal::api::log::LogTarget;
use veecle_os::osal::embassy::net::udp;

pub const SERVER_ADDRESS: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 56, 1), 8000));

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = examples_embassy_stm32::initialize_board();

    let net_config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 56, 1), 24),
        dns_servers: Vec::new(),
        gateway: None,
    });

    let net_stack = examples_embassy_stm32::initialize_networking(
        spawner,
        peripherals,
        net_config,
        EthernetAddress([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]),
    );

    net_stack.wait_config_up().await;

    veecle_os::osal::embassy::log::Log::println(format_args!("Network task initialized"));

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut rx_meta_buffer = [PacketMetadata::EMPTY; 1024];
    let mut tx_meta_buffer = [PacketMetadata::EMPTY; 1024];

    let embassy_socket = embassy_net::udp::UdpSocket::new(
        net_stack,
        &mut rx_meta_buffer,
        &mut rx_buffer,
        &mut tx_meta_buffer,
        &mut tx_buffer,
    );
    let socket = udp::UdpSocket::new(embassy_socket).unwrap();

    veecle_os::runtime::execute! {
        actors: [
            UdpServerActor<udp::UdpSocket, veecle_os::osal::embassy::log::Log>: (
                socket,
                SERVER_ADDRESS
            ),
        ],
    }
    .await
}
