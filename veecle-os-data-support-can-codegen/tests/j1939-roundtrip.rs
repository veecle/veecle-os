#![expect(missing_docs)]

// Since the code is generated we don't want to format it.
// TODO: maybe this should be inserted in the generated code.
#[rustfmt::skip]
// We use `#[cfg(all())]` to test the cfg handling of the code generator, Clippy doesn't like this.
#[expect(clippy::non_minimal_cfg)]
#[path = "cases/CSS-Electronics-SAE-J1939-DEMO.rs"]
mod j1939;

use j1939::ccvs1::WheelBasedVehicleSpeed;
use j1939::eec1::EngineSpeed;
use j1939::{Ccvs1, Eec1};

#[test]
fn eec1() {
    let eec1 = Eec1 {
        engine_speed: EngineSpeed::try_from(0.5).unwrap(),
    };
    assert_eq!(
        eec1,
        Eec1::try_from(my_veecle_os_data_support_can::Frame::from(&eec1)).unwrap()
    );
}

#[test]
fn ccvs1() {
    let ccvs1 = Ccvs1 {
        wheel_based_vehicle_speed: WheelBasedVehicleSpeed::try_from(0.5).unwrap(),
    };
    assert_eq!(
        ccvs1,
        Ccvs1::try_from(my_veecle_os_data_support_can::Frame::from(&ccvs1)).unwrap()
    );
}
