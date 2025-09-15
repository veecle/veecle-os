#![expect(missing_docs, reason = "test")]

use veecle_osal_std::net::tcp::TcpSocket;

#[tokio::test]
async fn tcp_connect() {
    const IP_ADDRESS: &str = "127.3.0.1";
    let client = TcpSocket::new();
    let server = TcpSocket::new();
    veecle_osal_api::net::tcp::test_suite::test_connect(client, server, IP_ADDRESS).await;
}

#[tokio::test]
async fn tcp_send_recv_v4() {
    const IP_ADDRESS: &str = "127.3.0.3";
    let client = TcpSocket::new();
    let server = TcpSocket::new();
    veecle_osal_api::net::tcp::test_suite::test_send_recv(client, server, IP_ADDRESS).await;
}

#[tokio::test]
async fn tcp_send_recv_v6() {
    const IP_ADDRESS: &str = "::1";
    let client = TcpSocket::new();
    let server = TcpSocket::new();
    veecle_osal_api::net::tcp::test_suite::test_send_recv(client, server, IP_ADDRESS).await;
}

#[tokio::test]
async fn tcp_connect_refused() {
    const IP_ADDRESS: &str = "127.3.0.4";
    let client = TcpSocket::new();
    veecle_osal_api::net::tcp::test_suite::test_connect_refused(client, IP_ADDRESS).await;
}

#[tokio::test]
async fn tcp_accept_with_zero_port() {
    const IP_ADDRESS: &str = "127.3.0.5";
    let server = TcpSocket::new();
    veecle_osal_api::net::tcp::test_suite::test_accept_with_zero_port(server, IP_ADDRESS).await;
}

#[tokio::test]
async fn tcp_close_connection() {
    const IP_ADDRESS: &str = "127.3.0.6";
    let client = TcpSocket::new();
    let server = TcpSocket::new();
    veecle_osal_api::net::tcp::test_suite::test_close_connection(client, server, IP_ADDRESS).await;
}

#[tokio::test]
async fn test_accept_all_zero_ip_v4() {
    const IP_ADDRESS: &str = "127.3.0.7";
    let client = TcpSocket::new();
    let server = TcpSocket::new();
    veecle_osal_api::net::tcp::test_suite::test_accept_all_zero_ip(client, server, IP_ADDRESS)
        .await;
}

#[tokio::test]
async fn test_accept_all_zero_ip_v6() {
    const IP_ADDRESS: &str = "::1";
    let client = TcpSocket::new();
    let server = TcpSocket::new();
    veecle_osal_api::net::tcp::test_suite::test_accept_all_zero_ip(client, server, IP_ADDRESS)
        .await;
}
