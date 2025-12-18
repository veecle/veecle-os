mod fake_veecle_os_runtime {
    pub trait Storable {
        type DataType: std::fmt::Debug;
    }
}

#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(crate = self::fake_veecle_os_runtime)]
pub struct Sensor0 {
    test: u8,
}

#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(crate = self::fake_veecle_os_runtime, data_type = u8)]
pub struct Sensor1;

// Check that attribute order doesn't matter.
#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(data_type = u8, crate = self::fake_veecle_os_runtime)]
pub struct Sensor2;

fn main() {}
