#![expect(missing_docs, reason = "test")]

use veecle_osal_std::net::udp::UdpSocket;

#[tokio::test]
async fn udp_bind_all_zero_address_v6() {
    let socket = UdpSocket::new();
    veecle_osal_api::net::udp::test_suite::test_bind_all_zero_address_v6(socket).await;
}

#[tokio::test]
async fn udp_bind_all_zero_address_v4() {
    let socket = UdpSocket::new();
    veecle_osal_api::net::udp::test_suite::test_bind_all_zero_address_v4(socket).await;
}

#[tokio::test]
async fn udp_bind_specific_port() {
    const IP_ADDRESS: &str = "127.2.4.1";
    let socket = UdpSocket::new();
    veecle_osal_api::net::udp::test_suite::test_bind_specific_port(socket, IP_ADDRESS).await;
}

#[tokio::test]
async fn udp_send_recv_basic() {
    const IP_ADDRESS: &str = "127.2.4.2";
    let socket1 = UdpSocket::new();
    let socket2 = UdpSocket::new();
    veecle_osal_api::net::udp::test_suite::test_send_recv(socket1, socket2, IP_ADDRESS).await;
}

#[tokio::test]
async fn udp_local_addr_before_bind() {
    let socket = UdpSocket::new();
    veecle_osal_api::net::udp::test_suite::test_local_addr_before_bind(socket).await;
}

#[tokio::test]
async fn udp_close_socket() {
    const IP_ADDRESS: &str = "127.2.4.4";
    let socket = UdpSocket::new();
    veecle_osal_api::net::udp::test_suite::test_close_socket(socket, IP_ADDRESS).await;
}

#[tokio::test]
async fn udp_recv_without_bind() {
    let socket = UdpSocket::new();
    veecle_osal_api::net::udp::test_suite::test_recv_without_bind(socket).await;
}

#[tokio::test]
async fn udp_send_without_bind() {
    const IP_ADDRESS: &str = "127.2.4.5";
    let socket = UdpSocket::new();
    veecle_osal_api::net::udp::test_suite::test_send_without_bind(socket, IP_ADDRESS).await;
}

#[tokio::test]
async fn udp_bind_multiple_sockets_same_ip() {
    const IP_ADDRESS: &str = "127.2.4.6";
    let socket1 = UdpSocket::new();
    let socket2 = UdpSocket::new();
    veecle_osal_api::net::udp::test_suite::test_bind_multiple_sockets_same_ip(
        socket1, socket2, IP_ADDRESS,
    )
    .await;
}

#[tokio::test]
async fn udp_zero_length_datagram() {
    const IP_ADDRESS: &str = "127.2.4.7";
    let socket1 = UdpSocket::new();
    let socket2 = UdpSocket::new();
    veecle_osal_api::net::udp::test_suite::test_zero_length_datagram(socket1, socket2, IP_ADDRESS)
        .await;
}

#[tokio::test]
async fn udp_max_datagram_size() {
    const IP_ADDRESS: &str = "127.2.4.8";
    let socket1 = UdpSocket::new();
    let socket2 = UdpSocket::new();
    veecle_osal_api::net::udp::test_suite::test_max_datagram_size(socket1, socket2, IP_ADDRESS)
        .await;
}

#[tokio::test]
async fn udp_multiple_binds() {
    const IP_ADDRESS: &str = "127.2.4.9";
    let socket = UdpSocket::new();
    veecle_osal_api::net::udp::test_suite::test_multiple_binds(socket, IP_ADDRESS).await;
}

#[tokio::test]
async fn udp_rebind_after_close() {
    const IP_ADDRESS: &str = "127.2.4.10";
    let socket = UdpSocket::new();
    veecle_osal_api::net::udp::test_suite::test_rebind_after_close(socket, IP_ADDRESS).await;
}
