//! Types to work with the CommonAPI configuration.

#![allow(
    missing_docs,
    reason = "private module re-exported for use in crate's private binary"
)]

use std::path::Path;

use serde::{Deserialize, Serialize};
use tempfile::{Builder, NamedTempFile};

use crate::config::test_service::Config as TestServiceConfig;

const TEMPLATE: &str = r"
[default]
binding=someip

[logging]
console = true
level = error
";

/// Minimal required subset of CommonAPI configuration.
///
/// Reference: <https://github.com/COVESA/capicxx-core-tools/blob/master/docx/CommonAPICppUserGuide>.
#[derive(Deserialize, Serialize, Clone, PartialEq, Default, Debug)]
pub struct Config {
    pub default: Default,
    pub logging: Logging,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Default, Debug)]
pub struct Default {
    pub binding: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Default, Debug)]
pub struct Logging {
    pub console: String,
    pub level: String,
}

impl From<&TestServiceConfig> for Config {
    fn from(config: &TestServiceConfig) -> Self {
        let mut common_api: Self =
            serde_ini::from_str(TEMPLATE).expect("common api config should be deserialized");

        common_api.logging.level = config.logging_level.to_common_api().to_string();

        common_api
    }
}

/// Common API configuration in a temporary directory.
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
        let common_api_config = Config::from(test_service_config);

        let temp_file = Builder::new()
            .prefix("common_api")
            .suffix(".ini")
            .tempfile()
            .expect("temp common api config file should be created");

        serde_ini::to_writer(&temp_file, &common_api_config)
            .expect("temp common api config file should be written");

        Self { temp_file }
    }
}
