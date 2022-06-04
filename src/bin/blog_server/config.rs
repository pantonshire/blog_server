use std::{net::SocketAddr, path::PathBuf, str};

use libshire::uuid::Uuid;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct Config {
    pub bind: SocketAddr,
    pub concurrency_limit: usize,
    pub static_dir: PathBuf,
    pub favicon_dir: PathBuf,
    pub robots_path: PathBuf,
    pub posts_dir: PathBuf,
    pub post_media_dir: PathBuf,
    pub namespace_uuid: Uuid,
    pub self_ref: SelfRefConfig,
    pub rss: RssConfig,
    pub atom: AtomConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct SelfRefConfig {
    pub protocol: String,
    pub domain: String,
}

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct RssConfig {
    pub num_posts: usize,
    pub title: String,
    pub ttl: u32,
}

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct AtomConfig {
    pub num_posts: usize,
    pub title: String,
}

impl str::FromStr for Config {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}
