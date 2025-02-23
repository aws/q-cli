use std::borrow::Cow;
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::LazyLock;

use crossterm::queue;
use crossterm::style::{
    self,
    Color,
};
use eyre::{
    ContextCompat,
    Result,
    bail,
    eyre,
};
use fig_os_shim::Context;
use serde::Deserialize;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{
    LinesWithEndings,
    as_24_bit_terminal_escaped,
};
use tokio::io::AsyncWriteExt;
use tracing::error;

use super::InvokeOutput;
use crate::cli::chat::tools::absolute_to_relative;

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

#[derive(Debug, Deserialize)]
#[serde(tag = "command")]
pub enum FsWrite {
    #[serde(rename = "create")]
    Create { path: String, file_text: String },
    #[serde(rename = "str_replace")]
    StrReplace {
        path: String,
        old_str: String,
        new_str: String,
    },
    #[serde(rename = "insert")]
    Insert {
        path: String,
        insert_line: usize,
        new_str: String,
    },
}

impl FsWrite {
    pub async fn invoke(&self, ctx: &Context, updates: &mut impl Write) -> Result<InvokeOutput> {
        let fs = ctx.fs();
        let cwd = ctx.env().current_dir()?;
        match self {
            FsWrite::Create { path, file_text } => {
                queue!(
                    updates,
                    style::SetForegroundColor(Color::Green),
                    style::Print(format!("Creating a new file at {}", format_path(cwd, path))),
                    style::ResetColor,
                    style::Print("\n"),
                )?;
                let mut file = fs.create_new(path).await?;
                file.write_all(file_text.as_bytes()).await?;
                file.sync_data().await?;
                Ok(Default::default())
            },
            FsWrite::StrReplace { path, old_str, new_str } => {
                let file = fs.read_to_string(&path).await?;
                let matches = file.match_indices(old_str).collect::<Vec<_>>();
                queue!(
                    updates,
                    style::SetForegroundColor(Color::Green),
                    style::Print(format!("Updating {}", format_path(cwd, path))),
                    style::ResetColor,
                    style::Print("\n"),
                )?;
                match matches.len() {
                    0 => Err(eyre!("no occurrences of old_str were found")),
                    1 => {
                        let file = file.replacen(old_str, new_str, 1);
                        fs.write(path, file).await?;
                        Ok(Default::default())
                    },
                    x => Err(eyre!("{x} occurrences of old_str were found when only 1 is expected")),
                }
            },
            FsWrite::Insert {
                path,
                insert_line,
                new_str,
            } => {
                queue!(
                    updates,
                    style::SetForegroundColor(Color::Green),
                    style::Print(format!(
                        "Inserting at line {} in {}",
                        insert_line,
                        format_path(cwd, path)
                    )),
                    style::ResetColor,
                    style::Print("\n"),
                )?;
                let path = fs.chroot_path_str(path);
                let mut file = fs.read_to_string(&path).await?;

                // Get the index of the start of the line to insert at.
                let num_lines = file.lines().enumerate().map(|(i, _)| i + 1).last().unwrap_or(1);
                let insert_line = insert_line.clamp(&0, &num_lines);
                let mut i = 0;
                for _ in 0..*insert_line {
                    let line_len = &file[i..].find("\n").map_or(file[i..].len(), |i| i + 1);
                    i += line_len;
                }
                file.insert_str(i, new_str);
                fs.write(&path, &file).await?;
                Ok(Default::default())
            },
        }
    }

    pub fn queue_description(&self, ctx: &Context, updates: &mut impl Write) -> Result<()> {
        let cwd = ctx.env().current_dir()?;
        match self {
            FsWrite::Create { path, file_text } => {
                let path = format_path(cwd, path);
                let file = stylize_output_if_able(&path, file_text, None, None);
                queue!(
                    updates,
                    style::Print("Path: "),
                    style::SetForegroundColor(Color::Green),
                    style::Print(path),
                    style::ResetColor,
                    style::Print("\n\n"),
                )?;
                queue!(updates, style::Print(file), style::ResetColor)?;
                Ok(())
            },
            FsWrite::Insert {
                path,
                insert_line,
                new_str,
            } => {
                let path = format_path(cwd, path);
                let file = stylize_output_if_able(&path, new_str, Some(*insert_line), Some("+"));
                queue!(
                    updates,
                    style::Print("Path: "),
                    style::SetForegroundColor(Color::Green),
                    style::Print(path),
                    style::ResetColor,
                    style::Print("\n\n"),
                )?;
                queue!(updates, style::Print(file), style::ResetColor)?;
                Ok(())
            },
            FsWrite::StrReplace { path, old_str, new_str } => {
                let path = format_path(cwd, path);
                let file = ctx.fs().read_to_string_sync(&path)?;
                // TODO: we should pass some additional lines as context before and after the file.
                let (start_line, _) = match line_number_at(&file, old_str) {
                    Some((start_line, end_line)) => (Some(start_line), Some(end_line)),
                    _ => (None, None),
                };
                let old_str = stylize_output_if_able(&path, old_str, start_line, Some("-"));
                let new_str = stylize_output_if_able(&path, new_str, start_line, Some("+"));
                queue!(
                    updates,
                    style::Print("Path: "),
                    style::SetForegroundColor(Color::Green),
                    style::Print(path),
                    style::ResetColor,
                    style::Print("\n\n"),
                )?;
                queue!(
                    updates,
                    style::SetAttribute(style::Attribute::Bold),
                    style::Print("Replacing:\n"),
                    style::SetAttribute(style::Attribute::Reset),
                )?;
                queue!(updates, style::Print(old_str), style::ResetColor)?;
                queue!(updates, style::Print("\n\n"))?;
                queue!(
                    updates,
                    style::SetAttribute(style::Attribute::Bold),
                    style::Print("With:\n"),
                    style::SetAttribute(style::Attribute::Reset),
                )?;
                queue!(updates, style::Print(new_str), style::ResetColor)?;
                Ok(())
            },
        }
    }

    pub async fn validate(&mut self, _ctx: &Context) -> Result<()> {
        match self {
            FsWrite::Create { path, .. } => {
                if path.is_empty() {
                    bail!("Path must not be empty")
                };
            },
            FsWrite::StrReplace { path, .. } | FsWrite::Insert { path, .. } => {
                if !PathBuf::from(&path).exists() {
                    bail!("The provided path must exist in order to replace or insert contents into it")
                }
            },
        }

        Ok(())
    }
}

/// Small helper for formatting the path in [FsWrite] output.
fn format_path(cwd: impl AsRef<Path>, path: impl AsRef<Path>) -> String {
    absolute_to_relative(cwd, path.as_ref())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or(path.as_ref().to_string_lossy().to_string())
}

/// Limits the passed str to `max_len`.
///
/// If the str exceeds `max_len`, then the first `max_len` characters are returned with a suffix of
/// `"<...Truncated>`. Otherwise, the str is returned as is.
#[allow(dead_code)]
fn truncate_str(text: &str, max_len: usize) -> Cow<'_, str> {
    if text.len() > max_len {
        let mut out = String::new();
        let t = "<...Truncated>";
        out.push_str(&text[..max_len]);
        out.push_str(t);
        out.into()
    } else {
        text.into()
    }
}

/// Returns the number of characters required for displaying line numbers for `file_text`.
fn terminal_width(line_count: usize) -> usize {
    ((line_count as f32 + 0.1).log10().ceil()) as usize
}

fn stylize_output_if_able<'a>(
    path: impl AsRef<Path>,
    file_text: &'a str,
    starting_line: Option<usize>,
    gutter_prefix: Option<&str>,
) -> Cow<'a, str> {
    match stylized_file(path, file_text, starting_line, gutter_prefix) {
        Ok(s) => s.into(),
        Err(err) => {
            error!(?err, "unable to syntax highlight the output");
            file_text.into()
        },
    }
}

/// Returns a 24bit terminal escaped syntax-highlighted [String] of the file pointed to by `path`,
/// if able.
///
/// Params:
/// - `starting_line` - 1-indexed line to start the line number at.
/// - `gutter_prefix` - character to display in the first cell of the gutter, before the file
///   number.
fn stylized_file(
    path: impl AsRef<Path>,
    file_text: impl AsRef<str>,
    starting_line: Option<usize>,
    gutter_prefix: Option<&str>,
) -> Result<String> {
    let starting_line = starting_line.unwrap_or(1);
    let gutter_prefix = gutter_prefix.unwrap_or(" ");

    let ps = &*SYNTAX_SET;
    let ts = &*THEME_SET;

    let extension = path
        .as_ref()
        .extension()
        .wrap_err("missing extension")?
        .to_str()
        .wrap_err("not utf8")?;

    let syntax = ps
        .find_syntax_by_extension(extension)
        .wrap_err_with(|| format!("missing extension: {}", extension))?;

    let theme = &ts.themes["base16-ocean.dark"];
    let mut h = HighlightLines::new(syntax, theme);
    let gutter_width = terminal_width(file_text.as_ref().lines().count()) + terminal_width(starting_line);
    let file_text = LinesWithEndings::from(file_text.as_ref());
    let (gutter_fg, gutter_bg) = match (
        theme.settings.gutter_foreground,
        theme.settings.gutter,
        theme.settings.foreground,
        theme.settings.background,
    ) {
        (Some(gutter_fg), Some(gutter_bg), _, _) => (gutter_fg, gutter_bg),
        (_, _, Some(fg), Some(bg)) => (fg, bg),
        _ => bail!("missing theme"),
    };
    let gutter_prefix_style = syntect::highlighting::Style {
        foreground: gutter_fg,
        background: gutter_bg,
        font_style: syntect::highlighting::FontStyle::BOLD,
    };
    let gutter_linenum_style = syntect::highlighting::Style {
        foreground: gutter_fg,
        background: gutter_bg,
        font_style: syntect::highlighting::FontStyle::default(),
    };

    let mut file = String::new();
    file.push_str(&as_24_bit_terminal_escaped(&[(gutter_linenum_style, "\n\n")], true));
    for (i, line) in file_text.enumerate() {
        let i = (i + starting_line).to_string();
        let gutter_content = format!("{:>width$} ", i, width = gutter_width);
        let mut ranges = vec![
            (gutter_prefix_style, gutter_prefix),
            (gutter_linenum_style, gutter_content.as_str()),
        ];
        ranges.append(&mut h.highlight_line(line, ps)?);
        let escaped_line = as_24_bit_terminal_escaped(&ranges[..], true);
        file.push_str(&escaped_line);
    }

    Ok(file)
}

/// Returns a 1-indexed line number range of the start and end of `needle` inside `file`.
fn line_number_at(file: impl AsRef<str>, needle: impl AsRef<str>) -> Option<(usize, usize)> {
    let file = file.as_ref();
    let needle = needle.as_ref();
    if let Some((i, _)) = file.match_indices(needle).next() {
        let start = file[..i].matches("\n").count();
        let end = needle.matches("\n").count();
        Some((start + 1, start + end + 1))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    const TEST_FILE_CONTENTS: &str = "\
1: Hello world!
2: This is line 2
3: asdf
4: Hello world!
";

    const TEST_FILE_PATH: &str = "/test_file.txt";
    const TEST_HIDDEN_FILE_PATH: &str = "/aaaa2/.hidden";

    /// Sets up the following filesystem structure:
    /// ```text
    /// test_file.txt
    /// /home/testuser/
    /// /aaaa1/
    ///     /bbbb1/
    ///         /cccc1/
    /// /aaaa2/
    ///     .hidden
    /// ```
    async fn setup_test_directory() -> Arc<Context> {
        let ctx = Context::builder().with_test_home().await.unwrap().build_fake();
        let fs = ctx.fs();
        fs.write(TEST_FILE_PATH, TEST_FILE_CONTENTS).await.unwrap();
        fs.create_dir_all("/aaaa1/bbbb1/cccc1").await.unwrap();
        fs.create_dir_all("/aaaa2").await.unwrap();
        fs.write(TEST_HIDDEN_FILE_PATH, "this is a hidden file").await.unwrap();
        ctx
    }

    #[test]
    fn test_fs_write_deserialize() {
        let path = "/my-file";
        let file_text = "hello world";

        // create
        let v = serde_json::json!({
            "path": path,
            "command": "create",
            "file_text": file_text
        });
        let fw = serde_json::from_value::<FsWrite>(v).unwrap();
        assert!(matches!(fw, FsWrite::Create { .. }));

        // str_replace
        let v = serde_json::json!({
            "path": path,
            "command": "str_replace",
            "old_str": "prev string",
            "new_str": "new string",
        });
        let fw = serde_json::from_value::<FsWrite>(v).unwrap();
        assert!(matches!(fw, FsWrite::StrReplace { .. }));

        // insert
        let v = serde_json::json!({
            "path": path,
            "command": "insert",
            "insert_line": 3,
            "new_str": "new string",
        });
        let fw = serde_json::from_value::<FsWrite>(v).unwrap();
        assert!(matches!(fw, FsWrite::Insert { .. }));
    }

    #[tokio::test]
    async fn test_fs_write_tool_create() {
        let ctx = setup_test_directory().await;
        let mut stdout = std::io::stdout();

        let file_text = "Hello, world!";
        let v = serde_json::json!({
            "path": "/my-file",
            "command": "create",
            "file_text": file_text
        });
        serde_json::from_value::<FsWrite>(v)
            .unwrap()
            .invoke(&ctx, &mut stdout)
            .await
            .unwrap();

        assert_eq!(ctx.fs().read_to_string("/my-file").await.unwrap(), file_text);
    }

    #[tokio::test]
    async fn test_fs_write_tool_str_replace() {
        let ctx = setup_test_directory().await;
        let mut stdout = std::io::stdout();

        // No instances found
        let v = serde_json::json!({
            "path": TEST_FILE_PATH,
            "command": "str_replace",
            "old_str": "asjidfopjaieopr",
            "new_str": "1623749",
        });
        assert!(
            serde_json::from_value::<FsWrite>(v)
                .unwrap()
                .invoke(&ctx, &mut stdout)
                .await
                .is_err()
        );

        // Multiple instances found
        let v = serde_json::json!({
            "path": TEST_FILE_PATH,
            "command": "str_replace",
            "old_str": "Hello world!",
            "new_str": "Goodbye world!",
        });
        assert!(
            serde_json::from_value::<FsWrite>(v)
                .unwrap()
                .invoke(&ctx, &mut stdout)
                .await
                .is_err()
        );

        // Single instance found and replaced
        let v = serde_json::json!({
            "path": TEST_FILE_PATH,
            "command": "str_replace",
            "old_str": "1: Hello world!",
            "new_str": "1: Goodbye world!",
        });
        serde_json::from_value::<FsWrite>(v)
            .unwrap()
            .invoke(&ctx, &mut stdout)
            .await
            .unwrap();
        assert_eq!(
            ctx.fs()
                .read_to_string(TEST_FILE_PATH)
                .await
                .unwrap()
                .lines()
                .next()
                .unwrap(),
            "1: Goodbye world!",
            "expected the only occurrence to be replaced"
        );
    }

    #[tokio::test]
    async fn test_fs_write_tool_insert_at_beginning() {
        let ctx = setup_test_directory().await;
        let mut stdout = std::io::stdout();

        let new_str = "1: New first line!\n";
        let v = serde_json::json!({
            "path": TEST_FILE_PATH,
            "command": "insert",
            "insert_line": 0,
            "new_str": new_str,
        });
        serde_json::from_value::<FsWrite>(v)
            .unwrap()
            .invoke(&ctx, &mut stdout)
            .await
            .unwrap();
        let actual = ctx.fs().read_to_string(TEST_FILE_PATH).await.unwrap();
        assert_eq!(
            format!("{}\n", actual.lines().next().unwrap()),
            new_str,
            "expected the first line to be updated to '{}'",
            new_str
        );
        assert_eq!(
            actual.lines().skip(1).collect::<Vec<_>>(),
            TEST_FILE_CONTENTS.lines().collect::<Vec<_>>(),
            "the rest of the file should not have been updated"
        );
    }

    #[tokio::test]
    async fn test_fs_write_tool_insert_after_first_line() {
        let ctx = setup_test_directory().await;
        let mut stdout = std::io::stdout();

        let new_str = "2: New second line!\n";
        let v = serde_json::json!({
            "path": TEST_FILE_PATH,
            "command": "insert",
            "insert_line": 1,
            "new_str": new_str,
        });

        serde_json::from_value::<FsWrite>(v)
            .unwrap()
            .invoke(&ctx, &mut stdout)
            .await
            .unwrap();
        let actual = ctx.fs().read_to_string(TEST_FILE_PATH).await.unwrap();
        assert_eq!(
            format!("{}\n", actual.lines().nth(1).unwrap()),
            new_str,
            "expected the second line to be updated to '{}'",
            new_str
        );
        assert_eq!(
            actual.lines().skip(2).collect::<Vec<_>>(),
            TEST_FILE_CONTENTS.lines().skip(1).collect::<Vec<_>>(),
            "the rest of the file should not have been updated"
        );
    }

    #[tokio::test]
    async fn test_fs_write_tool_insert_when_no_newlines_in_file() {
        let ctx = Context::builder().with_test_home().await.unwrap().build_fake();
        let mut stdout = std::io::stdout();

        let test_file_path = "/file.txt";
        let test_file_contents = "hello there";
        ctx.fs().write(test_file_path, test_file_contents).await.unwrap();

        let new_str = "test";

        // First, test appending
        let v = serde_json::json!({
            "path": test_file_path,
            "command": "insert",
            "insert_line": 1,
            "new_str": new_str,
        });
        serde_json::from_value::<FsWrite>(v)
            .unwrap()
            .invoke(&ctx, &mut stdout)
            .await
            .unwrap();
        let actual = ctx.fs().read_to_string(test_file_path).await.unwrap();
        assert_eq!(actual, format!("{}{}", test_file_contents, new_str),);

        // Then, test prepending
        let v = serde_json::json!({
            "path": test_file_path,
            "command": "insert",
            "insert_line": 0,
            "new_str": new_str,
        });
        serde_json::from_value::<FsWrite>(v)
            .unwrap()
            .invoke(&ctx, &mut stdout)
            .await
            .unwrap();
        let actual = ctx.fs().read_to_string(test_file_path).await.unwrap();
        assert_eq!(actual, format!("{}{}{}", new_str, test_file_contents, new_str),);
    }

    #[test]
    fn test_truncate_str() {
        let s = "Hello, world!";
        assert_eq!(truncate_str(s, 6), "Hello,<...Truncated>");
        let s = "Hello, world!";
        assert_eq!(truncate_str(s, 13), s);
        let s = "Hello, world!";
        assert_eq!(truncate_str(s, 0), "<...Truncated>");
    }

    #[test]
    fn test_gutter_width() {
        assert_eq!(terminal_width(1), 1);
        assert_eq!(terminal_width(9), 1);
        assert_eq!(terminal_width(10), 2);
        assert_eq!(terminal_width(99), 2);
        assert_eq!(terminal_width(100), 3);
        assert_eq!(terminal_width(999), 3);
    }
}
