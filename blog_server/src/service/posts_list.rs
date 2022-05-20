use axum::extract::Extension;
use maud::html;

use crate::{
    posts_store::ConcurrentPostsStore,
    template,
};
use super::response::Html;

pub async fn handle(Extension(posts): Extension<ConcurrentPostsStore>) -> Html {
    Html::new()
        .with_title_static("Articles")
        .with_crawler_permissive()
        .with_head(html! {
            link href="/static/styles/main.css" rel="stylesheet";
        })
        .with_body(template::main_page(html! {
            section .content_section {
                h1 { "Articles" }
                p {
                    "A collection of words I have written, against my better judgement."
                }
                ul {
                    @for post in posts.read().await.iter_by_created().rev() {
                        li {
                            a href={"/articles/" (post.id_str())} { (post.title()) }
                            span class="quiet" {
                                " — " (post.created().format("%Y/%m/%d"))
                            }
                        }
                    }
                }
            }
        }))
}
