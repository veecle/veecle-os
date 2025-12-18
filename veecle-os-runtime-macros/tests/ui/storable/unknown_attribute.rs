#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(foo = "bar")]
pub struct Sensor {
    test: u8,
}

fn main() {}
