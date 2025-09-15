#[derive(Debug)]
#[storable]
pub struct Sensor<T>
where
    T: Default + std::fmt::Debug,
{
    test: u8,
    test0: u8,
    test1: u8,
    test2: T,
}

fn main() {}
