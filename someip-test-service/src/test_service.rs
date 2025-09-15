//! Main public API entry point for interacting with the SOME/IP test service implementation.

use std::env::consts::OS;
use std::net::UdpSocket;
use std::process::Child;

use crate::config::{common_api, test_service, vsomeip};
use crate::{endpoint, subprocess};

/// SOME/IP test service wrapper that manages the lifecycle of a test service instance.
#[derive(Debug)]
pub struct TestService {
    endpoint: UdpSocket,
    child_process: Child,
    _vsomeip_config: vsomeip::TempConfig,
    _common_api_config: common_api::TempConfig,
}

impl TestService {
    /// Creates a new test service instance with the specified configuration.
    ///
    /// The service will be automatically terminated when the returned instance is dropped.
    ///
    /// # Panics
    ///
    /// - When the platform is not Linux.
    /// - When the vsomeip configuration file cannot be written to a temporary directory.
    /// - When the Common API configuration file cannot be written to a temporary directory.
    /// - When the test service cannot be spawned in a sub-process.
    /// - When the endpoint for communication with the test service cannot be created.
    pub fn new(config: &test_service::Config) -> Self {
        assert!(
            OS == "linux",
            "only Linux should be used. Please decorate your test with `#[cfg(target_os = \"linux\")]`"
        );

        let _vsomeip_config = vsomeip::TempConfig::from(config);
        let _common_api_config = common_api::TempConfig::from(config);

        let child_process = subprocess::spawn(&_common_api_config, &_vsomeip_config)
            .expect("test service should be spawned in a subprocess");

        let endpoint = endpoint::create(config)
            .expect("endpoint for communication with test service should be created");

        Self {
            endpoint,
            child_process,
            _vsomeip_config,
            _common_api_config,
        }
    }

    /// Sends a request to the test service.
    pub fn send(&self, data: &[u8]) -> anyhow::Result<()> {
        endpoint::send(&self.endpoint, data)?;
        Ok(())
    }

    /// Receives a response from the test service.
    pub fn receive(&self, data: &mut [u8]) -> anyhow::Result<()> {
        endpoint::receive(&self.endpoint, data)?;
        Ok(())
    }

    /// Sends a request and waits for a response in a single operation.
    pub fn send_and_receive(&self, request: &[u8], response: &mut [u8]) -> anyhow::Result<()> {
        self.send(request)?;
        self.receive(response)?;
        Ok(())
    }
}

impl Drop for TestService {
    /// Ensures the test service process is terminated when this instance is dropped.
    fn drop(&mut self) {
        self.child_process
            .kill()
            .expect("failed to kill test service subprocess");
    }
}
