use std::{fmt, str};

use chrono::{DateTime, Utc};
use libshire::strings::ShString22;
use serde::{Serialize, Deserialize};

use super::error::Error;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Header {
    pub(super) title: String,
    pub(super) subtitle: Option<String>,
    pub(super) author: ShString22,
    #[serde(default)]
    pub(super) tags: Vec<ShString22>,
    #[serde(default = "crate::time::unix_epoch")]
    pub(super) published: DateTime<Utc>,
}

impl Header {
    #[inline]
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[inline]
    #[must_use]
    pub fn title_mut(&mut self) -> &mut String {
        &mut self.title
    }

    #[inline]
    #[must_use]
    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    #[inline]
    #[must_use]
    pub fn subtitle_mut(&mut self) -> &mut Option<String> {
        &mut self.subtitle
    }

    #[inline]
    #[must_use]
    pub fn author(&self) -> &str {
        &self.author
    }

    #[inline]
    #[must_use]
    pub fn author_mut(&mut self) -> &mut ShString22 {
        &mut self.author
    }

    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[ShString22] {
        &self.tags
    }

    #[inline]
    #[must_use]
    pub fn tags_mut(&mut self) -> &mut Vec<ShString22> {
        &mut self.tags
    }

    #[inline]
    #[must_use]
    pub fn published(&self) -> DateTime<Utc> {
        self.published
    }

    #[inline]
    #[must_use]
    pub fn published_mut(&mut self) -> &mut DateTime<Utc> {
        &mut self.published
    }
}

impl str::FromStr for Header {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
            .map_err(Error::from)
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        toml::to_string_pretty(self)
            .map_err(|_| fmt::Error)
            .and_then(|s| f.write_str(&s))
    }
}
