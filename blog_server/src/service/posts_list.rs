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
            link rel="alternate" type="application/atom+xml" href="/atom.xml";
            link rel="alternate" type="application/rss+xml" href="/rss.xml";
        })
        .with_body(template::main_page(html! {
            section .content_section {
                h1 { "Articles" }
                p {
                    "A collection of words I have written, against my better judgement."
                }
                ul .articles_list {
                    @for post in posts.read().await.iter_by_created().rev() {
                        li {
                            h3 { a href={"/articles/" (post.id())} { (post.title()) } }
                            @if let Some(subtitle) = post.subtitle() {
                                p .article_list_subtitle { (subtitle) }
                            }
                            p .article_list_published_date {
                                "Published " (post.created().format("%Y/%m/%d"))
                            }
                        }
                    }
                }
            }
        }))
}
