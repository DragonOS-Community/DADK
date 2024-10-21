use serde::{Deserialize, Deserializer, Serialize};

/// 目标处理器架构
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetArch {
    X86_64,
    RiscV64,
}

impl TargetArch {
    /// 期望的目标处理器架构（如果修改了枚举，那一定要修改这里）
    pub const EXPECTED: [&'static str; 2] = ["x86_64", "riscv64"];
}

impl Default for TargetArch {
    fn default() -> Self {
        TargetArch::X86_64
    }
}

impl TryFrom<&str> for TargetArch {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_ascii_lowercase().as_str() {
            "x86_64" => Ok(TargetArch::X86_64),
            "riscv64" => Ok(TargetArch::RiscV64),
            _ => Err(format!("Unknown target arch: {}", value)),
        }
    }
}

impl Into<&str> for TargetArch {
    fn into(self) -> &'static str {
        match self {
            TargetArch::X86_64 => "x86_64",
            TargetArch::RiscV64 => "riscv64",
        }
    }
}

impl Into<String> for TargetArch {
    fn into(self) -> String {
        let x: &str = self.into();
        x.to_string()
    }
}

impl<'de> Deserialize<'de> for TargetArch {
    fn deserialize<D>(deserializer: D) -> Result<TargetArch, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let r = TargetArch::try_from(s.as_str());
        match r {
            Ok(v) => Ok(v),
            Err(_) => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(s.as_str()),
                &format!("Expected one of {:?}", TargetArch::EXPECTED).as_str(),
            )),
        }
    }
}

impl Serialize for TargetArch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let string: String = Into::into(*self);
        serializer.serialize_str(string.as_str())
    }
}
