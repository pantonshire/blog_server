use std::{error, fmt};

use chrono::{DateTime, Utc};
use libshire::{strings::ShString22, uuid::{Uuid, UuidV5Error}};
use maud::{html, PreEscaped};

use crate::codeblock::CodeBlockRenderer;

use super::{id::PostId, Post};

pub fn parse(
    code_renderer: &CodeBlockRenderer,
    namespace: Uuid,
    post_id: PostId,
    file_name: &str,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
    source: &str,
) -> Result<Post, ParseError>
{
    MdPost::parse(file_name, source)
        .and_then(|post| render_mdpost(
            code_renderer,
            namespace,
            post_id,
            created,
            updated,
            post
        ))
}

fn render_mdpost(
    code_renderer: &CodeBlockRenderer,
    namespace: Uuid,
    id: PostId,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
    mdpost: MdPost,
) -> Result<Post, ParseError>
{
    use pulldown_cmark::{Options, Parser, html::push_html};
    
    const PARSER_OPTIONS: Options = Options::ENABLE_TABLES
        .union(Options::ENABLE_FOOTNOTES)
        .union(Options::ENABLE_STRIKETHROUGH);

    let uuid = Uuid::new_v5(namespace, &*id)
        .map_err(|err| match err {
            UuidV5Error::NameTooLong(len) => ParseError::IdTooLong(len),
        })?;

    let mut parser = PostMdParser::new(
        Parser::new_ext(&mdpost.markdown, PARSER_OPTIONS),
        code_renderer
    );

    let mut html_buf = String::new();
    push_html(&mut html_buf, parser.by_ref());
    
    Ok(Post {
        uuid,
        id,
        title: mdpost.title,
        subtitle: mdpost.subtitle,
        author: mdpost.author,
        html: PreEscaped(html_buf),
        tags: mdpost.tags,
        created,
        updated,
    })
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

            Event::Code(code) => {
                Event::Html(CowStr::Boxed(html! {
                    code .inline_code { (code) }
                }.into_string().into_boxed_str()))
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
    subtitle: Option<String>,
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
    subtitle: Option<String>,
    author: ShString22,
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
            subtitle: header.subtitle,
            author: header.author.into(),
            tags: header.tags.into_iter().map(|tag| tag.tag.into()).collect(),
        })
    }
}

#[derive(Debug)]
pub enum ParseError {
    MissingHeader,
    InvalidHeader(Box<knuffel::Error>),
    IdTooLong(usize),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHeader => write!(f, "post file has no header"),
            Self::InvalidHeader(err) => fmt::Display::fmt(err, f),
            Self::IdTooLong(len) => write!(f, "post id too long ({} bytes)", len),
        }
    }
}

impl error::Error for ParseError {}
