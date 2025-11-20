#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(data_type = "u8")]
pub struct Sensor0;

#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(data_type = "std::string::String")]
pub struct Sensor1;

#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(data_type = "Vec<u8>")]
pub struct Sensor2;

// `foo +` is parsed as a valid type (trait object type bound), then `2` is detected as extra tokens
#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(data_type = foo + 2)]
pub struct Sensor3;

#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(data_type = 1)]
pub struct Sensor4;

fn main() {}
