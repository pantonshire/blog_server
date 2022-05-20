use maud::html;

use crate::template;
use super::response::Html;

pub async fn handle() -> Html {
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
                    li {
                        "Twitter: "
                        a href="https://twitter.com/pantonshire" { "@pantonshire" }
                    }
                    li {
                        "Mastodon: "
                        a href="https://tech.lgbt/web/@pantonshire#" { "@pantonshire@tech.lgbt" }
                    }
                    li {
                        "Discord: pantonshire#2076"
                    }
                }
            }
        }))
}
