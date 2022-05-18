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
                        li { a href="/" { "Articles" } }
                        li { a href="/" { "Atom" } }
                        li { a href="/" { "RSS" } }
                        li { a href="/" { "GitHub" } }
                        li { a href="/" { "Contact" } }
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
