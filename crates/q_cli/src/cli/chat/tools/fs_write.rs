use std::borrow::Cow;
use std::io::Write;

use crossterm::queue;
use crossterm::style::{
    self,
    Color,
};
use eyre::{
    Result,
    eyre,
};
use fig_os_shim::Context;
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

use super::{
    InvokeOutput,
    relative_path,
};

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
                    style::Print(format!("Creating a new file at {}", relative_path(&cwd, path))),
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
                    style::Print(format!("Updating {}", relative_path(&cwd, path))),
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
                        relative_path(&cwd, path)
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

    pub fn queue_description(&self, updates: &mut impl Write) -> Result<()> {
        match self {
            FsWrite::Create { path, file_text } => Ok(queue!(
                updates,
                style::Print(format!(
                    "fs write create with path {} with {} ...\n",
                    path,
                    file_text.chars().take(10).collect::<String>()
                ))
            )?),
            FsWrite::Insert {
                path,
                insert_line,
                new_str,
            } => Ok(queue!(
                updates,
                style::Print(format!(
                    "fs write insert with path {} at line {} with {} ...\n",
                    path,
                    insert_line,
                    new_str.chars().take(10).collect::<String>()
                ))
            )?),
            FsWrite::StrReplace { path, old_str, new_str } => Ok(queue!(
                updates,
                style::Print(format!(
                    "fs write str replace with path {} replacing {} with {}\n",
                    path, old_str, new_str
                ))
            )?),
        }
    }

    pub async fn validate(&mut self, _ctx: &Context) -> Result<()> {
        // TODO: check to see if paths are valid
        Ok(())
    }
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
}
