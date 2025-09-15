#![deny(unused_must_use)]

use veecle_osal_api::net::tcp::TcpSocket;

pub async fn test(mut input: impl TcpSocket) {
    let connection1 = input
        .connect("127.0.0.1:8080".parse().unwrap())
        .await
        .unwrap();
    let _connection2 = input
        .connect("127.0.0.1:8080".parse().unwrap())
        .await
        .unwrap();
    dbg!(&connection1);
}

fn main() {}
