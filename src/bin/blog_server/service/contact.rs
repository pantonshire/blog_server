use std::sync::Arc;

use axum::Extension;
use maud::html;

use crate::{template, Context};

use super::response::Html;

pub(super) async fn handle(Extension(context): Extension<Arc<Context>>) -> Html {
    Html::new()
        .with_title_static("Contact")
        .with_crawler_permissive()
        .with_head(html! {
            link href="/static/styles/main.css" rel="stylesheet";
        })
        .with_body(template::main_page(html! {
            section .content_section {
                h1 { "Contact" }
                p {
                    "If you want to contact me, you can find me at:"
                }
                ul {
                    @for contact in &context.config().contact {
                        li {
                            (contact.name) ": "
                            @if let Some(url) = contact.url.as_deref() {
                                a href=(url) { (contact.user) }
                            } @else {
                                (contact.user)
                            }
                        }
                    }
                }
            }
        }))
}
