use serde::Deserialize;

/// Supported hypervisor types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HypervisorType {
    Qemu,
    CloudHypervisor,
}

impl<'de> Deserialize<'de> for HypervisorType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_ascii_lowercase().as_str() {
            "qemu" => Ok(HypervisorType::Qemu),
            "cloud-hypervisor" => Ok(HypervisorType::CloudHypervisor),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown hypervisor type: {}",
                s
            ))),
        }
    }
}
