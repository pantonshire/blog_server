use axum::extract::Extension;
use maud::html;

use crate::{
    posts_store::ConcurrentPostsStore,
    template,
};
use super::response::Html;

pub async fn handle(Extension(posts): Extension<ConcurrentPostsStore>) -> Html {
    Html::new()
        .with_title_static("Pantonshire")
        .with_crawler_permissive()
        .with_head(html! {
            link href="/static/styles/main.css" rel="stylesheet";
        })
        .with_body(template::main_page(html! {
            section .content_section {
                h2 { "Who I am" }
                p {
                    "Hi! I'm Tom, a computer science student and hobbyist programmer."
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
                h2 { "Things I've made" }
                p { "Todo" }
            }

            section .content_section {
                h2 { "Articles" }
                ul {
                    @for post in posts.read().await.iter_by_created().rev().take(5) {
                        li {
                            a href={"/articles/" (post.id())} { (post.title()) }
                        }
                    }
                }
                p {
                    a href="/articles" { "See all" }
                }
            }
        }))
}
