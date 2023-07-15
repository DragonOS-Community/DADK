use std::{fs::File, path::Path};

use reqwest::{blocking::ClientBuilder, Url};

pub struct FileUtils;

impl FileUtils {
    ///从指定url下载文件到指定路径
    pub fn download_file(url: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let tempurl = Url::parse(url).expect("failed to parse the url");
        let file_name = tempurl
            .path_segments()
            .unwrap()
            .last()
            .expect("failed to get the filename from the url");
        let client = ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        let mut response = client.get(url).send()?;
        let mut file = File::create(path.join(file_name))?;
        response.copy_to(&mut file)?;
        Ok(())
    }

    /// 把指定路径下所有文件和文件夹递归地移动到另一个文件中
    pub fn move_files(src: &Path, dst: &Path) -> std::io::Result<()> {
        for entry in src.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            let new_path = dst.join(path.file_name().unwrap());
            if entry.file_type()?.is_dir() {
                std::fs::create_dir_all(&new_path)?;
                FileUtils::move_files(&path, &new_path)?;
            } else {
                std::fs::rename(&path, &new_path)?;
            }
        }
        Ok(())
    }

    /// 递归地复制给定目录下所有文件到另一个文件夹中
    pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
        if src.is_dir() {
            std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;
            for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                let dst_path = dst.join(path.file_name().unwrap());
                if path.is_dir() {
                    FileUtils::copy_dir_all(&path, &dst_path).map_err(|e| e.to_string())?;
                } else {
                    std::fs::copy(&path, &dst_path).map_err(|e| e.to_string())?;
                }
            }
        } else {
            return Err(format!("No such source directory:{:?}", src));
        }
        Ok(())
    }
}
