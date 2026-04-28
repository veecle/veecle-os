mod fake_veecle_os_runtime {
    pub trait Storable {
        type DataType: std::fmt::Debug;
    }

    pub trait Flatten {
        fn flatten(&self, buffer: &mut impl MetricBuffer);
    }

    pub trait MetricBuffer {
        fn add_metric(&mut self, key: &'static str, value: ());
    }
}

#[derive(Debug, veecle_os_runtime_macros::Storable)]
#[storable(crate = self::fake_veecle_os_runtime)]
pub struct Sensor0 {
    test: u8,
}

fn main() {}
