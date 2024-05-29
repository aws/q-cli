// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Additional Chat message context associated with the Chat Message
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct UserInputMessageContext {
    /// Editor state chat message context.
    pub editor_state: ::std::option::Option<crate::types::EditorState>,
    /// Shell state chat message context.
    pub shell_state: ::std::option::Option<crate::types::ShellState>,
    /// Git state chat message context.
    pub git_state: ::std::option::Option<crate::types::GitState>,
    /// Environment state chat message context.
    pub env_state: ::std::option::Option<crate::types::EnvState>,
    /// The state of a user's AppStudio UI when sending a message.
    pub app_studio_context: ::std::option::Option<crate::types::AppStudioState>,
    /// Diagnostic chat message context.
    pub diagnostic: ::std::option::Option<crate::types::Diagnostic>,
}
impl UserInputMessageContext {
    /// Editor state chat message context.
    pub fn editor_state(&self) -> ::std::option::Option<&crate::types::EditorState> {
        self.editor_state.as_ref()
    }

    /// Shell state chat message context.
    pub fn shell_state(&self) -> ::std::option::Option<&crate::types::ShellState> {
        self.shell_state.as_ref()
    }

    /// Git state chat message context.
    pub fn git_state(&self) -> ::std::option::Option<&crate::types::GitState> {
        self.git_state.as_ref()
    }

    /// Environment state chat message context.
    pub fn env_state(&self) -> ::std::option::Option<&crate::types::EnvState> {
        self.env_state.as_ref()
    }

    /// The state of a user's AppStudio UI when sending a message.
    pub fn app_studio_context(&self) -> ::std::option::Option<&crate::types::AppStudioState> {
        self.app_studio_context.as_ref()
    }

    /// Diagnostic chat message context.
    pub fn diagnostic(&self) -> ::std::option::Option<&crate::types::Diagnostic> {
        self.diagnostic.as_ref()
    }
}
impl UserInputMessageContext {
    /// Creates a new builder-style object to manufacture
    /// [`UserInputMessageContext`](crate::types::UserInputMessageContext).
    pub fn builder() -> crate::types::builders::UserInputMessageContextBuilder {
        crate::types::builders::UserInputMessageContextBuilder::default()
    }
}

/// A builder for [`UserInputMessageContext`](crate::types::UserInputMessageContext).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct UserInputMessageContextBuilder {
    pub(crate) editor_state: ::std::option::Option<crate::types::EditorState>,
    pub(crate) shell_state: ::std::option::Option<crate::types::ShellState>,
    pub(crate) git_state: ::std::option::Option<crate::types::GitState>,
    pub(crate) env_state: ::std::option::Option<crate::types::EnvState>,
    pub(crate) app_studio_context: ::std::option::Option<crate::types::AppStudioState>,
    pub(crate) diagnostic: ::std::option::Option<crate::types::Diagnostic>,
}
impl UserInputMessageContextBuilder {
    /// Editor state chat message context.
    pub fn editor_state(mut self, input: crate::types::EditorState) -> Self {
        self.editor_state = ::std::option::Option::Some(input);
        self
    }

    /// Editor state chat message context.
    pub fn set_editor_state(mut self, input: ::std::option::Option<crate::types::EditorState>) -> Self {
        self.editor_state = input;
        self
    }

    /// Editor state chat message context.
    pub fn get_editor_state(&self) -> &::std::option::Option<crate::types::EditorState> {
        &self.editor_state
    }

    /// Shell state chat message context.
    pub fn shell_state(mut self, input: crate::types::ShellState) -> Self {
        self.shell_state = ::std::option::Option::Some(input);
        self
    }

    /// Shell state chat message context.
    pub fn set_shell_state(mut self, input: ::std::option::Option<crate::types::ShellState>) -> Self {
        self.shell_state = input;
        self
    }

    /// Shell state chat message context.
    pub fn get_shell_state(&self) -> &::std::option::Option<crate::types::ShellState> {
        &self.shell_state
    }

    /// Git state chat message context.
    pub fn git_state(mut self, input: crate::types::GitState) -> Self {
        self.git_state = ::std::option::Option::Some(input);
        self
    }

    /// Git state chat message context.
    pub fn set_git_state(mut self, input: ::std::option::Option<crate::types::GitState>) -> Self {
        self.git_state = input;
        self
    }

    /// Git state chat message context.
    pub fn get_git_state(&self) -> &::std::option::Option<crate::types::GitState> {
        &self.git_state
    }

    /// Environment state chat message context.
    pub fn env_state(mut self, input: crate::types::EnvState) -> Self {
        self.env_state = ::std::option::Option::Some(input);
        self
    }

    /// Environment state chat message context.
    pub fn set_env_state(mut self, input: ::std::option::Option<crate::types::EnvState>) -> Self {
        self.env_state = input;
        self
    }

    /// Environment state chat message context.
    pub fn get_env_state(&self) -> &::std::option::Option<crate::types::EnvState> {
        &self.env_state
    }

    /// The state of a user's AppStudio UI when sending a message.
    pub fn app_studio_context(mut self, input: crate::types::AppStudioState) -> Self {
        self.app_studio_context = ::std::option::Option::Some(input);
        self
    }

    /// The state of a user's AppStudio UI when sending a message.
    pub fn set_app_studio_context(mut self, input: ::std::option::Option<crate::types::AppStudioState>) -> Self {
        self.app_studio_context = input;
        self
    }

    /// The state of a user's AppStudio UI when sending a message.
    pub fn get_app_studio_context(&self) -> &::std::option::Option<crate::types::AppStudioState> {
        &self.app_studio_context
    }

    /// Diagnostic chat message context.
    pub fn diagnostic(mut self, input: crate::types::Diagnostic) -> Self {
        self.diagnostic = ::std::option::Option::Some(input);
        self
    }

    /// Diagnostic chat message context.
    pub fn set_diagnostic(mut self, input: ::std::option::Option<crate::types::Diagnostic>) -> Self {
        self.diagnostic = input;
        self
    }

    /// Diagnostic chat message context.
    pub fn get_diagnostic(&self) -> &::std::option::Option<crate::types::Diagnostic> {
        &self.diagnostic
    }

    /// Consumes the builder and constructs a
    /// [`UserInputMessageContext`](crate::types::UserInputMessageContext).
    pub fn build(self) -> crate::types::UserInputMessageContext {
        crate::types::UserInputMessageContext {
            editor_state: self.editor_state,
            shell_state: self.shell_state,
            git_state: self.git_state,
            env_state: self.env_state,
            app_studio_context: self.app_studio_context,
            diagnostic: self.diagnostic,
        }
    }
}
