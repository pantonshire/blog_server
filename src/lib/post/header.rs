use chrono::{DateTime, Utc};
use kdl::KdlDocument;
use libshire::strings::ShString22;

use crate::time::{datetime_unix_seconds, unix_epoch};

use super::error::Error;

pub struct Header {
    pub(super) title: String,
    pub(super) subtitle: Option<String>,
    pub(super) author: ShString22,
    pub(super) tags: Vec<ShString22>,
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
    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    #[inline]
    #[must_use]
    pub fn author(&self) -> &str {
        &self.author
    }

    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[ShString22] {
        &self.tags
    }

    #[inline]
    #[must_use]
    pub fn published(&self) -> DateTime<Utc> {
        self.published
    }
}

impl<'a> TryFrom<&'a KdlDocument> for Header {
    type Error = Error;

    fn try_from(doc: &'a KdlDocument) -> Result<Self, Self::Error> {
        let title = doc
            .get_arg("title")
            .ok_or(Error::FieldMissing { field: "title" })
            .and_then(|value| value
                .as_string()
                .ok_or(Error::BadType { field: "title", expected: "string" })
                .map(|title| title.to_owned()))?;

        let subtitle = doc
            .get_arg("subtitle")
            .map(|value| value
                .as_string()
                .ok_or(Error::BadType { field: "subtitle", expected: "string" })
                .map(|subtitle| subtitle.to_owned()))
            .transpose()?;

        let author = doc
            .get_arg("title")
            .ok_or(Error::FieldMissing { field: "author" })
            .and_then(|value| value
                .as_string()
                .ok_or(Error::BadType { field: "author", expected: "string" })
                .map(ShString22::from))?;

        let tags = doc
            .get("tags")
            .map(|node| node.entries())
            .unwrap_or_default()
            .iter()
            .filter_map(|entry| match entry.name() {
                Some(_) => None,
                None => Some(entry.value()),
            })
            .map(|value| value
                .as_string()
                .ok_or(Error::BadType { field: "tag", expected: "string" })
                .map(ShString22::from))
            .collect::<Result<_, _>>()?;

        let published = doc
            .get_arg("published")
            .map(|value| value
                .as_i64()
                .ok_or(Error::BadType { field: "published", expected: "integer unix timestamp (seconds)" })
                .map(datetime_unix_seconds))
            .transpose()?
            .unwrap_or_else(unix_epoch);
        
        Ok(Header {
            title,
            subtitle,
            author,
            tags,
            published,
        })
    }
}

impl TryFrom<KdlDocument> for Header {
    type Error = Error;

    fn try_from(value: KdlDocument) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}
