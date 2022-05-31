mod id;
mod parse;

use chrono::{DateTime, Utc};
use libshire::{strings::ShString22, uuid::Uuid};
use maud::{Markup, PreEscaped};

pub use id::PostId;
pub use parse::{parse, ParseError};

const POST_FILE_EXTENSION: &str = ".kdl.md";

pub struct Post {
    uuid: Uuid,
    id: PostId,
    title: String,
    subtitle: Option<String>,
    author: ShString22,
    html: Markup,
    tags: Vec<ShString22>,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}

impl Post {
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn id(&self) -> &PostId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn html(&self) -> PreEscaped<&str> {
        PreEscaped(&self.html.0)
    }

    pub fn tags(&self) -> &[ShString22] {
        &self.tags
    }

    pub fn created(&self) -> DateTime<Utc> {
        self.created
    }

    pub fn updated(&self) -> DateTime<Utc> {
        self.updated
    }
}
