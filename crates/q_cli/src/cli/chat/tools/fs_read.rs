use std::collections::VecDeque;
use std::fmt::Display;
use std::fs::Metadata;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use async_trait::async_trait;
use eyre::Result;
use fig_os_shim::Context;
use serde::Deserialize;
use tracing::warn;

use super::{Error, InvokeOutput, OutputKind, Tool};

#[derive(Debug, Deserialize)]
pub struct FsRead {
    pub path: String,
    pub read_range: Option<Vec<i32>>,
    #[serde(skip)]
    pub ty: Option<bool>,
}

enum FsReadType {
    File,
    Dir,
}

impl FsRead {
    pub fn read_range(&self) -> Result<Option<(i32, Option<i32>)>, Error> {
        match &self.read_range {
            Some(range) => match (range.first(), range.get(1)) {
                (Some(depth), None) => Ok(Some((*depth, None))),
                (Some(start), Some(end)) => Ok(Some((*start, Some(*end)))),
                other => Err(Error::Custom(format!("Invalid read range: {:?}", other).into())),
            },
            None => Ok(None),
        }
    }
}

#[async_trait]
impl Tool for FsRead {
    fn display_name(&self) -> String {
        "Read from filesystem".to_owned()
    }

    async fn invoke(&self, ctx: &Context) -> Result<InvokeOutput, Error> {
        // Required for testing scenarios: since the path is passed directly as a command argument,
        // we need to pass it through the Context first.
        let path = ctx.fs().chroot_path_str(&self.path);
        let is_file = ctx.fs().symlink_metadata(&self.path).await?.is_file();

        if is_file {
            // TODO: file size limit?
            let file = ctx.fs().read_to_string(&path).await?;

            if let Some((start, Some(end))) = self.read_range()? {
                let line_count = file.lines().count();

                // Convert negative 1-based indices to positive 0-based indices.
                let convert_index = |i: i32| -> usize {
                    if i <= 0 {
                        (line_count as i32 + i) as usize
                    } else {
                        i as usize - 1
                    }
                };
                let (start, end) = (convert_index(start), convert_index(end));
                if start > end {
                    return Ok(InvokeOutput {
                        output: OutputKind::Text(String::new()),
                    });
                }

                // The range should be inclusive on both ends.
                let f = file
                    .lines()
                    .skip(start)
                    .take(end - start + 1)
                    .collect::<Vec<_>>()
                    .join("\n");
                return Ok(InvokeOutput {
                    output: OutputKind::Text(f),
                });
            }
            return Ok(InvokeOutput {
                output: OutputKind::Text(file),
            });
        } else {
            let max_depth = self.read_range()?.map_or(0, |(d, _)| d);
            let mut result = Vec::new();
            let mut dir_queue = VecDeque::new();
            dir_queue.push_back((PathBuf::from(path), 0));
            while let Some((path, depth)) = dir_queue.pop_front() {
                if depth > max_depth {
                    break;
                }
                let mut read_dir = ctx.fs().read_dir(path).await?;
                while let Some(ent) = read_dir.next_entry().await? {
                    use std::os::unix::fs::MetadataExt;
                    let md = ent.metadata().await?;
                    let formatted_mode = format_mode(md.permissions().mode()).into_iter().collect::<String>();

                    let modified_timestamp = md.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();
                    let datetime = time::OffsetDateTime::from_unix_timestamp(modified_timestamp as i64).unwrap();
                    let formatted_date = datetime
                        .format(time::macros::format_description!(
                            "[month repr:short] [day] [hour]:[minute]"
                        ))
                        .unwrap();

                    // Mostly copying "The Long Format" from `man ls`.
                    // TODO: query user/group database to convert uid/gid to names?
                    result.push(format!(
                        "{}{} {} {} {} {} {} {}",
                        format_ftype(&md),
                        formatted_mode,
                        md.nlink(),
                        md.uid(),
                        md.gid(),
                        md.size(),
                        formatted_date,
                        ent.path().to_string_lossy()
                    ));
                    if md.is_dir() {
                        dir_queue.push_back((ent.path(), depth + 1));
                    }
                }
            }
            return Ok(InvokeOutput {
                output: OutputKind::Text(result.join("\n")),
            });
        }
    }

    async fn show_readable_intention(&self) -> Result<(), Error> {
        let is_file = self.ty.expect("Tool needs to have been validated");

        if is_file {
            crossterm::queue!(
                std::io::stdout(),
                crossterm::style::Print(format!("Reading file: {}, ", self.path))
            )?;

            if let Some(ref read_range) = self.read_range {
                let start = read_range.first();
                let end = read_range.get(1);

                match (start, end) {
                    (Some(start), Some(end)) => crossterm::queue!(
                        std::io::stdout(),
                        crossterm::style::Print(format!("from line {} to {}\n", start, end))
                    )?,
                    (Some(start), None) => {
                        let input = if *start > 0 {
                            format!("from line {} to end of file\n", start)
                        } else {
                            format!("{} line from the end of file to end of file", start)
                        };
                        crossterm::queue!(std::io::stdout(), crossterm::style::Print(input))?
                    },
                    _ => {
                        return Err(Error::Custom(std::borrow::Cow::Borrowed("Incorrect arguments passed")));
                    },
                }
            } else {
                return Err(Error::Custom(std::borrow::Cow::Borrowed("Incorrect arguments passed")));
            }
        } else {
            crossterm::queue!(
                std::io::stdout(),
                crossterm::style::Print(format!("Reading directory: {}, ", self.path))
            )?;

            let depth = if let Some(ref depth) = self.read_range {
                *depth.first().unwrap_or(&0)
            } else {
                0
            };
            crossterm::queue!(
                std::io::stdout(),
                crossterm::style::Print(format!("with maximum depth of {}", depth))
            )?;
        }

        Ok(())
    }

    async fn validate(&mut self, ctx: &Context) -> Result<(), Error> {
        let is_file = ctx.fs().symlink_metadata(&self.path).await?.is_file();
        self.ty = Some(is_file);

        Ok(())
    }
}

fn format_ftype(md: &Metadata) -> char {
    if md.is_symlink() {
        'l'
    } else if md.is_file() {
        '-'
    } else if md.is_dir() {
        'd'
    } else {
        warn!("unknown file metadata: {:?}", md);
        '-'
    }
}

/// Formats a permissions mode into the form used by `ls`, e.g. `0o644` to `rw-r--r--`
fn format_mode(mode: u32) -> [char; 9] {
    let mut mode = mode & 0o777;
    let mut res = ['-'; 9];
    fn octal_to_chars(val: u32) -> [char; 3] {
        match val {
            1 => ['-', '-', 'x'],
            2 => ['-', 'w', '-'],
            3 => ['-', 'w', 'x'],
            4 => ['r', '-', '-'],
            5 => ['r', '-', 'x'],
            6 => ['r', 'w', '-'],
            7 => ['r', 'w', 'x'],
            _ => ['-', '-', '-'],
        }
    }
    for c in res.rchunks_exact_mut(3) {
        c.copy_from_slice(&octal_to_chars(mode & 0o7));
        mode /= 0o10;
    }
    res
}

impl Display for FsRead {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crossterm::queue!(
            std::io::stdout(),
            crossterm::style::Print(format!("fs read with path {}, ", self.path))
        )
        .map_err(|_| std::fmt::Error)?;

        Ok(())
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
    fn test_fs_read_creation() {
        let v = serde_json::json!({ "path": "/test_file.txt", "read_range": vec![1, 2] });
        let fr = serde_json::from_value::<FsRead>(v).unwrap();

        assert_eq!(fr.path, TEST_FILE_PATH);
        assert_eq!(fr.read_range.unwrap(), vec![1, 2]);

        let v = serde_json::json!({ "path": "/test_file.txt", "read_range": vec![-1] });
        let fr = serde_json::from_value::<FsRead>(v).unwrap();

        assert_eq!(fr.path, TEST_FILE_PATH);
        assert_eq!(fr.read_range.unwrap(), vec![-1]);
    }

    #[tokio::test]
    async fn test_fs_read_tool_for_files() {
        let ctx = setup_test_directory().await;
        let lines = TEST_FILE_CONTENTS.lines().collect::<Vec<_>>();

        macro_rules! assert_lines {
            ($range:expr, $expected:expr) => {
                let v = serde_json::json!({
                    "path": TEST_FILE_PATH,
                    "read_range": $range,
                });
                let output = serde_json::from_value::<FsRead>(v).unwrap().invoke(&ctx).await.unwrap();

                if let OutputKind::Text(text) = output.output {
                    assert_eq!(text, $expected.join("\n"), "actual(left) does not equal expected(right) for range: {:?}", $range);
                } else {
                    panic!("expected text output");
                }
            }
        }
        assert_lines!((1, 2), lines[..=1]);
        assert_lines!((1, -1), lines[..]);
        assert_lines!((2, 1), [] as [&str; 0]);
        assert_lines!((-2, -1), lines[2..]);
    }

    #[test]
    fn test_format_mode() {
        macro_rules! assert_mode {
            ($actual:expr, $expected:expr) => {
                assert_eq!(format_mode($actual).iter().collect::<String>(), $expected);
            };
        }
        assert_mode!(0o000, "---------");
        assert_mode!(0o700, "rwx------");
        assert_mode!(0o744, "rwxr--r--");
        assert_mode!(0o641, "rw-r----x");
    }

    #[tokio::test]
    async fn test_fs_read_tool_for_directories() {
        let ctx = setup_test_directory().await;

        // Testing without depth
        let v = serde_json::json!({
            "path": "/",
            "read_range": None::<()>,
        });
        let output = serde_json::from_value::<FsRead>(v).unwrap().invoke(&ctx).await.unwrap();

        if let OutputKind::Text(text) = output.output {
            assert_eq!(text.lines().collect::<Vec<_>>().len(), 4);
        } else {
            panic!("expected text output");
        }

        // Testing with depth level 1
        let v = serde_json::json!({
            "path": "/",
            "read_range": Some(vec![1]),
        });
        let output = serde_json::from_value::<FsRead>(v).unwrap().invoke(&ctx).await.unwrap();

        if let OutputKind::Text(text) = output.output {
            let lines = text.lines().collect::<Vec<_>>();
            assert_eq!(lines.len(), 7);
            assert!(
                !lines.iter().any(|l| l.contains("cccc1")),
                "directory at depth level 2 should not be included in output"
            );
        } else {
            panic!("expected text output");
        }
    }
}
