use axum::extract::{Extension, Path};
use maud::html;

use crate::posts_store::ConcurrentPostsStore;
use super::response::{ErrorResponse, HtmlResponse};

pub async fn handle(
    Path(post_id): Path<String>,
    Extension(posts): Extension<ConcurrentPostsStore>
) -> Result<HtmlResponse, ErrorResponse>
{
    let post = posts.get(&post_id)
        .await
        .ok_or(ErrorResponse::PostNotFound)?;

    Ok(HtmlResponse::new()
        .with_crawler_permissive()
        .with_title_owned(post.title().to_owned())
        .with_head(html! {
            link href="/static/style/code.css" rel="stylesheet";
        })
        .with_body(html! {
            h1 { (post.title()) }
            p { "by " (post.author()) }
            article {
                (post.html())
            }
        }))
}
