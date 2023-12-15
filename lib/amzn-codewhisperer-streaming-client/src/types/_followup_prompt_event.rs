// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Streaming Response Event for Followup Prompt.
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct FollowupPromptEvent {
    /// Followup Prompt for the Assistant Response
    pub followup_prompt: ::std::option::Option<crate::types::FollowupPrompt>,
}
impl FollowupPromptEvent {
    /// Followup Prompt for the Assistant Response
    pub fn followup_prompt(&self) -> ::std::option::Option<&crate::types::FollowupPrompt> {
        self.followup_prompt.as_ref()
    }
}
impl FollowupPromptEvent {
    /// Creates a new builder-style object to manufacture [`FollowupPromptEvent`](crate::types::FollowupPromptEvent).
    pub fn builder() -> crate::types::builders::FollowupPromptEventBuilder {
        crate::types::builders::FollowupPromptEventBuilder::default()
    }
}

/// A builder for [`FollowupPromptEvent`](crate::types::FollowupPromptEvent).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct FollowupPromptEventBuilder {
    pub(crate) followup_prompt: ::std::option::Option<crate::types::FollowupPrompt>,
}
impl FollowupPromptEventBuilder {
    /// Followup Prompt for the Assistant Response
    pub fn followup_prompt(mut self, input: crate::types::FollowupPrompt) -> Self {
        self.followup_prompt = ::std::option::Option::Some(input);
        self
    }
    /// Followup Prompt for the Assistant Response
    pub fn set_followup_prompt(mut self, input: ::std::option::Option<crate::types::FollowupPrompt>) -> Self {
        self.followup_prompt = input;
        self
    }
    /// Followup Prompt for the Assistant Response
    pub fn get_followup_prompt(&self) -> &::std::option::Option<crate::types::FollowupPrompt> {
        &self.followup_prompt
    }
    /// Consumes the builder and constructs a [`FollowupPromptEvent`](crate::types::FollowupPromptEvent).
    pub fn build(self) -> crate::types::FollowupPromptEvent {
        crate::types::FollowupPromptEvent {
            followup_prompt: self.followup_prompt,
        }
    }
}
