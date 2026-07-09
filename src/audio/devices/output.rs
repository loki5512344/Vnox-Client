use anyhow::{Result, anyhow};
use cpal::Device;
use cpal::traits::{DeviceTrait, HostTrait};

pub fn output_at(index: usize) -> Result<Device> {
    let host = cpal::default_host();
    if index == 0 {
        return host
            .default_output_device()
            .ok_or_else(|| anyhow!("no default output device"));
    }

    let devices: Vec<Device> = host.output_devices()?.collect();

    let mut names: Vec<String> = Vec::new();
    for dev in &devices {
        if let Ok(name) = dev.description().map(|d| d.name().to_string())
            && !name.is_empty()
            && !names.contains(&name)
        {
            names.push(name);
        }
    }

    let name = names
        .get(index.saturating_sub(1))
        .ok_or_else(|| anyhow!("output device index {index} out of range"))?;

    for dev in devices {
        if dev
            .description()
            .map(|d| d.name().to_string())
            .ok()
            .as_deref()
            == Some(name.as_str())
        {
            return Ok(dev);
        }
    }

    Err(anyhow!("output device not found: {name}"))
}
