use axum::extract::Extension;
use maud::html;

use blog::db::ConcurrentPostsStore;

use crate::template;

use super::response::Html;

pub async fn handle(Extension(posts): Extension<ConcurrentPostsStore>) -> Html {
    Html::new()
        .with_title_static("Pantonshire")
        .with_crawler_permissive()
        .with_head(html! {
            link href="/static/styles/main.css" rel="stylesheet";
            link rel="alternate" type="application/atom+xml" href="/atom.xml";
            link rel="alternate" type="application/rss+xml" href="/rss.xml";
        })
        .with_body(template::main_page(html! {
            section .content_section {
                p {
                    "Hi! I'm Tom, a computer science student and hobbyist programmer, and \
                     definitely not an egg-shaped robot."
                }
                figure {
                    img src="/static/images/tombot_450.png"
                        alt="A drawing by @smolrobots (on Twitter) of me as a robot. It has an \
                             egg-shaped body and a smiling face, and is doing a little dance next \
                             to a Raspberry Pi."
                        width="256";
                    figcaption .quiet {
                        "Drawn by "
                        a href="https://twitter.com/smolrobots" { "@smolrobots" }
                    }
                }
            }

            section .content_section {
                h2 { "Some things I've made" }
                ul {
                    li { a href="https://github.com/pantonshire/goldcrest" { "Goldcrest" } ", a proxy for the Twitter v1 API using gRPC. I made it to address the problem of centralised rate-limit tracking to avoid exceeding to Twitter's quotas." }
                    li { a href="https://github.com/pantonshire/smolbotbot" { "Smolbotbot" } ", an art archival project for the drawings of " a href="https://twitter.com/smolrobots" { "@smolrobots" } "! A Twitter bot continuously checks for new small robots, and the data it collects is made available at " a href="https://smolbotbot.com" { "smolbotbot.com" } "." }
                    li { a href="https://github.com/pantonshire/enumscribe" { "Enumscribe" } ", a Rust procedural macro library for generating conversion methods between enum types and strings, with support for " a href="https://serde.rs" { "Serde" } "." }
                    li { a href="https://github.com/pantonshire/tasque" { "Tasque" } ", a very in-progress cron-like job scheduler that I'm having a lot of fun working on." }
                }
            }

            section .content_section {
                h2 { "Articles" }
                p { "Some recent ones:" }
                ul .articles_list {
                    @for post in posts.read().await.iter_by_published().rev().take(3) {
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
                p {
                    a href="/articles" { "See the rest" }
                }
            }
        }))
}
