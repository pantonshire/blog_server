use maud::{html, Markup, PreEscaped};
use syntect::html::{ClassedHTMLGenerator, ClassStyle};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub const CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed { prefix: "cb_" };

pub struct CodeBlockRenderer {
    syntax_set: SyntaxSet,
}

impl CodeBlockRenderer {
    pub fn new() -> Self {
        // Load Syntect's default syntax set from Sublime syntax definitions embedded in the
        // binary.
        let default_syntax_set = SyntaxSet::load_defaults_newlines();
        Self::new_with_syntax_set(default_syntax_set)
    }

    pub fn new_with_syntax_set(syntax_set: SyntaxSet) -> Self {
        Self {
            syntax_set,
        }
    }

    pub fn render(&self, lang: &str, source: &str) -> Markup {
        const CONTEXT_DELIM: &str = "@@";

        // Grab the optional context information between @@s from the first line of the code block.
        let (context, source) = source.split_once('\n')
            .and_then(|(context, source)| context
                .trim()
                .strip_prefix(CONTEXT_DELIM)
                .and_then(|context| context.strip_suffix(CONTEXT_DELIM))
                .map(|context| (Some(context.trim()), source)))
            .unwrap_or((None, source));

        // Search the syntax set for the syntax definition for the language specified for the code
        // block (after the triple backtick), and default to plaintext if no syntax definition is
        // found.
        let syntax = self.syntax_set
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut html_gen = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &self.syntax_set,
            CLASS_STYLE
        );

        for line in LinesWithEndings::from(source) {
            html_gen.parse_html_for_line_which_includes_newline(line);
        }

        let html_out = html_gen.finalize();

        html! {
            .codeblock {
                @if context.is_some() || !lang.is_empty() {
                    .codeblock_banner {
                        span .codeblock_language { (lang) }
                        span .codeblock_context { (context.unwrap_or("")) }
                    }
                }
                pre .codeblock_code {
                    code {
                        (PreEscaped(html_out))
                    }
                }
            }
        }
    }
}

impl Default for CodeBlockRenderer {
    fn default() -> Self {
        Self::new()
    }
}
