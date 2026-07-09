mod input;
mod output;

pub use input::input_at;
pub use output::output_at;

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};

const DEFAULT_LABEL: &str = "Default";

#[derive(Debug, Clone)]
pub struct DeviceLabel {
    pub display: String,
    pub raw_name: String,
    pub host_api: String,
    pub channels: u16,
}

#[derive(Debug, Default)]
pub struct DeviceLists {
    pub input_labels: Vec<String>,
    pub output_labels: Vec<String>,
    pub input_devices: Vec<DeviceLabel>,
    pub output_devices: Vec<DeviceLabel>,
}

pub fn enumerate() -> Result<DeviceLists> {
    let host = cpal::default_host();
    let host_api = host.id().name().to_string();
    let (input_labels, input_devices) = list_labels(host.input_devices()?, &host_api);
    let (output_labels, output_devices) = list_labels(host.output_devices()?, &host_api);
    Ok(DeviceLists {
        input_labels,
        output_labels,
        input_devices,
        output_devices,
    })
}

fn list_labels(
    devices: impl Iterator<Item = cpal::Device>,
    host_api: &str,
) -> (Vec<String>, Vec<DeviceLabel>) {
    let mut labels = vec![DEFAULT_LABEL.into()];
    let mut metadatas = vec![DeviceLabel {
        display: DEFAULT_LABEL.into(),
        raw_name: String::new(),
        host_api: host_api.into(),
        channels: 0,
    }];
    for dev in devices {
        let Ok(name) = dev.description().map(|d| d.name().to_string()) else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        let channels = supported_channel_count(&dev);
        let display = format!("{} ({})", name, host_api);
        if metadatas.iter().any(|m| m.raw_name == name) {
            continue;
        }
        labels.push(display.clone());
        metadatas.push(DeviceLabel {
            display,
            raw_name: name,
            host_api: host_api.into(),
            channels,
        });
    }
    (labels, metadatas)
}

fn supported_channel_count(dev: &cpal::Device) -> u16 {
    if let Ok(cfg) = dev.default_input_config() {
        return cfg.channels();
    }
    if let Ok(cfg) = dev.default_output_config() {
        return cfg.channels();
    }
    if let Ok(configs) = dev.supported_input_configs()
        && let Some(cfg) = configs.into_iter().next()
    {
        return cfg.channels();
    }
    if let Ok(configs) = dev.supported_output_configs()
        && let Some(cfg) = configs.into_iter().next()
    {
        return cfg.channels();
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_is_first_label() {
        let lists = enumerate().unwrap_or_default();
        if !lists.input_labels.is_empty() {
            assert_eq!(lists.input_labels[0], "Default");
        }
        if !lists.output_labels.is_empty() {
            assert_eq!(lists.output_labels[0], "Default");
        }
    }
    #[test]
    fn default_meta_is_sentinel() {
        let lists = enumerate().unwrap_or_default();
        assert_eq!(
            lists.input_devices.first().map(|m| m.raw_name.as_str()),
            Some("")
        );
        assert_eq!(
            lists.output_devices.first().map(|m| m.raw_name.as_str()),
            Some("")
        );
    }
}
