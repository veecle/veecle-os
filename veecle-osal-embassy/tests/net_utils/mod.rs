pub mod loopback;

use crate::net_utils::loopback::Loopback;
use embassy_executor::{Executor, Spawner};
use embassy_net::udp::PacketMetadata;
use embassy_net::{Config, Ipv4Cidr, Ipv6Cidr, Stack, StackResources};
use static_cell::StaticCell;
use std::net::IpAddr;

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Loopback>) -> ! {
    runner.run().await
}

/// Advances mock time to allow smoltcp timers to expire.
///
/// Tests use `embassy-time` with the `mock-driver` feature, which provides a mock clock
/// that does not advance automatically.
/// smoltcp relies on time-based delays for various TCP operations, including delayed ACK
/// (acknowledgment packets sent after a 10ms delay to allow piggybacking on data packets).
///
/// Without this task continuously advancing mock time, smoltcp's timers would never expire,
/// causing operations like `flush()` to hang indefinitely waiting for ACKs.
///
/// This task advances mock time by 1ms on each poll and immediately reschedules itself.
/// When mock time is advanced, the `MockDriver` automatically wakes any expired timers,
/// which triggers the network runner to poll smoltcp and process pending operations.
#[embassy_executor::task]
async fn time_advance_task() -> ! {
    use std::task::Poll;
    core::future::poll_fn(|cx| {
        embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(1));
        cx.waker().wake_by_ref();
        Poll::Pending
    })
    .await
}

#[embassy_executor::task]
async fn main_task(
    spawner: Spawner,
    interface_address: IpAddr,
    test_function: fn(Stack<'static>, Spawner),
) {
    spawner.spawn(time_advance_task()).unwrap();
    let device = Loopback::new();

    let config = match interface_address {
        IpAddr::V4(address) => Config::ipv4_static(embassy_net::StaticConfigV4 {
            address: Ipv4Cidr::new(address, 8),
            dns_servers: heapless::Vec::new(),
            gateway: None,
        }),
        IpAddr::V6(address) => Config::ipv6_static(embassy_net::StaticConfigV6 {
            address: Ipv6Cidr::new(address, 8),
            dns_servers: heapless::Vec::new(),
            gateway: None,
        }),
    };

    // We don't require the seed to be random for tests.
    let seed = 4;

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) =
        embassy_net::new(device, config, RESOURCES.init(StackResources::new()), seed);

    // Launch network task
    spawner.spawn(net_task(runner)).unwrap();
    test_function(stack, spawner);
}

/// Runs a test on the Embassy loopback network stack.
///
/// Every test must be in a separate binary to avoid conflicts on static Embassy resources.
pub fn embassy_test(interface_address: &str, test: fn(Stack<'static>, Spawner)) {
    let interface_address = interface_address.parse().unwrap();
    static EXECUTOR: StaticCell<Executor> = StaticCell::new();

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner
            .spawn(main_task(spawner, interface_address, test))
            .unwrap();
    });
}

pub const UDP_MAX_PACKET_SIZE: usize = 65507;

pub fn udp_socket(stack: Stack) -> impl veecle_osal_api::net::udp::UdpSocket {
    let rx_meta_buffer = Box::leak(Box::new([PacketMetadata::EMPTY; 1024]));
    let rx_buffer = Box::leak(Box::new([0u8; UDP_MAX_PACKET_SIZE]));
    let tx_meta_buffer = Box::leak(Box::new([PacketMetadata::EMPTY; 1024]));
    let tx_buffer = Box::leak(Box::new([0u8; UDP_MAX_PACKET_SIZE]));
    let embassy_socket = embassy_net::udp::UdpSocket::new(
        stack,
        rx_meta_buffer,
        rx_buffer,
        tx_meta_buffer,
        tx_buffer,
    );
    veecle_osal_embassy::net::udp::UdpSocket::new(embassy_socket).unwrap()
}

pub fn tcp_socket<'a>(stack: Stack<'a>) -> impl veecle_osal_api::net::tcp::TcpSocket + 'a {
    let rx_buffer = Box::leak(Box::new([0u8; 4096]));
    let tx_buffer = Box::leak(Box::new([0u8; 4096]));
    let embassy_socket = embassy_net::tcp::TcpSocket::new(stack, rx_buffer, tx_buffer);
    veecle_osal_embassy::net::tcp::TcpSocket::new(embassy_socket).unwrap()
}
