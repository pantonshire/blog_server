use std::sync::Arc;

use axum::extract::{Extension, Path};
use maud::html;

use crate::{Context, template};

use super::response::{Error, Html};

pub(super) async fn handle(
    Path(post_id): Path<String>,
    Extension(context): Extension<Arc<Context>>,
) -> Result<Html, Error>
{
    let post = context.posts().get(&post_id)
        .await
        .ok_or(Error::PostNotFound)?;

    Ok(Html::new()
        .with_crawler_permissive()
        .with_title_owned(post.title().to_owned())
        .with_head(html! {
            link href="/static/styles/main.css" rel="stylesheet";
            link href="/static/styles/code.css" rel="stylesheet";
            link rel="alternate" type="application/atom+xml" href="/atom.xml";
            link rel="alternate" type="application/rss+xml" href="/rss.xml";
        })
        .with_body(template::main_page(html! {
            section .article_header {
                h1 .article_title { (post.title()) }
                @if let Some(subtitle) = post.subtitle() {
                    p .article_subtitle { (subtitle) }
                }
                p .article_published_date { "Published " (post.published().format("%Y/%m/%d")) }
                @if let Some(source_url) = context.config().github.edit_url.as_deref() {
                    p .article_edit {
                        a href={(source_url) "/" (post.id()) ".toml.md"} {
                            "Propose a change on GitHub"
                        }
                    }
                }
            }
            article .article_content {
                (post.html())
            }
        })))
}
