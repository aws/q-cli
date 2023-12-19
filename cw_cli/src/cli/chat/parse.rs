use winnow::ascii::{
    line_ending,
    multispace0,
    not_line_ending,
};
use winnow::combinator::{
    alt,
    delimited,
    rest,
};
use winnow::error::ParserError;
use winnow::prelude::*;
use winnow::token::{
    take_till,
    take_until1,
    take_while,
};
use winnow::Partial;

#[derive(Debug, Clone, PartialEq)]
enum MarkdownElements {
    Heading {
        level: u8,
        content: String,
    },
    Paragraph {
        content: String,
    },
    List {
        items: Vec<String>,
        ordered: bool,
    },
    CodeBlock {
        content: String,
        language: Option<String>,
    },
    Image {
        src: String,
        alt: String,
    },
    Link {
        href: String,
        text: String,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    BlockQuote {
        content: String,
    },
    HorizontalRule,
    Bold {
        content: String,
    },
    Italic {
        content: String,
    },
    Strikethrough {
        content: String,
    },
    InlineCode {
        content: String,
    },
    /// Not a markdown element, just spacing
    NewLine,
}

type Stream<'i> = Partial<&'i str>;

fn parse<'i>(input: &mut Stream<'i>) -> PResult<Vec<MarkdownElements>> {
    let mut elements = Vec::new();

    loop {
        match alt((heading, codeblock, paragraph)).parse_next(input) {
            Ok(element) => {
                elements.push(element);
            },
            Err(err) if err.is_incomplete() => {
                break;
            },
            Err(err) => return Err(err),
        }
    }

    Ok(elements)
}

// Markdown heading (#, ##, ###, etc.)
fn heading<'i, E: ParserError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<MarkdownElements, E> {
    let (a, _, b) = (take_while(1.., |c| c == '#'), multispace0, not_line_ending).parse_next(input)?;
    Ok(MarkdownElements::Heading {
        level: a.len() as u8,
        content: b.to_string(),
    })
}

// Codeblock
fn codeblock<'i, E: ParserError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<MarkdownElements, E> {
    "```".parse_next(input)?;
    let language = not_line_ending.parse_next(input)?;
    line_ending.parse_next(input)?;
    let content = take_until1("```").parse_next(input)?.into();
    "```".parse_next(input)?;

    Ok(MarkdownElements::CodeBlock {
        content,
        language: if language.is_empty() {
            None
        } else {
            Some(language.into())
        },
    })
}

fn paragraph<'i, E: ParserError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<MarkdownElements, E> {
    let content: String = take_till(0.., |c| c == '\n' || c == '\r').parse_next(input)?.into();
    line_ending.parse_next(input)?;
    if content.is_empty() {
        Ok(MarkdownElements::NewLine)
    } else {
        Ok(MarkdownElements::Paragraph { content })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_stream {
        ($input:expr, $expected:expr) => {
            let mut input = Partial::new($input);
            assert_eq!(parse(&mut input).unwrap(), $expected);
        };
    }

    #[test]
    fn test_heading() {
        assert_stream!("# Hello world\n", vec![MarkdownElements::Heading {
            level: 1,
            content: "Hello world".to_string(),
        }]);

        assert_stream!("## Hello world\n", vec![MarkdownElements::Heading {
            level: 2,
            content: "Hello world".to_string(),
        }]);
    }

    #[test]
    fn test_codeblock() {
        assert_stream!(
            "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```\n",
            vec![MarkdownElements::CodeBlock {
                content: "fn main() {\n    println!(\"Hello, world!\");\n}\n".to_string(),
                language: Some("rust".to_string()),
            }]
        );

        assert_stream!("```\nfn main() {\n    println!(\"Hello, world!\");\n}\n```\n", vec![
            MarkdownElements::CodeBlock {
                content: "fn main() {\n    println!(\"Hello, world!\");\n}\n".to_string(),
                language: None,
            }
        ]);
    }

    #[test]
    fn doc() {
        let md_doc = "## Hello world\n\nThis is a paragraph.\n\n## Another heading\n\nThis is another paragraph.";

        assert_stream!(md_doc, vec![
            MarkdownElements::Heading {
                level: 2,
                content: "Hello world".to_string(),
            },
            MarkdownElements::NewLine,
            MarkdownElements::NewLine,
            MarkdownElements::Paragraph {
                content: "This is a paragraph.".to_string(),
            },
            MarkdownElements::NewLine,
            MarkdownElements::NewLine,
            MarkdownElements::Heading {
                level: 2,
                content: "Another heading".to_string(),
            },
            MarkdownElements::NewLine,
            MarkdownElements::NewLine,
            MarkdownElements::Paragraph {
                content: "This is another paragraph.".to_string(),
            }
        ]);
    }
}
