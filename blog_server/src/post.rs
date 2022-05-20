use std::{borrow, error, fmt, ops};

use chrono::{DateTime, Utc};
use libshire::strings::ShString22;
use maud::{Markup, PreEscaped};

use crate::codeblock::CodeBlockRenderer;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PostId(ShString22);

impl PostId {
    pub fn from_file_name(file_name: &str) -> Option<Self> {
        const POST_FILE_EXTENSION: &str = ".kdl.md";

        fn is_invalid_char(c: char) -> bool {
            c == '/' || c == '\\' || c == '.'
        }

        let prefix = file_name
            .strip_suffix(POST_FILE_EXTENSION)?;

        if prefix.contains(is_invalid_char) {
            return None;
        }

        Some(Self(ShString22::new_from_str(prefix)))
    }
}

impl ops::Deref for PostId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl ops::DerefMut for PostId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl AsRef<str> for PostId {
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsMut<str> for PostId {
    fn as_mut(&mut self) -> &mut str {
        self
    }
}

impl borrow::Borrow<str> for PostId {
    fn borrow(&self) -> &str {
        self
    }
}

impl borrow::BorrowMut<str> for PostId {
    fn borrow_mut(&mut self) -> &mut str {
        self
    }
}

impl fmt::Display for PostId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

pub struct Post {
    id: PostId,
    title: String,
    author: String,
    html: Markup,
    tags: Vec<ShString22>,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}

impl Post {
    pub fn id_str(&self) -> &str {
        &self.id
    }

    pub fn id(&self) -> &PostId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn html(&self) -> PreEscaped<&str> {
        PreEscaped(&self.html.0)
    }

    pub fn tags(&self) -> &[ShString22] {
        &self.tags
    }

    pub fn created(&self) -> DateTime<Utc> {
        self.created
    }

    pub fn updated(&self) -> DateTime<Utc> {
        self.updated
    }
    
    pub fn parse(
        code_renderer: &CodeBlockRenderer,
        post_id: PostId,
        file_name: &str,
        created: DateTime<Utc>,
        updated: DateTime<Utc>,
        source: &str,
    ) -> Result<Self, ParseError>
    {
        let mdpost = MdPost::parse(file_name, source)?;
        Ok(Self::from_mdpost(code_renderer, post_id, created, updated, mdpost))
    }

    fn from_mdpost(
        code_renderer: &CodeBlockRenderer,
        id: PostId,
        created: DateTime<Utc>,
        updated: DateTime<Utc>,
        mdpost: MdPost,
    ) -> Self
    {
        use pulldown_cmark::{Options, Parser, html::push_html};
        
        const PARSER_OPTIONS: Options = Options::ENABLE_TABLES
            .union(Options::ENABLE_FOOTNOTES)
            .union(Options::ENABLE_STRIKETHROUGH);

        let mut parser = PostMdParser::new(
            Parser::new_ext(&mdpost.markdown, PARSER_OPTIONS),
            code_renderer
        );

        let mut html_buf = String::new();
        push_html(&mut html_buf, parser.by_ref());
        
        Self {
            id,
            title: mdpost.title,
            author: mdpost.author,
            html: PreEscaped(html_buf),
            tags: mdpost.tags,
            created,
            updated,
        }
    }
}

/// Iterator struct which wraps another event iterator in order to render code blocks, collect the links
/// encountered and generate a summary of the text content.
struct PostMdParser<'p, I> {
    iter: I,
    code_renderer: &'p CodeBlockRenderer,
    links: Vec<String>,
    summary: String,
}

impl<'p, I> PostMdParser<'p, I> {
    fn new(iter: I, code_renderer: &'p CodeBlockRenderer) -> Self {
        Self {
            iter,
            code_renderer,
            links: Vec::new(),
            summary: String::new(),
        }
    }
}

impl<'e, 'p, I> Iterator for PostMdParser<'p, I> where I: Iterator<Item = pulldown_cmark::Event<'e>> {
    type Item = pulldown_cmark::Event<'e>;

    fn next(&mut self) -> Option<Self::Item> {
        use pulldown_cmark::{CodeBlockKind, CowStr, Event, LinkType, Tag};

        self.iter.next().map(|event| match event {
            // When we reach a code block, we want to collect the text content until the code block finishes
            // and have the `CodeBlockRenderer` render it
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                let mut code_buf = String::new();

                for event in self.iter.by_ref() {
                    match event {
                        // The code block has finished, so break out of the loop
                        Event::End(Tag::CodeBlock(_)) => break,
                        // All text events until the end of the code block should be considered as code, so
                        // add the text to the `code_buf` to be rendered later
                        Event::Text(text) => code_buf.push_str(&text),
                        // Ignore all other events
                        _ => (),
                    }
                }

                let highlighted = self.code_renderer.render(&lang, &code_buf);
                Event::Html(CowStr::Boxed(highlighted.into_string().into_boxed_str()))
            },

            event => {
                match &event {
                    Event::Start(Tag::Link(LinkType::Inline | LinkType::Autolink, destination, _title)) => {
                        self.links.push(destination.clone().into_string());
                    },

                    //TODO: better way of generating a summary
                    Event::Text(text) => {
                        if self.summary.is_empty() {
                            self.summary = text.clone().into_string();
                        }
                    },

                    _ => (),
                }

                event
            },
        })
    }
}

#[derive(knuffel::Decode)]
struct HeaderNode {
    #[knuffel(child, unwrap(argument))]
    title: String,
    #[knuffel(child, unwrap(argument))]
    author: String,
    #[knuffel(children(name="tag"))]
    tags: Vec<TagNode>,
}

#[derive(knuffel::Decode)]
struct TagNode {
    #[knuffel(argument)]
    tag: String,
}

#[derive(Debug)]
struct MdPost {
    markdown: String,
    title: String,
    author: String,
    tags: Vec<ShString22>,
}

impl MdPost {
    fn parse(file_name: &str, source: &str) -> Result<Self, ParseError> {
        const END_OF_HEADER_DELIM: &str = "\n---\n";

        let (header, md) = source.split_once(END_OF_HEADER_DELIM)
            .ok_or(ParseError::MissingHeader)?;

        let header = knuffel::parse::<HeaderNode>(file_name, header)
            .map_err(|err| ParseError::InvalidHeader(Box::new(err)))?;

        let md = md.trim_start();

        Ok(Self {
            markdown: md.to_owned(),
            title: header.title,
            author: header.author,
            tags: header.tags.into_iter().map(|tag| tag.tag.into()).collect(),
        })
    }
}

#[derive(Debug)]
pub enum ParseError {
    MissingHeader,
    InvalidHeader(Box<knuffel::Error>),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingHeader => write!(f, "Post file has no header"),
            ParseError::InvalidHeader(err) => fmt::Display::fmt(err, f),
        }
    }
}

impl error::Error for ParseError {}
