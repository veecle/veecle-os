#![expect(missing_docs)]

mod inner {
    veecle_os_data_support_can::generate!(
        pub mod j1939 {
            #![dbc = include_str!("../../veecle-os-data-support-can-codegen/tests/cases/CSS-Electronics-SAE-J1939-DEMO.dbc")]
        }
    );
}

use inner::j1939::ccvs1::WheelBasedVehicleSpeed;
use inner::j1939::eec1::EngineSpeed;
use inner::j1939::{Ccvs1, Eec1};

#[test]
fn eec1() {
    let eec1 = Eec1 {
        engine_speed: EngineSpeed::try_from(0.5).unwrap(),
    };
    assert_eq!(
        eec1,
        Eec1::try_from(veecle_os_data_support_can::Frame::from(&eec1)).unwrap()
    );
}

#[test]
fn ccvs1() {
    let ccvs1 = Ccvs1 {
        wheel_based_vehicle_speed: WheelBasedVehicleSpeed::try_from(0.5).unwrap(),
    };
    assert_eq!(
        ccvs1,
        Ccvs1::try_from(veecle_os_data_support_can::Frame::from(&ccvs1)).unwrap()
    );
}
