use maud::{html, Markup, PreEscaped};
use pulldown_cmark::{
    CodeBlockKind,
    CowStr,
    Event,
    Options,
    Parser,
    Tag,
    html::push_html,
};

use crate::codeblock::CodeBlockRenderer;

pub(super) fn render_markdown(code_renderer: &CodeBlockRenderer, markdown: &str) -> Markup {
    const PARSER_OPTIONS: Options = Options::ENABLE_TABLES
        .union(Options::ENABLE_FOOTNOTES)
        .union(Options::ENABLE_STRIKETHROUGH);

    let mut parser = {
        let parser = Parser::new_ext(markdown, PARSER_OPTIONS);
        PostMdParser::new(parser, code_renderer)
    };

    let mut html_buf = String::new();
    push_html(&mut html_buf, parser.by_ref());

    PreEscaped(html_buf)
}

/// Iterator struct which wraps another event iterator in order to render code blocks, collect the links
/// encountered and generate a summary of the text content.
struct PostMdParser<'p, I> {
    iter: I,
    code_renderer: &'p CodeBlockRenderer,
}

impl<'p, I> PostMdParser<'p, I> {
    fn new(iter: I, code_renderer: &'p CodeBlockRenderer) -> Self {
        Self {
            iter,
            code_renderer,
        }
    }
}

impl<'e, 'p, I> Iterator for PostMdParser<'p, I> where I: Iterator<Item = pulldown_cmark::Event<'e>> {
    type Item = pulldown_cmark::Event<'e>;

    fn next(&mut self) -> Option<Self::Item> {
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

            event => event,
        })
    }
}
