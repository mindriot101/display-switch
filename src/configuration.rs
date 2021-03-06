//
// Copyright © 2020 Haim Gelfenbeyn
// This code is licensed under MIT license (see LICENSE.txt for details)
//

use anyhow::{anyhow, Result};
use config;
use dirs;
use serde::{Deserialize, Deserializer};

use crate::display_control;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    #[serde(deserialize_with = "Configuration::deserialize_usb_device")]
    pub usb_device: String,
    pub monitor_input: display_control::InputSource,
}

impl Configuration {
    pub fn load() -> Result<Self> {
        let config_file_name = Self::config_file_name()?;
        let mut settings = config::Config::default();
        settings
            .merge(config::File::from(config_file_name.to_path_buf()))?
            .merge(config::Environment::with_prefix("DISPLAY_SWITCH"))?;
        let config = settings.try_into::<Self>()?;
        info!("Configuration loaded ({:?}): {:?}", config_file_name, config);
        Ok(config)
    }

    fn deserialize_usb_device<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(s.to_lowercase())
    }

    pub fn config_file_name() -> Result<std::path::PathBuf> {
        #[cfg(target_os = "macos")]
        let config_dir = dirs::preference_dir().ok_or(anyhow!("Config directory not found"))?;
        #[cfg(target_os = "windows")]
        let config_dir = dirs::config_dir()
            .ok_or(anyhow!("Config directory not found"))?
            .join("display-switch");
        std::fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("display-switch.ini"))
    }

    pub fn log_file_name() -> Result<std::path::PathBuf> {
        #[cfg(target_os = "macos")]
        let log_dir = dirs::home_dir()
            .ok_or(anyhow!("Home directory not found"))?
            .join("Library")
            .join("Logs")
            .join("display-switch");
        #[cfg(target_os = "windows")]
        let log_dir = dirs::data_local_dir()
            .ok_or(anyhow!("Data-local directory not found"))?
            .join("display-switch");
        std::fs::create_dir_all(&log_dir)?;
        Ok(log_dir.join("display-switch.log"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::ConfigError;
    use config::FileFormat::Ini;
    //use crate::display_control::{InputSource, SymbolicInputSource};
    //use crate::display_control::InputSource::Symbolic;

    #[test]
    fn test_log_file_name() {
        let file_name = Configuration::log_file_name();
        assert!(file_name.is_ok());
        assert!(file_name.unwrap().ends_with("display-switch.log"))
    }

    fn load_test_config(config_str: &str) -> Result<Configuration, ConfigError> {
        let mut settings = config::Config::default();
        settings.merge(config::File::from_str(config_str, Ini)).unwrap();
        settings.try_into::<Configuration>()
    }

    #[test]
    fn test_usb_device_deserialization() {
        let config = load_test_config(
            r#"
            usb_device = "dead:BEEF"
            monitor_input = "DisplayPort2"
        "#,
        )
        .unwrap();
        assert_eq!(config.usb_device, "dead:beef")
    }

    #[test]
    fn test_symbolic_input_deserialization() {
        let config = load_test_config(
            r#"
            usb_device = "dead:BEEF"
            monitor_input = "DisplayPort2"
        "#,
        )
        .unwrap();
        assert_eq!(config.monitor_input.value(), 0x10);
    }

    #[test]
    fn test_decimal_input_deserialization() {
        let config = load_test_config(
            r#"
            usb_device = "dead:BEEF"
            monitor_input = 22
        "#,
        )
        .unwrap();
        assert_eq!(config.monitor_input.value(), 22);
    }

    #[test]
    fn test_hexadecimal_input_deserialization() {
        let config = load_test_config(
            r#"
            usb_device = "dead:BEEF"
            monitor_input = "0x10"
        "#,
        )
        .unwrap();
        assert_eq!(config.monitor_input.value(), 16);
    }
}
