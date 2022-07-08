use std::sync::Arc;

use axum::extract::Extension;
use maud::html;

use crate::{Context, template};

use super::response::Html;

pub(super) async fn handle(Extension(context): Extension<Arc<Context>>) -> Html {
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
                    @for post in context.posts().read().await.iter_by_published().rev() {
                        li {
                            h3 { a href={"/articles/" (post.id())} { (post.title()) } }
                            @if let Some(subtitle) = post.subtitle() {
                                p .article_list_subtitle { (subtitle) }
                            }
                            p .article_list_published_date {
                                "Published " (post.published().format("%Y/%m/%d"))
                            }
                        }
                    }
                }
            }
        }))
}
