//! Test that crate was properly built & linked.

use std::env::set_var;
use std::io::Write;

use ntest_timeout::timeout;
use someip_test_service_sys::{launch, terminate};

#[test]
#[timeout(240000)]
fn smoke_test() {
    // We want to avoid conflicts between parallel CI test runs.
    let random_network_name = format!("test-service-{}", rand::random::<u32>());
    let free_udp_port = {
        let socket =
            std::net::UdpSocket::bind(("127.0.0.1", 0)).expect("failed to get free UDP port");
        let address = socket.local_addr().expect("failed to get socket address");
        address.port()
    };
    let mut vsomeip_config = tempfile::Builder::new()
        .prefix("vsomeip")
        .suffix(".json")
        .tempfile()
        .expect("failed to create temporary vsomeip config file");
    vsomeip_config
        .write_all(
            include_str!("config/vsomeip.json")
                .replace("<network_placeholder>", &random_network_name)
                .replace("<udp_port_placeholder>", &free_udp_port.to_string())
                .as_ref(),
        )
        .expect("failed to write temporary vsomeip config file");

    // SAFETY: We don't expect more then one test in this crate. Thus no concurrent calls to the std::env::set_var.
    // SAFETY: COMMONAPI_CONFIG and VSOMEIP_CONFIGURATION environment variables are set prior to launching test service.
    unsafe {
        set_var(
            "COMMONAPI_CONFIG",
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/common_api.ini"),
        );
        set_var("VSOMEIP_CONFIGURATION", vsomeip_config.path().as_os_str());
        launch();
        launch(); // We check that double launch is not causing crash.
        terminate();
    }
}
