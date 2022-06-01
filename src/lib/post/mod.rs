mod error;
mod id;
mod header;
mod markdown_post;
mod render;
mod rendered_post;
mod source;

pub use error::Error;
pub use header::Header;
pub use id::Id;
pub use markdown_post::MarkdownPost;
pub use rendered_post::RenderedPost;
pub use source::PostSource;

const POST_FILE_EXTENSION: &str = ".kdl.md";

pub type Post = RenderedPost;
