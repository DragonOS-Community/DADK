use std::{fs::File, io::Write, mem::ManuallyDrop, path::PathBuf, process::Command};

use crate::context::DADKExecContext;
use anyhow::{anyhow, Result};
use dadk_config::rootfs::{fstype::FsType, partition::PartitionType};

use super::loopdev::LoopDeviceBuilder;
pub(super) fn create(ctx: &DADKExecContext, skip_if_exists: bool) -> Result<()> {
    let disk_image_path = ctx.disk_image_path();
    if disk_image_path.exists() {
        if skip_if_exists {
            return Ok(());
        }
        return Err(anyhow!(
            "Disk image already exists: {}",
            disk_image_path.display()
        ));
    }

    disk_path_safety_check(&disk_image_path)?;

    // 获取镜像大小
    let image_size = ctx.disk_image_size();
    create_raw_img(&disk_image_path, image_size).expect("Failed to create raw disk image");

    // 判断是否需要分区？

    let r = if ctx.rootfs().partition.image_should_be_partitioned() {
        create_partitioned_image(ctx, &disk_image_path)
    } else {
        create_unpartitioned_image(ctx, &disk_image_path)
    };

    if r.is_err() {
        std::fs::remove_file(&disk_image_path).expect("Failed to remove disk image");
    }
    r
}

pub(super) fn delete(ctx: &DADKExecContext, skip_if_not_exists: bool) -> Result<()> {
    let disk_image_path = ctx.disk_image_path();
    if !disk_image_path.exists() {
        if skip_if_not_exists {
            return Ok(());
        }
        return Err(anyhow!(
            "Disk image does not exist: {}",
            disk_image_path.display()
        ));
    }
    disk_path_safety_check(&disk_image_path)?;

    std::fs::remove_file(&disk_image_path)
        .map_err(|e| anyhow!("Failed to remove disk image: {}", e))?;
    Ok(())
}

pub fn mount(ctx: &DADKExecContext) -> Result<()> {
    let disk_image_path = ctx.disk_image_path();
    if !disk_image_path.exists() {
        return Err(anyhow!(
            "Disk image does not exist: {}",
            disk_image_path.display()
        ));
    }
    let disk_mount_path = ctx.disk_mount_path();

    // 尝试创建挂载点
    std::fs::create_dir_all(&disk_mount_path)
        .map_err(|e| anyhow!("Failed to create disk mount path: {}", e))?;

    let partitioned = ctx.rootfs().partition.image_should_be_partitioned();
    log::trace!("Disk image is partitioned: {}", partitioned);
    if partitioned {
        mount_partitioned_image(ctx, &disk_image_path, &disk_mount_path)?
    } else {
        mount_unpartitioned_image(ctx, &disk_image_path, &disk_mount_path)?
    }
    log::info!("Disk image mounted at {}", disk_mount_path.display());
    Ok(())
}

fn mount_partitioned_image(
    ctx: &DADKExecContext,
    disk_image_path: &PathBuf,
    disk_mount_path: &PathBuf,
) -> Result<()> {
    let mut loop_device = ManuallyDrop::new(
        LoopDeviceBuilder::new()
            .img_path(disk_image_path.clone())
            .build()
            .map_err(|e| anyhow!("Failed to create loop device: {}", e))?,
    );

    loop_device
        .attach()
        .map_err(|e| anyhow!("mount: Failed to attach loop device: {}", e))?;

    let dev_path = loop_device.partition_path(1)?;
    mount_unpartitioned_image(ctx, &dev_path, disk_mount_path)?;

    Ok(())
}

fn mount_unpartitioned_image(
    _ctx: &DADKExecContext,
    disk_image_path: &PathBuf,
    disk_mount_path: &PathBuf,
) -> Result<()> {
    let cmd = Command::new("mount")
        .arg(disk_image_path)
        .arg(disk_mount_path)
        .output()
        .map_err(|e| anyhow!("Failed to mount disk image: {}", e))?;
    if !cmd.status.success() {
        return Err(anyhow!(
            "Failed to mount disk image: {}",
            String::from_utf8_lossy(&cmd.stderr)
        ));
    }
    Ok(())
}

pub fn umount(ctx: &DADKExecContext) -> Result<()> {
    let disk_img_path = ctx.disk_image_path();
    let disk_mount_path = ctx.disk_mount_path();
    let mut loop_device = LoopDeviceBuilder::new().img_path(disk_img_path).build();

    let should_detach_loop_device: bool;
    if let Ok(loop_device) = loop_device.as_mut() {
        if let Err(e) = loop_device.attach_by_exists() {
            log::trace!("umount: Failed to attach loop device: {}", e);
        }

        should_detach_loop_device = loop_device.attached();
    } else {
        should_detach_loop_device = false;
    }

    if disk_mount_path.exists() {
        let cmd = Command::new("umount")
            .arg(disk_mount_path)
            .output()
            .map_err(|e| anyhow!("Failed to umount disk image: {}", e));
        match cmd {
            Ok(cmd) => {
                if !cmd.status.success() {
                    let e = anyhow!(
                        "Failed to umount disk image: {}",
                        String::from_utf8_lossy(&cmd.stderr)
                    );
                    if should_detach_loop_device {
                        log::error!("{}", e);
                    } else {
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                if should_detach_loop_device {
                    log::error!("{}", e);
                } else {
                    return Err(e);
                }
            }
        }
    }

    if let Ok(mut loop_device) = loop_device {
        let loop_dev_path = loop_device.dev_path().cloned();
        loop_device.detach().ok();

        log::info!("Loop device detached: {:?}", loop_dev_path);
    }

    Ok(())
}

/// Ensures the provided disk image path is not a device node.
fn disk_path_safety_check(disk_image_path: &PathBuf) -> Result<()> {
    const DONT_ALLOWED_PREFIX: [&str; 5] =
        ["/dev/sd", "/dev/hd", "/dev/vd", "/dev/nvme", "/dev/mmcblk"];
    let path = disk_image_path.to_str().ok_or(anyhow!(
        "disk path safety check failed: disk path is not valid utf-8"
    ))?;

    DONT_ALLOWED_PREFIX.iter().for_each(|prefix| {
        if path.starts_with(prefix) {
            panic!("disk path safety check failed: disk path is not allowed to be a device node(except loop dev)");
        }
    });
    Ok(())
}

fn create_partitioned_image(ctx: &DADKExecContext, disk_image_path: &PathBuf) -> Result<()> {
    let part_type = ctx.rootfs().partition.partition_type;
    DiskPartitioner::create_partitioned_image(disk_image_path, part_type)?;
    // 挂载loop设备
    let mut loop_device = LoopDeviceBuilder::new()
        .img_path(disk_image_path.clone())
        .build()
        .map_err(|e| anyhow!("Failed to create loop device: {}", e))?;
    loop_device
        .attach()
        .map_err(|e| anyhow!("creat: Failed to attach loop device: {}", e))?;

    let partition_path = loop_device.partition_path(1)?;
    let fs_type = ctx.rootfs().metadata.fs_type;
    DiskFormatter::format_disk(&partition_path, &fs_type)?;
    loop_device.detach()?;
    Ok(())
}

fn create_unpartitioned_image(ctx: &DADKExecContext, disk_image_path: &PathBuf) -> Result<()> {
    // 直接对整块磁盘镜像进行格式化
    let fs_type = ctx.rootfs().metadata.fs_type;
    DiskFormatter::format_disk(disk_image_path, &fs_type)
}

/// 创建全0的raw镜像
fn create_raw_img(disk_image_path: &PathBuf, image_size: usize) -> Result<()> {
    log::trace!("Creating raw disk image: {}", disk_image_path.display());
    // 创建父目录
    if let Some(parent) = disk_image_path.parent() {
        log::trace!("Creating parent directory: {}", parent.display());
        std::fs::create_dir_all(parent)?;
    }
    // 打开或创建文件
    let mut file = File::create(disk_image_path)?;

    // 将文件大小设置为指定大小
    file.set_len(image_size.try_into().unwrap())?;

    // 写入全0数据
    let zero_buffer = vec![0u8; 4096]; // 4KB buffer for writing zeros
    let mut remaining_size = image_size;

    while remaining_size > 0 {
        let write_size = std::cmp::min(remaining_size, zero_buffer.len());
        file.write_all(&zero_buffer[..write_size as usize])?;
        remaining_size -= write_size;
    }

    Ok(())
}

pub fn check_disk_image_exists(ctx: &DADKExecContext) -> Result<()> {
    let disk_image_path = ctx.disk_image_path();
    if disk_image_path.exists() {
        println!("1");
    } else {
        println!("0");
    }
    Ok(())
}

pub fn show_mount_point(ctx: &DADKExecContext) -> Result<()> {
    let disk_mount_path = ctx.disk_mount_path();
    println!("{}", disk_mount_path.display());
    Ok(())
}

pub fn show_loop_device(ctx: &DADKExecContext) -> Result<()> {
    let disk_image_path = ctx.disk_image_path();
    let mut loop_device = LoopDeviceBuilder::new().img_path(disk_image_path).build()?;
    if let Err(e) = loop_device.attach_by_exists() {
        log::error!("Failed to attach loop device: {}", e);
    } else {
        println!("{}", loop_device.dev_path().unwrap());
    }
    Ok(())
}

struct DiskPartitioner;

impl DiskPartitioner {
    fn create_partitioned_image(disk_image_path: &PathBuf, part_type: PartitionType) -> Result<()> {
        match part_type {
            PartitionType::None => {
                // This case should not be reached as we are in the partitioned image creation function
                return Err(anyhow::anyhow!("Invalid partition type: None"));
            }
            PartitionType::Mbr => {
                // Create MBR partitioned disk image
                Self::create_mbr_partitioned_image(disk_image_path)?;
            }
            PartitionType::Gpt => {
                // Create GPT partitioned disk image
                Self::create_gpt_partitioned_image(disk_image_path)?;
            }
        }
        Ok(())
    }

    fn create_mbr_partitioned_image(disk_image_path: &PathBuf) -> Result<()> {
        let disk_image_path_str = disk_image_path.to_str().expect("Invalid path");

        // 检查 fdisk 是否存在
        let output = Command::new("fdisk")
            .arg("--help")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?
            .wait_with_output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Command fdisk not found"));
        }

        // 向 fdisk 发送命令
        let fdisk_commands = "o\nn\n\n\n\n\na\nw\n";
        let mut fdisk_child = Command::new("fdisk")
            .arg(disk_image_path_str)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        let fdisk_stdin = fdisk_child.stdin.as_mut().expect("Failed to open stdin");
        fdisk_stdin.write_all(fdisk_commands.as_bytes())?;
        fdisk_stdin.flush()?;
        fdisk_child
            .wait()
            .unwrap_or_else(|e| panic!("Failed to run fdisk: {}", e));
        Ok(())
    }

    fn create_gpt_partitioned_image(_disk_image_path: &PathBuf) -> Result<()> {
        // Implement the logic to create a GPT partitioned disk image
        // This is a placeholder for the actual implementation
        unimplemented!("Not implemented: create_gpt_partitioned_image");
    }
}

struct DiskFormatter;

impl DiskFormatter {
    fn format_disk(disk_image_path: &PathBuf, fs_type: &FsType) -> Result<()> {
        match fs_type {
            FsType::Fat32 => Self::format_fat32(disk_image_path),
        }
    }

    fn format_fat32(disk_image_path: &PathBuf) -> Result<()> {
        // Use the `mkfs.fat` command to format the disk image as FAT32
        let status = Command::new("mkfs.fat")
            .arg("-F32")
            .arg(disk_image_path.to_str().unwrap())
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to format disk image as FAT32"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_create_raw_img_functional() -> Result<()> {
        // 创建一个临时文件路径
        let temp_file = NamedTempFile::new()?;
        let disk_image_path = temp_file.path().to_path_buf();
        let disk_image_size = 1024 * 1024usize;

        // 调用函数
        create_raw_img(&disk_image_path, disk_image_size)?;

        // 验证文件大小
        let metadata = fs::metadata(&disk_image_path)?;
        assert_eq!(metadata.len(), disk_image_size as u64);

        // 验证文件内容是否全为0
        let mut file = File::open(&disk_image_path)?;
        let mut buffer = vec![0u8; 4096];
        let mut all_zeros = true;

        while file.read(&mut buffer)? > 0 {
            for byte in &buffer {
                if *byte != 0 {
                    all_zeros = false;
                    break;
                }
            }
        }

        assert!(all_zeros, "File content is not all zeros");

        Ok(())
    }

    #[test]
    fn test_format_fat32() {
        // Create a temporary file to use as the disk image
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let disk_image_path = temp_file.path().to_path_buf();

        // 16MB
        let image_size = 16 * 1024 * 1024usize;
        create_raw_img(&disk_image_path, image_size).expect("Failed to create raw disk image");

        // Call the function to format the disk image
        DiskFormatter::format_disk(&disk_image_path, &FsType::Fat32)
            .expect("Failed to format disk image as FAT32");

        // Optionally, you can check if the disk image was actually formatted as FAT32
        // by running a command to inspect the filesystem type
        let output = Command::new("file")
            .arg("-sL")
            .arg(&disk_image_path)
            .output()
            .expect("Failed to execute 'file' command");

        let output_str = String::from_utf8_lossy(&output.stdout);
        assert!(
            output_str.contains("FAT (32 bit)"),
            "Disk image is not formatted as FAT32"
        );
    }

    #[test]
    fn test_create_mbr_partitioned_image() -> Result<()> {
        // Create a temporary file to use as the disk image
        let temp_file = NamedTempFile::new()?;
        let disk_image_path = temp_file.path().to_path_buf();

        eprintln!("Disk image path: {:?}", disk_image_path);
        // Create a raw disk image
        let disk_image_size = 16 * 1024 * 1024usize; // 16MB
        create_raw_img(&disk_image_path, disk_image_size)?;

        // Call the function to create the MBR partitioned image
        DiskPartitioner::create_mbr_partitioned_image(&disk_image_path)?;

        // Verify the disk image has been correctly partitioned
        let output = Command::new("fdisk")
            .env("LANG", "C") // Set LANG to C to force English output
            .env("LC_ALL", "C") // Set LC_ALL to C to force English output
            .arg("-l")
            .arg(&disk_image_path)
            .output()
            .expect("Failed to execute 'fdisk -l' command");

        let output_str = String::from_utf8_lossy(&output.stdout);
        assert!(
            output_str.contains("Disklabel type: dos"),
            "Disk image does not have an MBR partition table"
        );
        assert!(
            output_str.contains("Start"),
            "Disk image does not have a partition"
        );

        Ok(())
    }
}
