#[veecle_os_runtime::actor]
async fn unused<T>() -> core::convert::Infallible {
    core::future::pending().await
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        store: [],
        actors: [Unused<()>],
    };
}
