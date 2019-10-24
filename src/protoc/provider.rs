mod layout;
mod github;

use std::path::{Path, PathBuf};

use semver::Version;
use directories::BaseDirs;
use zip::ZipArchive;

pub enum DownloadError {
    Io(std::io::Error),
    Request(reqwest::Error),
    NotFound,
    Corrupted,
}

pub trait ProtocDownloader {
    fn download(&self, tag: &str, platform: &str, path: &Path) -> Result<String, DownloadError>;
}

pub struct ProtocProvider<D> {
    version: Version,
    content_path: PathBuf,
    binary_path: PathBuf,
    include_path: PathBuf,
    downloader: D,
}

impl<D> ProtocProvider<D>
    where
        D: ProtocDownloader
{
    pub fn new(version: Version, downloader: D) -> Option<Self> {
        let base_dirs = BaseDirs::new()?;
        let mut content_path = PathBuf::from(base_dirs.cache_dir());
        content_path.push(format!("v{}", &version));

        let mut binary_path = content_path.clone();
        layout::push_binary_path(&mut binary_path);

        let mut include_path = content_path.clone();
        layout::push_include_path(&mut include_path);

        Some(ProtocProvider { version, content_path, binary_path, include_path, downloader })
    }

    pub fn binary_path(&self) -> Option<&Path> {
        let path = &self.binary_path;
        if !path.exists() || !path.is_file() {
            return None;
        }

        Some(path.as_path())
    }

    pub fn include_path(&self) -> Option<&Path> {
        let path = &self.include_path;
        if !path.exists() || !path.is_dir() {
            return None;
        }

        Some(path.as_path())
    }

    pub fn download(&self) -> Result<(), DownloadError> {
        let tag = format!("v{}", self.version);
        let platform = layout::target_platform();

        self.clean_dir(&self.content_path)?;
        let zip_name = self.downloader.download(&tag, platform, &self.content_path)?;
        self.extract_zip(&zip_name)?;

        match (self.binary_path(), self.include_path()) {
            (Some(_), Some(_)) => Ok(()),
            _ => Err(DownloadError::Corrupted),
        }
    }

    fn clean_dir(&self, path: &Path) -> Result<(), std::io::Error> {
        for content in path.read_dir()? {
            let content = content?;
            if content.file_type()?.is_dir() {
                std::fs::remove_dir_all(&content.path())?;
            } else {
                std::fs::remove_file(&content.path())?;
            }
        }

        Ok(())
    }

    fn extract_zip(&self, name: &str) -> Result<(), std::io::Error> {
        let mut path = PathBuf::from(&self.content_path);
        path.push(name);

        let mut file = std::fs::File::open(&path)?;
        let mut archive = ZipArchive::new(&mut file)?;

        for i in 0..archive.len() {
            let mut zipfile = archive.by_index(i).unwrap();
            let out_path = zipfile.sanitized_name();

            if zipfile.name().ends_with('/') {
                std::fs::create_dir_all(&out_path)?;
            } else {
                if let Some(p) = out_path.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(&p)?;
                    }
                }

                let mut outfile = std::fs::File::create(&out_path)?;
                std::io::copy(&mut zipfile, &mut outfile)?;
            }

            #[cfg(unix)]
                self.set_permissions(&zipfile, &out_path)?;
        }

        Ok(())
    }

    #[cfg(unix)]
    fn set_permissions(&self, zipfile: &zip::read::ZipFile, path: &Path) -> std::io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        use std::fs::{set_permissions, Permissions};

        if let Some(mode) = zipfile.unix_mode() {
            return set_permissions(&path, Permissions::from_mode(mode));
        }

        Ok(())
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(e: std::io::Error) -> Self {
        DownloadError::Io(e)
    }
}

impl From<reqwest::Error> for DownloadError {
    fn from(e: reqwest::Error) -> Self {
        DownloadError::Request(e)
    }
}
