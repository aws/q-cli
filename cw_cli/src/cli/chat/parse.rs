use std::io::{
    Stderr,
    Write,
};

use crossterm::style::{
    Attribute,
    Color,
    Colors,
    Print,
    Stylize,
};
use crossterm::{
    style,
    Command,
    QueueableCommand,
};
use winnow::ascii::{
    self,
    digit1,
    multispace1,
    till_line_ending,
};
use winnow::combinator::{
    alt,
    delimited,
    preceded,
    terminated,
};
use winnow::error::{
    ErrMode,
    ErrorKind,
    ParserError,
};
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::{
    any,
    take_till,
    take_until,
    take_while,
};
use winnow::Partial;

#[derive(Debug, thiserror::Error)]
pub enum Error<'a> {
    #[error(transparent)]
    Stdio(#[from] std::io::Error),
    #[error("parse error {1}, input {0}")]
    Winnow(Partial<&'a str>, ErrorKind),
}

impl<'a> ParserError<Partial<&'a str>> for Error<'a> {
    fn from_error_kind(input: &Partial<&'a str>, kind: ErrorKind) -> Self {
        Self::Winnow(*input, kind)
    }

    fn append(
        self,
        _input: &Partial<&'a str>,
        _checkpoint: &winnow::stream::Checkpoint<
            winnow::stream::Checkpoint<&'a str, &'a str>,
            winnow::Partial<&'a str>,
        >,
        _kind: ErrorKind,
    ) -> Self {
        self
    }
}

#[derive(Debug, Default)]
pub struct ParseState {
    pub in_quote: bool,
}

pub fn interpret_markdown<'a>(
    mut i: Partial<&'a str>,
    o: &Stderr,
    state: &mut ParseState,
) -> PResult<Partial<&'a str>, Error<'a>> {
    let mut error: Option<Error<'_>> = None;
    let start = i.checkpoint();

    macro_rules! alt {
        ($($fns:ident),*) => {
            $({
                i.reset(&start);
                match $fns(o, state).parse_next(&mut i) {
                    Err(ErrMode::Backtrack(e)) => {
                        error = match error {
                            Some(error) => Some(error.or(e)),
                            None => Some(e),
                        };
                    },
                    res => return res.map(|_| i),
                }
            })*
        };
    }

    alt!(
        // multiline patterns
        blockquote,
        // linted_codeblock,
        codeblock,
        // single line patterns
        heading,
        bulleted_item,
        numbered_item,
        // inline patterns
        code,
        url,
        bold,
        italic,
        // symbols
        less_than,
        greater_than,
        ampersand,
        line_ending,
        // fallback
        text
    );

    match error {
        Some(e) => Err(ErrMode::Backtrack(e.append(&i, &start, ErrorKind::Alt))),
        None => Err(ErrMode::assert(&i, "no parsers")),
    }
}

fn heading<'a, 'b>(
    mut o: impl Write,
    _state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> {
    move |i| {
        let level = terminated(take_while(1.., |c| c == '#'), multispace1).parse_next(i)?;

        o.queue(style::PrintStyledContent(
            format!("{} ", "#".repeat(level.len()))
                .with(Color::Magenta)
                .attribute(Attribute::Bold),
        ))
        .map_err(cut)?;

        Ok(())
    }
}

fn bulleted_item<'a, 'b>(
    mut o: impl Write,
    _state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> {
    move |i| {
        ("-", multispace1).parse_next(i)?;
        queue(&mut o, style::ResetColor)?;
        queue(&mut o, style::SetAttribute(style::Attribute::Reset))?;
        queue(&mut o, Print("• "))
    }
}

fn numbered_item<'a, 'b>(
    mut o: impl Write,
    _state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> {
    move |i| {
        let (digits, _, _) = (digit1, ".", multispace1).parse_next(i)?;
        queue(&mut o, style::ResetColor)?;
        queue(&mut o, style::SetAttribute(style::Attribute::Reset))?;
        queue(&mut o, Print(format!("{digits}. ")))
    }
}

fn blockquote<'a, 'b>(
    mut o: impl Write,
    _state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> {
    move |i| {
        ("&gt;", multispace1).parse_next(i)?;
        o.queue(Print("> ".with(Color::Grey))).map_err(cut)?;
        Ok(())
    }
}

fn code<'a, 'b>(
    mut o: impl Write,
    _state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> {
    move |i| {
        let content: &str = ("`", take_until(0.., '`'), "`").parse_next(i)?.1;
        queue(&mut o, style::SetColors(Colors::new(Color::Green, Color::Reset)))?;
        queue(&mut o, Print(content))?;
        queue(&mut o, style::ResetColor)
    }
}

fn codeblock<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        "```".parse_next(i)?;

        queue(&mut o, Print("```"))?;

        state.in_quote = !state.in_quote;
        // if state.in_quote {
        //     queue(&mut o, style::Print("│ "))?;
        // }

        Ok(())
    }
}

fn bold<'a, 'b>(
    mut o: impl Write,
    _state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> {
    move |i: &mut Partial<&str>| {
        let content = alt((
            delimited("**", take_until(0.., "**"), "**"),
            preceded("**", till_line_ending),
        ))
        .parse_next(i)?;

        queue(&mut o, style::SetAttribute(Attribute::Bold))?;
        queue(&mut o, Print(content))
    }
}

fn italic<'a, 'b>(
    mut o: impl Write,
    _state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> {
    move |i| {
        let content = alt((
            delimited("*", take_until(1.., "_"), "*"),
            preceded("*", till_line_ending),
            delimited("_", take_until(1.., "_"), "_"),
            preceded("_", till_line_ending),
        ))
        .parse_next(i)?;
        queue(&mut o, style::SetAttribute(Attribute::Italic))?;
        queue(&mut o, Print(content))
    }
}

fn url<'a, 'b>(
    mut o: impl Write,
    _state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> {
    move |i| {
        let display = ("[", take_till(0.., ']'), "]").parse_next(i)?.1;
        let link = ("(", take_till(0.., ')'), ")").parse_next(i)?.1;

        queue(&mut o, style::SetForegroundColor(Color::Blue))?;
        queue(&mut o, Print(format!("{display} ")))?;
        queue(&mut o, style::SetForegroundColor(Color::DarkGrey))?;
        queue(&mut o, Print(link))?;
        queue(&mut o, style::SetForegroundColor(Color::Reset))
    }
}

fn less_than<'a, 'b>(
    mut o: impl Write,
    _state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> {
    move |i| {
        "&lt;".parse_next(i)?;
        queue(&mut o, Print('<'))
    }
}

fn greater_than<'a, 'b>(
    mut o: impl Write,
    _state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> {
    move |i| {
        "&gt;".parse_next(i)?;
        queue(&mut o, Print('>'))
    }
}

fn ampersand<'a, 'b>(
    mut o: impl Write,
    _state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> {
    move |i| {
        "&amp;".parse_next(i)?;
        queue(&mut o, Print('&'))
    }
}

fn line_ending<'a, 'b>(
    mut o: impl Write + 'b,
    _state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        ascii::line_ending.parse_next(i)?;

        queue(&mut o, style::ResetColor)?;
        queue(&mut o, style::SetAttribute(style::Attribute::Reset))?;
        queue(&mut o, Print("\n"))
    }
}

fn text<'a, 'b>(
    mut o: impl Write,
    _state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> {
    move |i| {
        let content = any.parse_next(i)?;
        queue(&mut o, Print(content))
    }
}

fn cut<'a>(err: std::io::Error) -> ErrMode<Error<'a>> {
    ErrMode::Cut(Error::Stdio(err))
}

fn queue<'a>(o: &mut impl Write, command: impl Command) -> Result<(), ErrMode<Error<'a>>> {
    o.queue(command).map_err(|err| ErrMode::Cut(Error::Stdio(err)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::{
        stderr,
        Write,
    };
    use std::time::Duration;

    use unicode_segmentation::UnicodeSegmentation;
    use winnow::stream::Offset as _;

    use super::*;

    #[test]
    #[ignore]
    fn test_readme() -> eyre::Result<()> {
        let readme = include_str!("../../../../README.md");
        let stderr = stderr();
        let mut buf = String::new();
        let mut offset = 0;
        let mut parse_state = ParseState::default();
        for grapheme in readme.graphemes(true) {
            buf.push_str(grapheme);

            let input = Partial::new(&buf[offset..]);
            match interpret_markdown(input, &stderr, &mut parse_state) {
                Ok(parsed) => {
                    offset += parsed.offset_from(&input);
                    stderr.lock().flush()?;
                },
                Err(ErrMode::Incomplete(_)) => {
                    continue;
                },
                _ => panic!(),
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        Ok(())
    }
}
