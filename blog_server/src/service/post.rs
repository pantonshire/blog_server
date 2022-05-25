use axum::extract::{Extension, Path};
use maud::html;

use crate::{
    posts_store::ConcurrentPostsStore,
    template,
};
use super::response::{Error, Html};

pub async fn handle(
    Path(post_id): Path<String>,
    Extension(posts): Extension<ConcurrentPostsStore>
) -> Result<Html, Error>
{
    let post = posts.get(&post_id)
        .await
        .ok_or(Error::PostNotFound)?;

    Ok(Html::new()
        .with_crawler_permissive()
        .with_title_owned(post.title().to_owned())
        .with_head(html! {
            link href="/static/styles/main.css" rel="stylesheet";
            link href="/static/styles/code.css" rel="stylesheet";
        })
        .with_body(template::main_page(html! {
            section .article_header {
                h1 .article_title { (post.title()) }
                @if let Some(subtitle) = post.subtitle() {
                    p .article_subtitle { (subtitle) }
                }
                p .article_published_date { "Published " (post.created().format("%Y/%m/%d")) }
            }
            article .article_content {
                (post.html())
            }
        })))
}
