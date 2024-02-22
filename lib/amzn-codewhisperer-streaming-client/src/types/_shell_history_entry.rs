// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// An single entry in the shell history
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq)]
pub struct ShellHistoryEntry {
    /// The shell command that was run
    pub command: ::std::string::String,
    /// The directory the command was ran in
    pub directory: ::std::option::Option<::std::string::String>,
    /// The exit code of the command after it finished
    pub exit_code: ::std::option::Option<i32>,
    /// The stdout from the command
    pub stdout: ::std::option::Option<::std::string::String>,
    /// The stderr from the command
    pub stderr: ::std::option::Option<::std::string::String>,
}
impl ShellHistoryEntry {
    /// The shell command that was run
    pub fn command(&self) -> &str {
        use std::ops::Deref;
        self.command.deref()
    }

    /// The directory the command was ran in
    pub fn directory(&self) -> ::std::option::Option<&str> {
        self.directory.as_deref()
    }

    /// The exit code of the command after it finished
    pub fn exit_code(&self) -> ::std::option::Option<i32> {
        self.exit_code
    }

    /// The stdout from the command
    pub fn stdout(&self) -> ::std::option::Option<&str> {
        self.stdout.as_deref()
    }

    /// The stderr from the command
    pub fn stderr(&self) -> ::std::option::Option<&str> {
        self.stderr.as_deref()
    }
}
impl ::std::fmt::Debug for ShellHistoryEntry {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut formatter = f.debug_struct("ShellHistoryEntry");
        formatter.field("command", &"*** Sensitive Data Redacted ***");
        formatter.field("directory", &"*** Sensitive Data Redacted ***");
        formatter.field("exit_code", &self.exit_code);
        formatter.field("stdout", &"*** Sensitive Data Redacted ***");
        formatter.field("stderr", &"*** Sensitive Data Redacted ***");
        formatter.finish()
    }
}
impl ShellHistoryEntry {
    /// Creates a new builder-style object to manufacture
    /// [`ShellHistoryEntry`](crate::types::ShellHistoryEntry).
    pub fn builder() -> crate::types::builders::ShellHistoryEntryBuilder {
        crate::types::builders::ShellHistoryEntryBuilder::default()
    }
}

/// A builder for [`ShellHistoryEntry`](crate::types::ShellHistoryEntry).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default)]
pub struct ShellHistoryEntryBuilder {
    pub(crate) command: ::std::option::Option<::std::string::String>,
    pub(crate) directory: ::std::option::Option<::std::string::String>,
    pub(crate) exit_code: ::std::option::Option<i32>,
    pub(crate) stdout: ::std::option::Option<::std::string::String>,
    pub(crate) stderr: ::std::option::Option<::std::string::String>,
}
impl ShellHistoryEntryBuilder {
    /// The shell command that was run
    /// This field is required.
    pub fn command(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.command = ::std::option::Option::Some(input.into());
        self
    }

    /// The shell command that was run
    pub fn set_command(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.command = input;
        self
    }

    /// The shell command that was run
    pub fn get_command(&self) -> &::std::option::Option<::std::string::String> {
        &self.command
    }

    /// The directory the command was ran in
    pub fn directory(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.directory = ::std::option::Option::Some(input.into());
        self
    }

    /// The directory the command was ran in
    pub fn set_directory(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.directory = input;
        self
    }

    /// The directory the command was ran in
    pub fn get_directory(&self) -> &::std::option::Option<::std::string::String> {
        &self.directory
    }

    /// The exit code of the command after it finished
    pub fn exit_code(mut self, input: i32) -> Self {
        self.exit_code = ::std::option::Option::Some(input);
        self
    }

    /// The exit code of the command after it finished
    pub fn set_exit_code(mut self, input: ::std::option::Option<i32>) -> Self {
        self.exit_code = input;
        self
    }

    /// The exit code of the command after it finished
    pub fn get_exit_code(&self) -> &::std::option::Option<i32> {
        &self.exit_code
    }

    /// The stdout from the command
    pub fn stdout(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.stdout = ::std::option::Option::Some(input.into());
        self
    }

    /// The stdout from the command
    pub fn set_stdout(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.stdout = input;
        self
    }

    /// The stdout from the command
    pub fn get_stdout(&self) -> &::std::option::Option<::std::string::String> {
        &self.stdout
    }

    /// The stderr from the command
    pub fn stderr(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.stderr = ::std::option::Option::Some(input.into());
        self
    }

    /// The stderr from the command
    pub fn set_stderr(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.stderr = input;
        self
    }

    /// The stderr from the command
    pub fn get_stderr(&self) -> &::std::option::Option<::std::string::String> {
        &self.stderr
    }

    /// Consumes the builder and constructs a
    /// [`ShellHistoryEntry`](crate::types::ShellHistoryEntry). This method will fail if any of
    /// the following fields are not set:
    /// - [`command`](crate::types::builders::ShellHistoryEntryBuilder::command)
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::types::ShellHistoryEntry, ::aws_smithy_types::error::operation::BuildError> {
        ::std::result::Result::Ok(crate::types::ShellHistoryEntry {
            command: self.command.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "command",
                    "command was not specified but it is required when building ShellHistoryEntry",
                )
            })?,
            directory: self.directory,
            exit_code: self.exit_code,
            stdout: self.stdout,
            stderr: self.stderr,
        })
    }
}
impl ::std::fmt::Debug for ShellHistoryEntryBuilder {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut formatter = f.debug_struct("ShellHistoryEntryBuilder");
        formatter.field("command", &"*** Sensitive Data Redacted ***");
        formatter.field("directory", &"*** Sensitive Data Redacted ***");
        formatter.field("exit_code", &self.exit_code);
        formatter.field("stdout", &"*** Sensitive Data Redacted ***");
        formatter.field("stderr", &"*** Sensitive Data Redacted ***");
        formatter.finish()
    }
}
