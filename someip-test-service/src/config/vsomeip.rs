//! Types to work with the vsomeip configuration.

// Private module re-exported for use in crate's private binary.
#![allow(missing_docs)]

use std::path::Path;

use serde::{Deserialize, Serialize};
use tempfile::{Builder, NamedTempFile};

use crate::config::test_service::Config as TestServiceConfig;

const TEMPLATE: &str = r#"
{
    "unicast": "10.0.2.15",
    "network": "vsomeip",
    "logging": {
        "level": "trace",
        "version": {
            "enable": "false"
        }
    },
    "services": [
        { "service": "1234", "instance": "5678", "unreliable": "30509" }
    ],
    "service-discovery": {
        "enable": "false",
        "multicast": "224.244.224.245",
        "port": "30490",
        "protocol": "udp"
    }
}
"#;

/// Minimal required subset of vsomeip configuration.
///
/// Reference: <https://github.com/COVESA/vsomeip/blob/master/documentation/vsomeipConfiguration.md>
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub unicast: String,
    pub network: String,
    pub logging: Logging,
    pub services: Vec<Service>,
    #[serde(rename = "service-discovery")]
    pub service_discovery: ServiceDiscovery,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Logging {
    pub level: String,
    pub version: Version,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub enable: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    pub service: String,
    pub instance: String,
    pub unreliable: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceDiscovery {
    pub enable: String,
    pub multicast: String,
    pub port: String,
    pub protocol: String,
}

impl From<&TestServiceConfig> for Config {
    fn from(config: &TestServiceConfig) -> Self {
        let mut vsomeip: Self =
            serde_json::from_str(TEMPLATE).expect("vsomeip config should be deserialized");

        vsomeip.unicast = config.unicast_address.clone();

        let service = vsomeip
            .services
            .first_mut()
            .expect("vsomeip config template should have at least one service");
        service.unreliable = config.unicast_port.to_string();

        vsomeip.network = format!("test-service-{}", rand::random::<u16>());

        vsomeip.service_discovery.multicast = config.multicast_address.clone();
        vsomeip.service_discovery.port = config.multicast_port.to_string();

        vsomeip.logging.level = config.logging_level.to_vsomeip().into();

        vsomeip
    }
}

/// vsomeip configuration in a temporary directory.
///
/// Deleted automatically when dropped.
#[derive(Debug)]
pub struct TempConfig {
    temp_file: NamedTempFile,
}

impl TempConfig {
    pub fn path(&self) -> &Path {
        self.temp_file.path()
    }
}

impl From<&TestServiceConfig> for TempConfig {
    fn from(test_service_config: &TestServiceConfig) -> Self {
        let vsomeip_config = Config::from(test_service_config);

        let temp_file = Builder::new()
            .prefix("vsomeip")
            .suffix(".json")
            .tempfile()
            .expect("temp vsomeip config file should be created");

        serde_json::to_writer(&temp_file, &vsomeip_config)
            .expect("temp vsomeip config file should be written");

        Self { temp_file }
    }
}
