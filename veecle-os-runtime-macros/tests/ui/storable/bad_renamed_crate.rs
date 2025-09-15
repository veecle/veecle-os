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

fn main() {}
