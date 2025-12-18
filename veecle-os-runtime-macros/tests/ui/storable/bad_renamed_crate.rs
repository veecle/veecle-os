#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(crate = "::veecle_os_runtime")]
pub struct Sensor0 {
    test: u8,
}

#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(crate())]
pub struct Sensor1 {
    test: u8,
}
#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(crate = ::veecle_os_runtime, crate = ::veecle_os_runtime2)]
pub struct Sensor2 {
    test: u8,
}

#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(crate = 1)]
pub struct Sensor3 {
    test: u8,
}

#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(crate = foo::1)]
pub struct Sensor4 {
    test: u8,
}

// `foo` is parsed as a valid path, then `<5>` is detected as extra tokens
#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(crate = foo<5>)]
pub struct Sensor5 {
    test: u8,
}

fn main() {}
