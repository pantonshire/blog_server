mod error;
mod id;
mod header;
mod render;
mod rendered_post;
mod source;

pub use error::Error;
pub use header::Header;
pub use id::Id;
pub use rendered_post::RenderedPost;
pub use source::PostSource;

const POST_FILE_EXTENSION: &str = ".toml.md";

pub type Post = RenderedPost;
