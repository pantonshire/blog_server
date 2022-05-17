use std::borrow::Cow;
use std::fmt::{self, Write};

use axum::response::{IntoResponse, Response};
use axum::http::{self, StatusCode};
use maud::{html, Markup, Render, Escaper, DOCTYPE};

pub struct HtmlResponse {
    status: StatusCode,
    title: Cow<'static, str>,
    head: Option<Markup>,
    body: Option<Markup>,
    crawler_hints: CrawlerHints,
}

impl HtmlResponse {
    pub fn new() -> Self {
        Self {
            status: StatusCode::OK,
            title: Cow::Borrowed("untitled"),
            head: None,
            body: None,
            crawler_hints: CrawlerHints::restrictive(),
        }
    }

    pub fn with_status(self, status: StatusCode) -> Self {
        Self { status, ..self }
    }

    pub fn with_title(self, title: Cow<'static, str>) -> Self {
        Self { title, ..self }
    }

    pub fn with_title_static(self, title: &'static str) -> Self {
        self.with_title(Cow::Borrowed(title))
    }

    pub fn with_title_owned(self, title: String) -> Self {
        self.with_title(Cow::Owned(title))
    }

    pub fn with_head(self, head: Markup) -> Self {
        Self { head: Some(head), ..self }
    }

    pub fn with_body(self, body: Markup) -> Self {
        Self { body: Some(body), ..self }
    }

    pub fn with_crawler_hints(self, crawler_hints: CrawlerHints) -> Self {
        Self { crawler_hints, ..self }
    }

    pub fn with_crawler_restrictive(self) -> Self {
        self.with_crawler_hints(CrawlerHints::restrictive())
    }

    pub fn with_crawler_permissive(self) -> Self {
        self.with_crawler_hints(CrawlerHints::permissive())
    }
}

impl Default for HtmlResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoResponse for HtmlResponse {
    fn into_response(self) -> Response {
        let html_doc = html! {
            (DOCTYPE)
            html {
                head {
                    meta charset="utf-8";
                    meta name="robots" content=(self.crawler_hints);
                    title { (self.title) }
                    @if let Some(head) = self.head {
                        (head)
                    }
                }
                body {
                    @if let Some(body) = self.body {
                        (body)
                    }
                }
            }
        };

        let mut response = (self.status, html_doc.into_string())
            .into_response();

        response.headers_mut()
            .append("Content-Type", http::HeaderValue::from_static("text/html; charset=utf-8"));

        response
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CrawlerHints {
    index: bool,
    follow: bool,
    archive: bool,
    snippet: bool,
    image_index: bool,
}

impl CrawlerHints {
    pub const fn restrictive() -> Self {
        Self {
            index: false,
            follow: false,
            archive: false,
            snippet: false,
            image_index: false,
        }
    }

    pub const fn permissive() -> Self {
        Self {
            index: true,
            follow: true,
            archive: true,
            snippet: true,
            image_index: true,
        }
    }

    pub const fn with_index(self, index: bool) -> Self {
        Self { index, ..self }
    }

    pub const fn with_follow(self, follow: bool) -> Self {
        Self { follow, ..self }
    }

    pub const fn with_archive(self, archive: bool) -> Self {
        Self { archive, ..self }
    }

    pub const fn with_snippet(self, snippet: bool) -> Self {
        Self { snippet, ..self }
    }

    pub const fn with_image_index(self, image_index: bool) -> Self {
        Self { image_index, ..self }
    }

    fn index_str(self) -> &'static str {
        if self.index {
            "index"
        } else {
            "noindex"
        }
    }

    fn follow_str(self) -> &'static str {
        if self.follow {
            "follow"
        } else {
            "nofollow"
        }
    }

    fn archive_strs(self) -> Option<[&'static str; 2]> {
        if self.archive {
            None
        } else {
            Some(["noarchive", "nocache"])
        }
    }

    fn snippet_str(self) -> Option<&'static str> {
        if self.snippet {
            None
        } else {
            Some("nosnippet")
        }
    }

    fn image_index_str(self) -> Option<&'static str> {
        if self.image_index {
            None
        } else {
            Some("noimageindex")
        }
    }

    fn write_meta_list_to<W: Write>(self, mut buf: W) -> fmt::Result {
        write!(buf, "{},{}", self.index_str(), self.follow_str())?;
        if let Some([archive_str, cache_str]) = self.archive_strs() {
            write!(buf, ",{},{}", archive_str, cache_str)?;
        }
        if let Some(snippet_str) = self.snippet_str() {
            write!(buf, ",{}", snippet_str)?;
        }
        if let Some(image_index_str) = self.image_index_str() {
            write!(buf, ",{}", image_index_str)?;
        }
        Ok(())
    }
}

impl Render for CrawlerHints {
    fn render_to(&self, buf: &mut String) {
        let escaper = Escaper::new(buf);
        let _result = self.write_meta_list_to(escaper);
    }
}
