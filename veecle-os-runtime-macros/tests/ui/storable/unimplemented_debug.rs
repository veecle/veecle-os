use veecle_os_runtime::Storable;

#[derive(Default, Storable)]
pub struct Sensor;

pub struct Data;

#[derive(Default, Storable)]
#[storable(data_type = "Data")]
pub struct Sensor1;

fn main() {}
