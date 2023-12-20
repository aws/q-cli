// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Additional Chat message context associated with the Chat Message
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct UserInputMessageContext {
    /// Editor state chat message context.
    pub editor_state: ::std::option::Option<crate::types::EditorState>,
    /// Diagnostic chat message context.
    pub diagnostic: ::std::option::Option<crate::types::Diagnostic>,
}
impl UserInputMessageContext {
    /// Editor state chat message context.
    pub fn editor_state(&self) -> ::std::option::Option<&crate::types::EditorState> {
        self.editor_state.as_ref()
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
            diagnostic: self.diagnostic,
        }
    }
}
