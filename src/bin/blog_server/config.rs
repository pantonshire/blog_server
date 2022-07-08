use std::{time::Duration, net::SocketAddr, path::PathBuf, str};

use libshire::uuid::Uuid;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct Config {
    pub bind: SocketAddr,
    pub concurrency_limit: usize,
    #[serde(rename = "fs_event_delay_millis", deserialize_with = "deserialize_millis")]
    pub fs_event_delay: Duration,
    pub namespace_uuid: Uuid,
    pub content: ContentConfig,
    pub github: GithubConfig,
    pub site: SiteConfig,
    pub rss: RssConfig,
    pub atom: AtomConfig,
    pub contact: Vec<ContactConfig>,
}

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct SiteConfig {
    pub protocol: String,
    pub domain: String,
}

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct ContentConfig {
    pub static_dir: PathBuf,
    pub favicon_dir: PathBuf,
    pub robots_path: PathBuf,
    pub posts_dir: PathBuf,
    pub post_media_dir: PathBuf,
}

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct GithubConfig {
    pub edit_url: Option<String>,
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

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct ContactConfig {
    pub name: String,
    pub user: String,
    pub url: Option<String>,
}

impl str::FromStr for Config {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

fn deserialize_millis<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>
{
    u64::deserialize(deserializer)
        .map(Duration::from_millis)
}
