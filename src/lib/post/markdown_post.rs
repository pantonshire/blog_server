use super::{error::Error, header::Header, source::PostSource};

pub struct MarkdownPost {
    pub(super) header: Header,
    pub(super) markdown: String,
}

impl MarkdownPost {
    #[inline]
    #[must_use]
    pub fn header(&self) -> &Header {
        &self.header
    }

    #[inline]
    #[must_use]
    pub fn markdown(&self) -> &str {
        &self.markdown
    }
}

impl TryFrom<PostSource> for MarkdownPost {
    type Error = Error;

    fn try_from(source: PostSource) -> Result<Self, Self::Error> {
        Ok(Self {
            header: source.header.try_into()?,
            markdown: source.markdown,
        })
    }
}
