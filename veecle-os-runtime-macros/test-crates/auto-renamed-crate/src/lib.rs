/// Test that the `crate = my_veecle_os_runtime` argument isn't required for the simple case of a renamed direct dependency.

#[derive(Debug, my_veecle_os_runtime::Storable)]
pub struct Foo;

#[my_veecle_os_runtime::actor]
pub async fn bar() -> my_veecle_os_runtime::Never {
    unimplemented!("testing compilation")
}
