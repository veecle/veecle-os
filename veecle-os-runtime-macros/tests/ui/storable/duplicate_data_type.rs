#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(data_type = "u8", data_type = "u16")]
pub struct Sensor {
    test: u8,
}

fn main() {}
