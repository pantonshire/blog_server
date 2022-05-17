use axum::extract::Extension;
use maud::html;

use crate::posts_store::ConcurrentPostsStore;
use super::response::HtmlResponse;

pub async fn handle(Extension(posts): Extension<ConcurrentPostsStore>) -> HtmlResponse {
    HtmlResponse::new()
        .with_title_static("Placeholder title")
        .with_crawler_permissive()
        .with_body(html! {
            h1 { "Here is my great heading" }
            p { "Hello world" }
            ul {
                @for post in posts.read().await.iter_by_created().rev() {
                    li {
                        a href={ "/posts/" (post.id_str()) } {
                            (post.title())
                        };
                    }
                }
            }
        })
}
