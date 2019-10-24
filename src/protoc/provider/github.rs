use std::path::{Path, PathBuf};

use reqwest::Client;
use serde::Deserialize;
use regex::Regex;

use super::DownloadError;
use super::ProtocDownloader;
use std::time::Duration;

pub struct GithubDownloader {
    client: Client,
    name_regex: Regex,
}

#[derive(Deserialize)]
struct Release {
    id: i32,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    id: i32,
    name: String,
    url: String,
}

impl GithubDownloader {
    pub fn new(client: Client) -> Self {
        let name_regex = Regex::new(r#"protoc-.+-((linux|osx|win).+).zip"#).unwrap();
        Self { client, name_regex }
    }

    fn download_asset(&self, asset: Asset, path: &Path) -> Result<String, DownloadError> {
        let mut path = PathBuf::from(path);
        path.push(&asset.name);

        let mut zip = std::fs::File::create(&path)?;
        self.client.get(&asset.url).send()?.copy_to(&mut zip)?;

        Ok(asset.name)
    }
}

impl ProtocDownloader for GithubDownloader {
    fn download(&self, tag: &str, platform: &str, path: &Path) -> Result<String, DownloadError> {
        let url = format!("https://api.github.com/repos/protocolbuffers/protobuf/releases/tags/{}", tag);
        let release = self.client.get(&url).send()?.json::<Release>()?;

        for asset in release.assets {
            let captures = match self.name_regex.captures(&asset.name) {
                Some(captures) => captures,
                None => continue,
            };

            let asset_platform = match captures.get(1) {
                Some(capture) => capture,
                None => continue,
            };

            if asset_platform.as_str() == platform {
                return self.download_asset(asset, path)
            }
        }

        Err(DownloadError::NotFound)
    }
}

impl Default for GithubDownloader {
    fn default() -> Self {
        let client = reqwest::ClientBuilder::new()
            .gzip(true)
            .connect_timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        GithubDownloader::new(client)
    }
}
