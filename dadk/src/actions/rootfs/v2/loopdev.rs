use core::str;
use std::{path::PathBuf, process::Command, thread::sleep, time::Duration};

use anyhow::{anyhow, Result};
use regex::Regex;

use crate::utils::abs_path;

const LOOP_DEVICE_LOSETUP_A_REGEX: &str = r"^/dev/loop(\d+)";

pub struct LoopDevice {
    img_path: Option<PathBuf>,
    loop_device_path: Option<String>,
    /// 尝试在drop时自动detach
    detach_on_drop: bool,
    /// mapper created
    mapper: bool,
}

impl LoopDevice {
    pub fn attached(&self) -> bool {
        self.loop_device_path.is_some()
    }

    pub fn dev_path(&self) -> Option<&String> {
        self.loop_device_path.as_ref()
    }

    pub fn attach(&mut self) -> Result<()> {
        if self.attached() {
            return Ok(());
        }
        if self.img_path.is_none() {
            return Err(anyhow!("Image path not set"));
        }

        let output = Command::new("losetup")
            .arg("-f")
            .arg("--show")
            .arg("-P")
            .arg(self.img_path.as_ref().unwrap())
            .output()?;

        if output.status.success() {
            let loop_device = String::from_utf8(output.stdout)?.trim().to_string();
            self.loop_device_path = Some(loop_device);
            sleep(Duration::from_millis(100));
            log::trace!(
                "Loop device attached: {}",
                self.loop_device_path.as_ref().unwrap()
            );
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to mount disk image: losetup command exited with status {}",
                output.status
            ))
        }
    }

    /// 尝试连接已经存在的loop device
    pub fn attach_by_exists(&mut self) -> Result<()> {
        if self.attached() {
            return Ok(());
        }
        if self.img_path.is_none() {
            return Err(anyhow!("Image path not set"));
        }
        log::trace!(
            "Try to attach loop device by exists: image path: {}",
            self.img_path.as_ref().unwrap().display()
        );
        // losetup -a 查看是否有已经attach了的，如果有，就附着上去
        let cmd = Command::new("losetup")
            .arg("-a")
            .output()
            .map_err(|e| anyhow!("Failed to run losetup -a: {}", e))?;
        let output = String::from_utf8(cmd.stdout)?;
        let s = __loop_device_path_by_disk_image_path(
            self.img_path.as_ref().unwrap().to_str().unwrap(),
            &output,
        )
        .map_err(|e| anyhow!("Failed to find loop device: {}", e))?;
        self.loop_device_path = Some(s);
        Ok(())
    }

    /// 获取指定分区的路径
    ///
    /// # 参数
    ///
    /// * `nth` - 分区的编号
    ///
    /// # 返回值
    ///
    /// 返回一个 `Result<String>`，包含分区路径的字符串。如果循环设备未附加，则返回错误。
    ///
    /// # 错误
    ///
    /// 如果循环设备未附加，则返回 `anyhow!("Loop device not attached")` 错误。
    pub fn partition_path(&mut self, nth: u8) -> Result<PathBuf> {
        if !self.attached() {
            return Err(anyhow!("Loop device not attached"));
        }
        let dev_path = self.loop_device_path.as_ref().unwrap();
        let direct_path = PathBuf::from(format!("{}p{}", dev_path, nth));

        // 判断路径是否存在
        if !direct_path.exists() {
            mapper::create_mapper(self.loop_device_path.as_ref().unwrap())?;
            self.mapper = true;
            let device_name = direct_path.file_name().unwrap();
            let parent_path = direct_path.parent().unwrap();
            let new_path = parent_path.join("mapper").join(device_name);
            if new_path.exists() {
                return Ok(new_path);
            }
            log::error!(
                "Both {} and {} not exist!",
                direct_path.display(),
                new_path.display()
            );
            return Err(anyhow!("Unable to find partition path {}", nth));
        }
        Ok(direct_path)
    }

    pub fn detach(&mut self) {
        if self.loop_device_path.is_none() {
            return;
        }
        let loop_device = self.loop_device_path.take().unwrap();
        let p = PathBuf::from(&loop_device);
        log::trace!(
            "Detach loop device: {}, exists: {}",
            p.display(),
            p.exists()
        );

        if self.mapper {
            mapper::detach_mapper(&loop_device);
            log::trace!("Detach mapper device: {}", &loop_device);
            self.mapper = false;
        }

        let output = Command::new("losetup").arg("-d").arg(&loop_device).output();

        if !output.is_ok() {
            log::error!(
                "losetup failed to detach loop device [{}]: {}",
                &loop_device,
                output.unwrap_err()
            );
            return;
        }

        let output = output.unwrap();

        if !output.status.success() {
            log::error!(
                "losetup failed to detach loop device [{}]: {}, {}",
                loop_device,
                output.status,
                str::from_utf8(output.stderr.as_slice()).unwrap_or("<Unknown>")
            );
        }
    }

    #[allow(dead_code)]
    pub fn detach_on_drop(&self) -> bool {
        self.detach_on_drop
    }

    #[allow(dead_code)]
    pub fn set_try_detach_when_drop(&mut self, try_detach_when_drop: bool) {
        self.detach_on_drop = try_detach_when_drop;
    }
}

impl Drop for LoopDevice {
    fn drop(&mut self) {
        if self.detach_on_drop {
            self.detach();
        }
    }
}

mod mapper {
    use anyhow::anyhow;
    use anyhow::Result;
    use std::process::Command;

    pub(super) fn create_mapper(dev_path: &str) -> Result<()> {
        let output = Command::new("kpartx")
            .arg("-a")
            .arg("-v")
            .arg(dev_path)
            .output()
            .map_err(|e| anyhow!("Failed to run kpartx: {}", e))?;
        if output.status.success() {
            let output_str = String::from_utf8(output.stdout)?;
            log::trace!("kpartx output: {}", output_str);
            return Ok(());
        }
        Err(anyhow!("Failed to create mapper"))
    }

    pub(super) fn detach_mapper(dev_path: &str) {
        let output = Command::new("kpartx")
            .arg("-d")
            .arg("-v")
            .arg(dev_path)
            .output();
        if output.is_ok() {
            let output = output.unwrap();
            if !output.status.success() {
                log::error!(
                    "kpartx failed to detach mapper device [{}]: {}, {}",
                    dev_path,
                    output.status,
                    String::from_utf8(output.stderr).unwrap_or("<Unknown>".to_string())
                );
            }
        } else {
            log::error!(
                "Failed to detach mapper device [{}]: {}",
                dev_path,
                output.unwrap_err()
            );
        }
    }
}

pub struct LoopDeviceBuilder {
    img_path: Option<PathBuf>,
    loop_device_path: Option<String>,
    detach_on_drop: bool,
}

impl LoopDeviceBuilder {
    pub fn new() -> Self {
        LoopDeviceBuilder {
            img_path: None,
            loop_device_path: None,
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

    pub fn build(self) -> Result<LoopDevice> {
        let loop_dev = LoopDevice {
            img_path: self.img_path,
            loop_device_path: self.loop_device_path,
            detach_on_drop: self.detach_on_drop,
            mapper: false,
        };

        Ok(loop_dev)
    }
}

fn __loop_device_path_by_disk_image_path(
    disk_img_path: &str,
    losetup_a_output: &str,
) -> Result<String> {
    let re = Regex::new(LOOP_DEVICE_LOSETUP_A_REGEX)?;
    for line in losetup_a_output.lines() {
        if !line.contains(disk_img_path) {
            continue;
        }
        let caps = re.captures(line);
        if caps.is_none() {
            continue;
        }
        let caps = caps.unwrap();
        let loop_device = caps.get(1).unwrap().as_str();
        let loop_device = format!("/dev/loop{}", loop_device);
        return Ok(loop_device);
    }
    Err(anyhow!("Loop device not found"))
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
