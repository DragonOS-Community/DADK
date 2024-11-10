use std::{path::PathBuf, process::Command};

use anyhow::{anyhow, Result};

pub struct LoopDevice {
    img_path: PathBuf,
    loop_device_path: Option<String>,
}
impl LoopDevice {
    pub fn attach(&mut self) -> Result<()> {
        if self.loop_device_path.is_some() {
            return Ok(());
        }
        let output = Command::new("losetup")
            .arg("-f")
            .arg("--show")
            .arg("-P")
            .arg(&self.img_path)
            .output()?;

        if output.status.success() {
            let loop_device = String::from_utf8(output.stdout)?.trim().to_string();
            self.loop_device_path = Some(loop_device);
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
    pub fn partition_path(&self, nth: u8) -> Result<PathBuf> {
        if self.loop_device_path.is_none() {
            return Err(anyhow!("Loop device not attached"));
        }
        let s = format!("{}p{}", self.loop_device_path.as_ref().unwrap(), nth);
        let s = PathBuf::from(s);
        // 判断路径是否存在
        if !s.exists() {
            return Err(anyhow!("Partition not exist"));
        }
        Ok(s)
    }

    pub fn detach(&mut self) -> Result<()> {
        if self.loop_device_path.is_none() {
            return Ok(());
        }
        let loop_device = self.loop_device_path.take().unwrap();
        let output = Command::new("losetup")
            .arg("-d")
            .arg(loop_device)
            .output()?;

        if output.status.success() {
            self.loop_device_path = None;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to detach loop device"))
        }
    }
}

impl Drop for LoopDevice {
    fn drop(&mut self) {
        self.detach().expect("Failed to detach loop device");
    }
}

pub struct LoopDeviceBuilder {
    img_path: Option<PathBuf>,
}

impl LoopDeviceBuilder {
    pub fn new() -> Self {
        LoopDeviceBuilder { img_path: None }
    }

    pub fn img_path(mut self, img_path: PathBuf) -> Self {
        self.img_path = Some(img_path);
        self
    }

    pub fn build(self) -> Result<LoopDevice> {
        let mut loop_dev = LoopDevice {
            img_path: self.img_path.unwrap(),
            loop_device_path: None,
        };
        loop_dev.attach()?;
        Ok(loop_dev)
    }
}
