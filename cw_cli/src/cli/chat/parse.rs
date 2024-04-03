use std::io::Write;

use crossterm::style::{
    Attribute,
    Color,
    Stylize,
};
use crossterm::{
    style,
    Command,
};
use unicode_width::{
    UnicodeWidthChar,
    UnicodeWidthStr,
};
use winnow::ascii::{
    self,
    alphanumeric1,
    digit1,
    space0,
    space1,
    till_line_ending,
};
use winnow::combinator::{
    alt,
    delimited,
    preceded,
    repeat,
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

#[derive(Debug)]
pub struct ParseState {
    pub terminal_width: usize,
    pub column: usize,
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub set_newline: bool,
    pub newline: bool,
}

impl ParseState {
    pub fn new(terminal_width: usize) -> Self {
        Self {
            terminal_width,
            column: 0,
            bold: false,
            italic: false,
            strikethrough: false,
            set_newline: false,
            newline: true,
        }
    }
}

pub fn interpret_markdown<'a, 'b>(
    mut i: Partial<&'a str>,
    mut o: impl Write + 'b,
    state: &mut ParseState,
) -> PResult<Partial<&'a str>, Error<'a>> {
    let mut error: Option<Error<'_>> = None;
    let start = i.checkpoint();

    macro_rules! alt {
        ($($fns:ident),*) => {
            $({
                i.reset(&start);
                match $fns(&mut o, state).parse_next(&mut i) {
                    Err(ErrMode::Backtrack(e)) => {
                        error = match error {
                            Some(error) => Some(error.or(e)),
                            None => Some(e),
                        };
                    },
                    res => {
                        return res.map(|_| i);
                    }
                }
            })*
        };
    }

    alt!(
        // This pattern acts as a short circuit for alphanumeric plaintext
        // More importantly, it's needed to support manual wordwrapping
        text,
        // multiline patterns
        blockquote,
        // linted_codeblock,
        codeblock,
        // single line patterns
        horizontal_rule,
        heading,
        bulleted_item,
        numbered_item,
        // inline patterns
        code,
        url,
        bold,
        italic,
        strikethrough,
        // symbols
        less_than,
        greater_than,
        ampersand,
        line_ending,
        // fallback
        fallback
    );

    match error {
        Some(e) => Err(ErrMode::Backtrack(e.append(&i, &start, ErrorKind::Alt))),
        None => Err(ErrMode::assert(&i, "no parsers")),
    }
}

fn text<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        let content = alphanumeric1.parse_next(i)?;
        queue_newline_or_advance(&mut o, state, content.width())?;
        queue(&mut o, style::Print(content))?;
        Ok(())
    }
}

fn heading<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        if !state.newline {
            return Err(ErrMode::from_error_kind(i, ErrorKind::Fail));
        }

        let level = terminated(take_while(1.., |c| c == '#'), space1).parse_next(i)?;
        let print = format!("{level} ");

        queue_newline_or_advance(&mut o, state, print.width())?;
        queue(&mut o, style::SetForegroundColor(Color::Magenta))?;
        queue(&mut o, style::SetAttribute(Attribute::Bold))?;
        queue(&mut o, style::Print(print))
    }
}

fn bulleted_item<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        if !state.newline {
            return Err(ErrMode::from_error_kind(i, ErrorKind::Fail));
        }

        let ws = (space0, "-", space1).parse_next(i)?.0;
        let print = format!("{ws}• ");

        queue_newline_or_advance(&mut o, state, print.width())?;
        queue(&mut o, style::ResetColor)?;
        queue(&mut o, style::SetAttribute(style::Attribute::Reset))?;
        queue(&mut o, style::Print(print))
    }
}

fn numbered_item<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        if !state.newline {
            return Err(ErrMode::from_error_kind(i, ErrorKind::Fail));
        }

        let (ws, digits, _, _) = (space0, digit1, ".", space1).parse_next(i)?;
        let print = format!("{ws}{digits}. ");

        queue_newline_or_advance(&mut o, state, print.width())?;
        queue(&mut o, style::ResetColor)?;
        queue(&mut o, style::SetAttribute(style::Attribute::Reset))?;
        queue(&mut o, style::Print(print))
    }
}

fn horizontal_rule<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        if !state.newline {
            return Err(ErrMode::from_error_kind(i, ErrorKind::Fail));
        }

        (
            space0,
            alt((take_while(3.., '-'), take_while(3.., '*'), take_while(3.., '_'))),
            space0,
        )
            .parse_next(i)?;

        state.column = 0;
        state.set_newline = true;

        queue(&mut o, style::ResetColor)?;
        queue(&mut o, style::SetAttribute(style::Attribute::Reset))?;
        queue(&mut o, style::Print(format!("{}\n", "━".repeat(state.terminal_width))))
    }
}

fn code<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        "`".parse_next(i)?;
        let code = terminated(take_until(0.., "`"), "`").parse_next(i)?;

        queue_newline_or_advance(&mut o, state, code.width())?;
        queue(&mut o, style::Print(code.green()))?;

        Ok(())
    }
}

fn blockquote<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        if !state.newline {
            return Err(ErrMode::from_error_kind(i, ErrorKind::Fail));
        }

        let level = repeat::<_, _, Vec<&'_ str>, _, _>(1.., terminated("&gt;", space0))
            .parse_next(i)?
            .len();
        let print = "│ ".repeat(level);

        queue(&mut o, style::SetForegroundColor(Color::Grey))?;
        queue_newline_or_advance(&mut o, state, print.width())?;
        queue(&mut o, style::Print(print))
    }
}

fn codeblock<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        if !state.newline {
            return Err(ErrMode::from_error_kind(i, ErrorKind::Fail));
        }

        // We don't want to do anything special to text inside codeblocks so we wait for all of it
        // The alternative is to switch between parse rules at the top level but that's slightly involved
        let language = preceded("```", till_line_ending).parse_next(i)?;
        let code = terminated(take_until(0.., "```"), "```").parse_next(i)?;
        ascii::line_ending.parse_next(i)?;

        queue(&mut o, style::ResetColor)?;
        queue(&mut o, style::SetAttribute(Attribute::Reset))?;

        // Do some simple replacements so it's slightly readable
        let out = code
            .trim()
            .replace("&amp;", "&")
            .replace("&gt;", ">")
            .replace("&lt;", "<");

        if !language.is_empty() && out.lines().count() > 1 {
            queue(&mut o, style::Print(format!("{}\n", language).bold()))?;
        }

        for line in out.lines() {
            queue(&mut o, style::Print(format!("{line}\n").green()))?;
        }

        state.column = 0;

        Ok(())
    }
}

fn bold<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        alt(("**", "__")).parse_next(i)?;
        state.bold = !state.bold;
        match state.bold {
            true => queue(&mut o, style::SetAttribute(Attribute::Bold)),
            false => queue(&mut o, style::SetAttribute(Attribute::NormalIntensity)),
        }
    }
}

fn italic<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        alt(("*", "_")).parse_next(i)?;
        state.italic = !state.italic;
        match state.italic {
            true => queue(&mut o, style::SetAttribute(Attribute::Italic)),
            false => queue(&mut o, style::SetAttribute(Attribute::NoItalic)),
        }
    }
}

fn strikethrough<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        "~~".parse_next(i)?;
        state.strikethrough = !state.strikethrough;
        match state.strikethrough {
            true => queue(&mut o, style::SetAttribute(Attribute::CrossedOut)),
            false => queue(&mut o, style::SetAttribute(Attribute::NotCrossedOut)),
        }
    }
}

fn url<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        let display = delimited("[", take_until(1.., "]("), "]").parse_next(i)?;
        let link = delimited("(", take_till(0.., ')'), ")").parse_next(i)?;

        queue_newline_or_advance(&mut o, state, display.width() + 1)?;
        queue(&mut o, style::SetForegroundColor(Color::Blue))?;
        queue(&mut o, style::Print(format!("{display} ")))?;
        queue(&mut o, style::SetForegroundColor(Color::DarkGrey))?;
        state.column += link.width();
        queue(&mut o, style::Print(link))?;
        queue(&mut o, style::SetForegroundColor(Color::Reset))
    }
}

fn less_than<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        "&lt;".parse_next(i)?;
        queue_newline_or_advance(&mut o, state, 1)?;
        queue(&mut o, style::Print('<'))
    }
}

fn greater_than<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        "&gt;".parse_next(i)?;
        queue_newline_or_advance(&mut o, state, 1)?;
        queue(&mut o, style::Print('>'))
    }
}

fn ampersand<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        "&amp;".parse_next(i)?;
        queue_newline_or_advance(&mut o, state, 1)?;
        queue(&mut o, style::Print('&'))
    }
}

fn line_ending<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        ascii::line_ending.parse_next(i)?;

        state.column = 0;
        state.set_newline = true;

        queue(&mut o, style::ResetColor)?;
        queue(&mut o, style::SetAttribute(style::Attribute::Reset))?;
        queue(&mut o, style::Print("\n"))
    }
}

fn fallback<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
) -> impl FnMut(&mut Partial<&'a str>) -> PResult<(), Error<'a>> + 'b {
    move |i| {
        let fallback = any.parse_next(i)?;
        if let Some(width) = fallback.width() {
            queue_newline_or_advance(&mut o, state, width)?;
            if fallback != ' ' || state.column != 1 {
                queue(&mut o, style::Print(fallback))?;
            }
        }

        Ok(())
    }
}

fn queue_newline_or_advance<'a, 'b>(
    mut o: impl Write + 'b,
    state: &'b mut ParseState,
    width: usize,
) -> Result<(), ErrMode<Error<'a>>> {
    if state.column > 0 && state.column + width > state.terminal_width {
        state.column = width;
        queue(&mut o, style::Print('\n'))?;
    } else {
        state.column += width;
    }

    Ok(())
}

fn queue<'a>(o: &mut impl Write, command: impl Command) -> Result<(), ErrMode<Error<'a>>> {
    use crossterm::QueueableCommand;
    o.queue(command).map_err(|err| ErrMode::Cut(Error::Stdio(err)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use winnow::stream::Offset;

    use super::*;

    fn assert_parse_eq(input: &'static str, output: &'static str) {
        let mut state = ParseState::new(1024);
        let mut result = vec![];
        let mut offset = 0;

        loop {
            let input = Partial::new(&input[offset..]);
            match interpret_markdown(input, &mut result, &mut state) {
                Ok(parsed) => {
                    offset += parsed.offset_from(&input);
                    state.newline = state.set_newline;
                    state.set_newline = false;
                },
                Err(err) => match err.into_inner() {
                    Some(err) => panic!("{err}"),
                    None => break, // Data was incomplete
                },
            }
        }

        result.flush().unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), output);
    }

    #[test]
    fn test_text() {
        assert_parse_eq("hello, world :)", "hello, world :)");
    }

    // TODO
    // blockquote,
    // codeblock,
    // horizontal_rule,
    // heading,
    // bulleted_item,
    // numbered_item,

    #[test]
    fn code() {
        assert_parse_eq("`print`", "\u{1b}[38;5;10mprint\u{1b}[39m");
    }

    #[test]
    fn url() {
        assert_parse_eq(
            "[[0]](google.com)",
            "\u{1b}[38;5;12m[0] \u{1b}[38;5;8mgoogle.com\u{1b}[39m",
        );
    }

    #[test]
    fn bold() {
        assert_parse_eq("**hello** ", "\u{1b}[1mhello\u{1b}[22m ");
    }

    #[test]
    fn italic() {
        assert_parse_eq("*hello* ", "\u{1b}[3mhello\u{1b}[23m ");
    }

    #[test]
    fn strikethrough() {
        assert_parse_eq("~~hello~~", "\u{1b}[9mhello\u{1b}[29m");
    }

    #[test]
    fn less_than() {
        assert_parse_eq("&lt;", "<");
    }

    #[test]
    fn greater_than() {
        assert_parse_eq("1 &gt; 2 ", "1 > 2 ");
    }

    #[test]
    fn ampersand() {
        assert_parse_eq("&amp;", "&");
    }

    #[test]
    fn line_ending() {
        assert_parse_eq(".\n.", ".\u{1b}[0m\u{1b}[0m\n.");
    }

    #[test]
    fn test_fallback() {
        assert_parse_eq("+ % @ . ?", "+ % @ . ?");
    }
}
