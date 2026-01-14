#[veecle_os_runtime::actor]
async fn unused<T>() -> veecle_os_runtime::Never {
    core::future::pending().await
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        store: [],
        actors: [Unused<()>],
    };
}
