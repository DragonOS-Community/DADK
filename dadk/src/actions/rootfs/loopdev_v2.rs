use core::str;
use std::{path::PathBuf, process::Command, thread::sleep, time::Duration};

use regex::Regex;

use crate::utils::abs_path;

const LOOP_DEVICE_LOSETUP_A_REGEX: &str = r"^/dev/loop(\d+)";


pub struct LoopDevice {
    path: PathBuf,
    detach_on_drop: bool,
    mapper: Mapper,
}

impl LoopDevice {
    fn new(img_path: PathBuf, detach_on_drop: bool) -> Result<Self, LoopError> {
        if !img_path.exists() {
            return Err(LoopError::ImageNotFound);
        }
        let str_img_path = img_path.to_str().ok_or(LoopError::InvalidUtf8)?;

        let may_attach = attach_exists_loop_by_image(str_img_path);

        let loop_device_path = match may_attach {
            Ok(loop_device_path) => {
                log::trace!("Loop device already attached: {}", loop_device_path);
                loop_device_path
            }
            Err(LoopError::LoopDeviceNotFound) => {
                log::trace!("No loop device found, try to attach");
                attach_loop_by_image(str_img_path)?
            }
            Err(err) => {
                log::error!("Failed to attach loop device: {}", err);
                return Err(err);
            }
        };

        let path = PathBuf::from(loop_device_path);
        sleep(Duration::from_millis(100));
        if !path.exists() {
            return Err(LoopError::LoopDeviceNotFound);
        }

        let mapper = Mapper::new(path.clone(), detach_on_drop)?;

        Ok(Self {
            path,
            detach_on_drop,
            mapper,
        })
    }

    pub fn dev_path(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    pub fn partition_path(&self, nth: u8) -> Result<PathBuf, LoopError> {
        self.mapper.partition_path(nth)
    }

    // #[allow(dead_code)]
    // pub fn detach_on_drop(&self) -> bool {
    //     self.detach_on_drop
    // }

    // #[allow(dead_code)]
    // pub fn set_detach_on_drop(&mut self, detach_on_drop: bool) {
    //     self.detach_on_drop = detach_on_drop;
    // }
}
impl Drop for LoopDevice {
    fn drop(&mut self) {
        if !self.detach_on_drop {
            return;
        }
        log::trace!(
            "Detach loop device: {}, exists: {}",
            &self.path.display(),
            self.path.exists()
        );
        if self.path.exists() {
            let path = self.path.to_string_lossy();
            if let Err(err) = LosetupCmd::new().arg("-d").arg(&path).output() {
                log::error!("Failed to detach loop device: {}", err);
            }
        }
    }
}

pub struct LoopDeviceBuilder {
    img_path: Option<PathBuf>,
    // loop_device_path: Option<String>,
    detach_on_drop: bool,
}

impl LoopDeviceBuilder {
    pub fn new() -> Self {
        LoopDeviceBuilder {
            img_path: None,
            // loop_device_path: None,
            detach_on_drop: true,
        }
    }

    pub fn img_path(mut self, img_path: PathBuf) -> Self {
        self.img_path = Some(abs_path(&img_path));
        self
    }

    #[allow(dead_code)]
    pub fn detach_on_drop(mut self, detach_on_drop: bool) -> Self {
        self.detach_on_drop = detach_on_drop;
        self
    }

    pub fn build(self) -> Result<LoopDevice, LoopError> {
        if self.img_path.is_none() {
            return Err(LoopError::ImageNotFound);
        }

        let img_path = self.img_path.unwrap();

        log::trace!(
            "Try to attach loop device by exists: image path: {}",
            img_path.display()
        );

        let loop_device = LoopDevice::new(img_path, self.detach_on_drop)?;
        Ok(loop_device)
    }
}


fn __loop_device_path_by_disk_image_path(
    disk_img_path: &str,
    losetup_a_output: &str,
) -> Result<String, LoopError> {
    let re = Regex::new(LOOP_DEVICE_LOSETUP_A_REGEX).unwrap();
    for line in losetup_a_output.lines() {
        if !line.contains(disk_img_path) {
            continue;
        }
        let caps = re.captures(line);
        if caps.is_none() {
            continue;
        }
        let caps = caps.unwrap();
        let loop_device = caps.get(1).unwrap().as_str().trim();
        let loop_device = format!("/dev/loop{}", loop_device);
        return Ok(loop_device);
    }
    Err(LoopError::LoopDeviceNotFound)
} 
#[derive(Debug)]
pub enum LoopError {
    InvalidUtf8,
    ImageNotFound,
    LoopDeviceNotFound,
    NoMapperAvailable,
    NoPartitionAvailable,
    Losetup(String),
    Kpartx(String),
    #[allow(dead_code)]
    Other(anyhow::Error),
}

impl From<std::string::FromUtf8Error> for LoopError {
    fn from(_: std::string::FromUtf8Error) -> Self {
        LoopError::InvalidUtf8
    }
}

impl std::fmt::Display for LoopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoopError::InvalidUtf8 => write!(f, "Invalid UTF-8"),
            LoopError::ImageNotFound => write!(f, "Image not found"),
            LoopError::LoopDeviceNotFound => write!(f, "Loop device not found"),
            LoopError::NoMapperAvailable => write!(f, "No mapper available"),
            LoopError::NoPartitionAvailable => write!(f, "No partition available"),
            LoopError::Losetup(err) => write!(f, "Losetup error: {}", err),
            LoopError::Kpartx(err) => write!(f, "Kpartx error: {}", err),
            LoopError::Other(err) => write!(f, "Other error: {}", err),
        }
    }
}

impl std::error::Error for LoopError {}




fn attach_loop_by_image(img_path: &str) -> Result<String, LoopError> {
    LosetupCmd::new()
        .arg("-f")
        .arg("--show")
        .arg("-P")
        .arg(img_path)
        .output()
        .map(|output_path| output_path.trim().to_string())
}

fn attach_exists_loop_by_image(img_path: &str) -> Result<String, LoopError> {
    // losetup -a 查看是否有已经attach了的，如果有，就附着上去
    let output = LosetupCmd::new().arg("-a").output()?;

    __loop_device_path_by_disk_image_path(img_path, &output)
}







struct LosetupCmd {
    inner: Command,
}

impl LosetupCmd {
    fn new() -> Self {
        LosetupCmd {
            inner: Command::new("losetup"),
        }
    }

    fn arg(&mut self, arg: &str) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    fn output(&mut self) -> Result<String, LoopError> {
        let output = self
            .inner
            .output()
            .map_err(|e| LoopError::Losetup(e.to_string()))?;
        if output.status.success() {
            let stdout = String::from_utf8(output.stdout)?;
            Ok(stdout)
        } else {
            Err(LoopError::Losetup(format!(
                "losetup failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }
}

struct Mapper {
    dev_path: PathBuf,
    detach_on_drop: bool,
    use_kpartx: bool,
    partitions: Vec<String>,
}

impl Mapper {
    fn new(path: PathBuf, detach_on_drop: bool) -> Result<Self, LoopError> {
        // check if raw part mapper is available, if {device_dir}/loopXpX
        let mut parts = Vec::new();
        let partition_name_prefix = format!("{}p", path.file_name().unwrap().to_str().unwrap());
        let device_dir = path.parent().unwrap();

        for entry in device_dir.read_dir().unwrap() {
            if let Ok(entry) = entry {
                let entry = entry.file_name().into_string().unwrap();
                if entry.starts_with(&partition_name_prefix) {
                    parts.push(entry);
                }
            }
        }

        if !parts.is_empty() {
            log::trace!("Found raw part mapper: {:?}", parts);
            return Ok(Self {
                dev_path: path.to_path_buf(),
                detach_on_drop,
                partitions: parts,
                use_kpartx: false,
            });
        }

        // check if mapper is created, found if {device_dir}/mapper/loopX
        let mapper_path = device_dir.join("mapper");

        for entry in mapper_path.read_dir().unwrap() {
            if let Ok(entry) = entry {
                let entry = entry.file_name().into_string().unwrap();
                if entry.starts_with(&partition_name_prefix) {
                    parts.push(entry);
                }
            }
        }

        if !parts.is_empty() {
            log::trace!("Found kpartx mapper exist: {:?}", parts);
            return Ok(Self {
                dev_path: path,
                detach_on_drop,
                partitions: parts,
                use_kpartx: true,
            });
        }

        KpartxCmd::new()
            .arg("-a")
            .arg(path.to_str().unwrap())
            .output()?;
        for entry in mapper_path.read_dir().unwrap() {
            if let Ok(entry) = entry {
                let entry = entry.file_name().into_string().unwrap();
                if entry.starts_with(&partition_name_prefix) {
                    parts.push(entry);
                }
            }
        }

        if !parts.is_empty() {
            log::trace!("New kpartx with parts: {:?}", parts);
            return Ok(Self {
                dev_path: path,
                detach_on_drop,
                partitions: parts,
                use_kpartx: true,
            });
        }

        Err(LoopError::NoMapperAvailable)
    }

    fn partition_path(&self, nth: u8) -> Result<PathBuf, LoopError> {
        if self.partitions.is_empty() {
            // unlikely, already checked in new()
            log::warn!("No partition available, but the mapper device exists!");
            return Err(LoopError::NoPartitionAvailable);
        }
        let map_root = if !self.use_kpartx {
            self.dev_path
                .parent()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        } else {
            // kpartx mapper device
            self.dev_path
                .with_file_name("mapper")
                .to_string_lossy()
                .into_owned()
        };
        let partition = PathBuf::from(format!(
            "{}/{}",
            map_root,
            self.partitions
                .get((nth - 1) as usize)
                .ok_or(LoopError::NoPartitionAvailable)?,
        ));
        if !partition.exists() {
            log::warn!("Partition exists, but the specified partition does not exist!");
            log::warn!("Available partitions: {:?}", self.partitions);
            log::warn!("Try to find partition: {}", partition.display());
            return Err(LoopError::NoPartitionAvailable);
        }
        Ok(partition)
    }
}

impl Drop for Mapper {
    fn drop(&mut self) {
        if !self.detach_on_drop {
            return;
        }
        if self.dev_path.exists() {
            let path = self.dev_path.to_string_lossy();
            if self.use_kpartx {
                if let Err(err) = KpartxCmd::new().arg("-d").arg(&path).output() {
                    log::error!("Failed to detach mapper device: {}", err);
                }
            }
        }
    }
}

struct KpartxCmd {
    inner: Command,
}

impl KpartxCmd {
    fn new() -> Self {
        KpartxCmd {
            inner: Command::new("kpartx"),
        }
    }

    fn arg(&mut self, arg: &str) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    fn output(&mut self) -> Result<String, LoopError> {
        let output = self
            .inner
            .output()
            .map_err(|e| LoopError::Kpartx(e.to_string()))?;
        if output.status.success() {
            let stdout = String::from_utf8(output.stdout)?;
            Ok(stdout)
        } else {
            Err(LoopError::Kpartx(format!(
                "kpartx failed execute: {:?}, Result: {}",
                self.inner.get_args(),
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_find_loop_device() {
        const DEVICE_NAME_SHOULD_MATCH: [&str; 3] =
            ["/dev/loop11", "/dev/loop11p1", "/dev/loop11p1 "];
        let device_name = "/dev/loop11";
        let re = Regex::new(LOOP_DEVICE_LOSETUP_A_REGEX).unwrap();
        for name in DEVICE_NAME_SHOULD_MATCH {
            assert!(re.find(name).is_some(), "{} should match", name);
            assert_eq!(
                re.find(name).unwrap().as_str(),
                device_name,
                "{} should match {}",
                name,
                device_name
            );
        }
    }

    #[test]
    fn test_parse_losetup_a_output() {
        let losetup_a_output = r#"/dev/loop1: []: (/data/bin/x86_64/disk.img)
/dev/loop29: []: (/var/lib/abc.img)
/dev/loop13: []: (/var/lib/snapd/snaps/gtk-common-themes_1535.snap
/dev/loop19: []: (/var/lib/snapd/snaps/gnome-42-2204_172.snap)"#;
        let disk_img_path = "/data/bin/x86_64/disk.img";
        let loop_device_path =
            __loop_device_path_by_disk_image_path(disk_img_path, losetup_a_output).unwrap();
        assert_eq!(loop_device_path, "/dev/loop1");
    }

    #[test]
    fn test_parse_lsblk_output_not_match() {
        let losetup_a_output = r#"/dev/loop1: []: (/data/bin/x86_64/disk.img)
/dev/loop29: []: (/var/lib/abc.img)
/dev/loop13: []: (/var/lib/snapd/snaps/gtk-common-themes_1535.snap
/dev/loop19: []: (/var/lib/snapd/snaps/gnome-42-2204_172.snap)"#;
        let disk_img_path = "/data/bin/riscv64/disk.img";
        let loop_device_path =
            __loop_device_path_by_disk_image_path(disk_img_path, losetup_a_output);
        assert!(
            loop_device_path.is_err(),
            "should not match any loop device"
        );
    }
}
