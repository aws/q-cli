// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Structure to represent generate assistant response response.
#[non_exhaustive]
#[derive(::std::fmt::Debug)]
pub struct GenerateAssistantResponseOutput {
    /// Streaming events from UniDirectional Streaming Conversational APIs.
    pub generate_assistant_response_response: crate::event_receiver::EventReceiver<
        crate::types::ChatResponseStream,
        crate::types::error::ChatResponseStreamError,
    >,
    _request_id: Option<String>,
}
impl GenerateAssistantResponseOutput {
    /// Streaming events from UniDirectional Streaming Conversational APIs.
    pub fn generate_assistant_response_response(
        &self,
    ) -> &crate::event_receiver::EventReceiver<
        crate::types::ChatResponseStream,
        crate::types::error::ChatResponseStreamError,
    > {
        &self.generate_assistant_response_response
    }
}
impl ::aws_types::request_id::RequestId for GenerateAssistantResponseOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl GenerateAssistantResponseOutput {
    /// Creates a new builder-style object to manufacture
    /// [`GenerateAssistantResponseOutput`](crate::operation::generate_assistant_response::GenerateAssistantResponseOutput).
    ///
    pub fn builder() -> crate::operation::generate_assistant_response::builders::GenerateAssistantResponseOutputBuilder
    {
        crate::operation::generate_assistant_response::builders::GenerateAssistantResponseOutputBuilder::default()
    }
}

/// A builder for
/// [`GenerateAssistantResponseOutput`](crate::operation::generate_assistant_response::GenerateAssistantResponseOutput).
///
#[non_exhaustive]
#[derive(::std::default::Default, ::std::fmt::Debug)]
pub struct GenerateAssistantResponseOutputBuilder {
    pub(crate) generate_assistant_response_response: ::std::option::Option<
        crate::event_receiver::EventReceiver<
            crate::types::ChatResponseStream,
            crate::types::error::ChatResponseStreamError,
        >,
    >,
    _request_id: Option<String>,
}
impl GenerateAssistantResponseOutputBuilder {
    /// Streaming events from UniDirectional Streaming Conversational APIs.
    /// This field is required.
    pub fn generate_assistant_response_response(
        mut self,
        input: crate::event_receiver::EventReceiver<
            crate::types::ChatResponseStream,
            crate::types::error::ChatResponseStreamError,
        >,
    ) -> Self {
        self.generate_assistant_response_response = ::std::option::Option::Some(input);
        self
    }

    /// Streaming events from UniDirectional Streaming Conversational APIs.
    pub fn set_generate_assistant_response_response(
        mut self,
        input: ::std::option::Option<
            crate::event_receiver::EventReceiver<
                crate::types::ChatResponseStream,
                crate::types::error::ChatResponseStreamError,
            >,
        >,
    ) -> Self {
        self.generate_assistant_response_response = input;
        self
    }

    /// Streaming events from UniDirectional Streaming Conversational APIs.
    pub fn get_generate_assistant_response_response(
        &self,
    ) -> &::std::option::Option<
        crate::event_receiver::EventReceiver<
            crate::types::ChatResponseStream,
            crate::types::error::ChatResponseStreamError,
        >,
    > {
        &self.generate_assistant_response_response
    }

    pub(crate) fn _request_id(mut self, request_id: impl Into<String>) -> Self {
        self._request_id = Some(request_id.into());
        self
    }

    pub(crate) fn _set_request_id(&mut self, request_id: Option<String>) -> &mut Self {
        self._request_id = request_id;
        self
    }

    /// Consumes the builder and constructs a
    /// [`GenerateAssistantResponseOutput`](crate::operation::generate_assistant_response::GenerateAssistantResponseOutput).
    /// This method will fail if any of the following fields are not set:
    /// - [`generate_assistant_response_response`](crate::operation::generate_assistant_response::builders::GenerateAssistantResponseOutputBuilder::generate_assistant_response_response)
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::generate_assistant_response::GenerateAssistantResponseOutput,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(crate::operation::generate_assistant_response::GenerateAssistantResponseOutput {
            generate_assistant_response_response: self.generate_assistant_response_response.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "generate_assistant_response_response",
                    "generate_assistant_response_response was not specified but it is required when building GenerateAssistantResponseOutput",
                )
            })?,
            _request_id: self._request_id,
        })
    }
}
