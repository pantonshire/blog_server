use chrono::{DateTime, Utc};
use libshire::{strings::ShString22, uuid::{Uuid, UuidV5Error}};
use maud::{Markup, PreEscaped};

use crate::{codeblock::CodeBlockRenderer, time::unix_epoch};

use super::{
    error::Error,
    header::Header,
    id::Id,
    source::PostSource,
    render::render_markdown,
};

pub struct RenderedPost {
    uuid: Uuid,
    id: Id,
    header: Header,
    updated: DateTime<Utc>,
    html: Markup,
}

impl RenderedPost {
    pub fn new_from_str(
        code_renderer: &CodeBlockRenderer,
        namespace: Uuid,
        id: Id,
        updated: Option<DateTime<Utc>>,
        source: &str
    ) -> Result<Self, Error>
    {
        let source = source.parse::<PostSource>()?;
        Self::new_from_source(code_renderer, namespace, id, updated, source)
    }

    pub fn new_from_source(
        code_renderer: &CodeBlockRenderer,
        namespace: Uuid,
        id: Id,
        updated: Option<DateTime<Utc>>,
        source: PostSource
    ) -> Result<Self, Error>
    {
        let uuid = Uuid::new_v5(namespace, &*id)
            .map_err(|err| match err {
                UuidV5Error::NameTooLong(len) => Error::IdTooLong(len),
            })?;

        Ok(Self {
            uuid,
            id,
            header: source.header,
            updated: updated.unwrap_or_else(unix_epoch),
            html: render_markdown(code_renderer, &source.markdown), 
        })
    }

    #[inline]
    #[must_use]
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    #[inline]
    #[must_use]
    pub fn id(&self) -> &Id {
        &self.id
    }

    #[inline]
    #[must_use]
    pub fn header(&self) -> &Header {
        &self.header
    }

    #[inline]
    #[must_use]
    pub fn title(&self) -> &str {
        self.header.title()
    }

    #[inline]
    #[must_use]
    pub fn subtitle(&self) -> Option<&str> {
        self.header.subtitle()
    }

    #[inline]
    #[must_use]
    pub fn author(&self) -> &str {
        self.header.author()
    }

    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[ShString22] {
        self.header.tags()
    }

    #[inline]
    #[must_use]
    pub fn published(&self) -> DateTime<Utc> {
        self.header.published()
    }

    #[inline]
    #[must_use]
    pub fn updated(&self) -> DateTime<Utc> {
        self.updated
    }

    #[inline]
    #[must_use]
    pub fn html(&self) -> PreEscaped<&str> {
        PreEscaped(&self.html.0)
    }
}
