use maud::{html, Markup};

pub fn main_page(content: Markup) -> Markup {
    html! {
        header #page_header .sticky_header {
            nav #page_nav {
                #title_box {
                    a href="/" { "Pantonshire" }
                }
                #right_nav_box {
                    ul {
                        li { a href="/articles" { "Articles" } }
                        li { a href="/atom.xml" { "Atom" } }
                        li { a href="/rss.xml" { "RSS" } }
                        li { a href="https://github.com/pantonshire" { "GitHub" } }
                        li { a href="/contact" { "Contact" } }
                    }
                }
            }
        }

        main #page_main {
            #content {
                (content)
            }
        }

        footer #page_footer {
            #page_footer_content {
                span { "Here is some footer text" }
            }
        }
    }
}
