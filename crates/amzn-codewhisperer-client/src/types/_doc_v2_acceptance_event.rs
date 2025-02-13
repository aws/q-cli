// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Interaction event for /doc, emitted when user accepts or rejects the generated content
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct DocV2AcceptanceEvent {
    /// ID which represents a multi-turn conversation
    pub conversation_id: ::std::string::String,
    #[allow(missing_docs)] // documentation missing in model
    pub number_of_added_chars: i32,
    #[allow(missing_docs)] // documentation missing in model
    pub number_of_added_lines: i32,
    #[allow(missing_docs)] // documentation missing in model
    pub number_of_added_files: i32,
    #[allow(missing_docs)] // documentation missing in model
    pub user_decision: crate::types::DocUserDecision,
    /// Tracks whether user chose to generate a new document, update an existing one, or edit
    /// document
    pub interaction_type: crate::types::DocInteractionType,
    #[allow(missing_docs)] // documentation missing in model
    pub number_of_navigations: i32,
    /// Specifies the folder depth level where the document should be generated
    pub folder_level: crate::types::DocFolderLevel,
}
impl DocV2AcceptanceEvent {
    /// ID which represents a multi-turn conversation
    pub fn conversation_id(&self) -> &str {
        use std::ops::Deref;
        self.conversation_id.deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn number_of_added_chars(&self) -> i32 {
        self.number_of_added_chars
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn number_of_added_lines(&self) -> i32 {
        self.number_of_added_lines
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn number_of_added_files(&self) -> i32 {
        self.number_of_added_files
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn user_decision(&self) -> &crate::types::DocUserDecision {
        &self.user_decision
    }

    /// Tracks whether user chose to generate a new document, update an existing one, or edit
    /// document
    pub fn interaction_type(&self) -> &crate::types::DocInteractionType {
        &self.interaction_type
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn number_of_navigations(&self) -> i32 {
        self.number_of_navigations
    }

    /// Specifies the folder depth level where the document should be generated
    pub fn folder_level(&self) -> &crate::types::DocFolderLevel {
        &self.folder_level
    }
}
impl DocV2AcceptanceEvent {
    /// Creates a new builder-style object to manufacture
    /// [`DocV2AcceptanceEvent`](crate::types::DocV2AcceptanceEvent).
    pub fn builder() -> crate::types::builders::DocV2AcceptanceEventBuilder {
        crate::types::builders::DocV2AcceptanceEventBuilder::default()
    }
}

/// A builder for [`DocV2AcceptanceEvent`](crate::types::DocV2AcceptanceEvent).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct DocV2AcceptanceEventBuilder {
    pub(crate) conversation_id: ::std::option::Option<::std::string::String>,
    pub(crate) number_of_added_chars: ::std::option::Option<i32>,
    pub(crate) number_of_added_lines: ::std::option::Option<i32>,
    pub(crate) number_of_added_files: ::std::option::Option<i32>,
    pub(crate) user_decision: ::std::option::Option<crate::types::DocUserDecision>,
    pub(crate) interaction_type: ::std::option::Option<crate::types::DocInteractionType>,
    pub(crate) number_of_navigations: ::std::option::Option<i32>,
    pub(crate) folder_level: ::std::option::Option<crate::types::DocFolderLevel>,
}
impl DocV2AcceptanceEventBuilder {
    /// ID which represents a multi-turn conversation
    /// This field is required.
    pub fn conversation_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.conversation_id = ::std::option::Option::Some(input.into());
        self
    }

    /// ID which represents a multi-turn conversation
    pub fn set_conversation_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.conversation_id = input;
        self
    }

    /// ID which represents a multi-turn conversation
    pub fn get_conversation_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.conversation_id
    }

    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn number_of_added_chars(mut self, input: i32) -> Self {
        self.number_of_added_chars = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_number_of_added_chars(mut self, input: ::std::option::Option<i32>) -> Self {
        self.number_of_added_chars = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_number_of_added_chars(&self) -> &::std::option::Option<i32> {
        &self.number_of_added_chars
    }

    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn number_of_added_lines(mut self, input: i32) -> Self {
        self.number_of_added_lines = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_number_of_added_lines(mut self, input: ::std::option::Option<i32>) -> Self {
        self.number_of_added_lines = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_number_of_added_lines(&self) -> &::std::option::Option<i32> {
        &self.number_of_added_lines
    }

    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn number_of_added_files(mut self, input: i32) -> Self {
        self.number_of_added_files = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_number_of_added_files(mut self, input: ::std::option::Option<i32>) -> Self {
        self.number_of_added_files = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_number_of_added_files(&self) -> &::std::option::Option<i32> {
        &self.number_of_added_files
    }

    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn user_decision(mut self, input: crate::types::DocUserDecision) -> Self {
        self.user_decision = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_user_decision(mut self, input: ::std::option::Option<crate::types::DocUserDecision>) -> Self {
        self.user_decision = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_user_decision(&self) -> &::std::option::Option<crate::types::DocUserDecision> {
        &self.user_decision
    }

    /// Tracks whether user chose to generate a new document, update an existing one, or edit
    /// document This field is required.
    pub fn interaction_type(mut self, input: crate::types::DocInteractionType) -> Self {
        self.interaction_type = ::std::option::Option::Some(input);
        self
    }

    /// Tracks whether user chose to generate a new document, update an existing one, or edit
    /// document
    pub fn set_interaction_type(mut self, input: ::std::option::Option<crate::types::DocInteractionType>) -> Self {
        self.interaction_type = input;
        self
    }

    /// Tracks whether user chose to generate a new document, update an existing one, or edit
    /// document
    pub fn get_interaction_type(&self) -> &::std::option::Option<crate::types::DocInteractionType> {
        &self.interaction_type
    }

    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn number_of_navigations(mut self, input: i32) -> Self {
        self.number_of_navigations = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_number_of_navigations(mut self, input: ::std::option::Option<i32>) -> Self {
        self.number_of_navigations = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_number_of_navigations(&self) -> &::std::option::Option<i32> {
        &self.number_of_navigations
    }

    /// Specifies the folder depth level where the document should be generated
    /// This field is required.
    pub fn folder_level(mut self, input: crate::types::DocFolderLevel) -> Self {
        self.folder_level = ::std::option::Option::Some(input);
        self
    }

    /// Specifies the folder depth level where the document should be generated
    pub fn set_folder_level(mut self, input: ::std::option::Option<crate::types::DocFolderLevel>) -> Self {
        self.folder_level = input;
        self
    }

    /// Specifies the folder depth level where the document should be generated
    pub fn get_folder_level(&self) -> &::std::option::Option<crate::types::DocFolderLevel> {
        &self.folder_level
    }

    /// Consumes the builder and constructs a
    /// [`DocV2AcceptanceEvent`](crate::types::DocV2AcceptanceEvent). This method will fail if
    /// any of the following fields are not set:
    /// - [`conversation_id`](crate::types::builders::DocV2AcceptanceEventBuilder::conversation_id)
    /// - [`user_decision`](crate::types::builders::DocV2AcceptanceEventBuilder::user_decision)
    /// - [`interaction_type`](crate::types::builders::DocV2AcceptanceEventBuilder::interaction_type)
    /// - [`folder_level`](crate::types::builders::DocV2AcceptanceEventBuilder::folder_level)
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::types::DocV2AcceptanceEvent, ::aws_smithy_types::error::operation::BuildError>
    {
        ::std::result::Result::Ok(crate::types::DocV2AcceptanceEvent {
            conversation_id: self.conversation_id.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "conversation_id",
                    "conversation_id was not specified but it is required when building DocV2AcceptanceEvent",
                )
            })?,
            number_of_added_chars: self.number_of_added_chars.unwrap_or_default(),
            number_of_added_lines: self.number_of_added_lines.unwrap_or_default(),
            number_of_added_files: self.number_of_added_files.unwrap_or_default(),
            user_decision: self.user_decision.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "user_decision",
                    "user_decision was not specified but it is required when building DocV2AcceptanceEvent",
                )
            })?,
            interaction_type: self.interaction_type.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "interaction_type",
                    "interaction_type was not specified but it is required when building DocV2AcceptanceEvent",
                )
            })?,
            number_of_navigations: self.number_of_navigations.unwrap_or_default(),
            folder_level: self.folder_level.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "folder_level",
                    "folder_level was not specified but it is required when building DocV2AcceptanceEvent",
                )
            })?,
        })
    }
}
