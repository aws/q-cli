// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Streaming events from UniDirectional Streaming Conversational APIs.
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub enum ChatResponseStream {
    /// Assistant response event - Text / Code snippet
    AssistantResponseEvent(crate::types::AssistantResponseEvent),
    /// Code References event
    CodeReferenceEvent(crate::types::CodeReferenceEvent),
    /// Followup prompt event
    FollowupPromptEvent(crate::types::FollowupPromptEvent),
    /// Message Metadata event
    MessageMetadataEvent(crate::types::MessageMetadataEvent),
    /// Web Reference links event
    SupplementaryWebLinksEvent(crate::types::SupplementaryWebLinksEvent),
    /// The `Unknown` variant represents cases where new union variant was received. Consider upgrading the SDK to the latest available version.
    /// An unknown enum variant
    ///
    /// _Note: If you encounter this error, consider upgrading your SDK to the latest version._
    /// The `Unknown` variant represents cases where the server sent a value that wasn't recognized
    /// by the client. This can happen when the server adds new functionality, but the client has not been updated.
    /// To investigate this, consider turning on debug logging to print the raw HTTP response.
    #[non_exhaustive]
    Unknown,
}
impl ChatResponseStream {
    /// Tries to convert the enum instance into [`AssistantResponseEvent`](crate::types::ChatResponseStream::AssistantResponseEvent), extracting the inner [`AssistantResponseEvent`](crate::types::AssistantResponseEvent).
    /// Returns `Err(&Self)` if it can't be converted.
    pub fn as_assistant_response_event(&self) -> ::std::result::Result<&crate::types::AssistantResponseEvent, &Self> {
        if let ChatResponseStream::AssistantResponseEvent(val) = &self {
            ::std::result::Result::Ok(val)
        } else {
            ::std::result::Result::Err(self)
        }
    }
    /// Returns true if this is a [`AssistantResponseEvent`](crate::types::ChatResponseStream::AssistantResponseEvent).
    pub fn is_assistant_response_event(&self) -> bool {
        self.as_assistant_response_event().is_ok()
    }
    /// Tries to convert the enum instance into [`CodeReferenceEvent`](crate::types::ChatResponseStream::CodeReferenceEvent), extracting the inner [`CodeReferenceEvent`](crate::types::CodeReferenceEvent).
    /// Returns `Err(&Self)` if it can't be converted.
    pub fn as_code_reference_event(&self) -> ::std::result::Result<&crate::types::CodeReferenceEvent, &Self> {
        if let ChatResponseStream::CodeReferenceEvent(val) = &self {
            ::std::result::Result::Ok(val)
        } else {
            ::std::result::Result::Err(self)
        }
    }
    /// Returns true if this is a [`CodeReferenceEvent`](crate::types::ChatResponseStream::CodeReferenceEvent).
    pub fn is_code_reference_event(&self) -> bool {
        self.as_code_reference_event().is_ok()
    }
    /// Tries to convert the enum instance into [`FollowupPromptEvent`](crate::types::ChatResponseStream::FollowupPromptEvent), extracting the inner [`FollowupPromptEvent`](crate::types::FollowupPromptEvent).
    /// Returns `Err(&Self)` if it can't be converted.
    pub fn as_followup_prompt_event(&self) -> ::std::result::Result<&crate::types::FollowupPromptEvent, &Self> {
        if let ChatResponseStream::FollowupPromptEvent(val) = &self {
            ::std::result::Result::Ok(val)
        } else {
            ::std::result::Result::Err(self)
        }
    }
    /// Returns true if this is a [`FollowupPromptEvent`](crate::types::ChatResponseStream::FollowupPromptEvent).
    pub fn is_followup_prompt_event(&self) -> bool {
        self.as_followup_prompt_event().is_ok()
    }
    /// Tries to convert the enum instance into [`MessageMetadataEvent`](crate::types::ChatResponseStream::MessageMetadataEvent), extracting the inner [`MessageMetadataEvent`](crate::types::MessageMetadataEvent).
    /// Returns `Err(&Self)` if it can't be converted.
    pub fn as_message_metadata_event(&self) -> ::std::result::Result<&crate::types::MessageMetadataEvent, &Self> {
        if let ChatResponseStream::MessageMetadataEvent(val) = &self {
            ::std::result::Result::Ok(val)
        } else {
            ::std::result::Result::Err(self)
        }
    }
    /// Returns true if this is a [`MessageMetadataEvent`](crate::types::ChatResponseStream::MessageMetadataEvent).
    pub fn is_message_metadata_event(&self) -> bool {
        self.as_message_metadata_event().is_ok()
    }
    /// Tries to convert the enum instance into [`SupplementaryWebLinksEvent`](crate::types::ChatResponseStream::SupplementaryWebLinksEvent), extracting the inner [`SupplementaryWebLinksEvent`](crate::types::SupplementaryWebLinksEvent).
    /// Returns `Err(&Self)` if it can't be converted.
    pub fn as_supplementary_web_links_event(&self) -> ::std::result::Result<&crate::types::SupplementaryWebLinksEvent, &Self> {
        if let ChatResponseStream::SupplementaryWebLinksEvent(val) = &self {
            ::std::result::Result::Ok(val)
        } else {
            ::std::result::Result::Err(self)
        }
    }
    /// Returns true if this is a [`SupplementaryWebLinksEvent`](crate::types::ChatResponseStream::SupplementaryWebLinksEvent).
    pub fn is_supplementary_web_links_event(&self) -> bool {
        self.as_supplementary_web_links_event().is_ok()
    }
    /// Returns true if the enum instance is the `Unknown` variant.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}
