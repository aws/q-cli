// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct PostFeedbackOutput {
    _request_id: Option<String>,
}
impl ::aws_http::request_id::RequestId for PostFeedbackOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl PostFeedbackOutput {
    /// Creates a new builder-style object to manufacture
    /// [`PostFeedbackOutput`](crate::operation::post_feedback::PostFeedbackOutput).
    pub fn builder() -> crate::operation::post_feedback::builders::PostFeedbackOutputBuilder {
        crate::operation::post_feedback::builders::PostFeedbackOutputBuilder::default()
    }
}

/// A builder for [`PostFeedbackOutput`](crate::operation::post_feedback::PostFeedbackOutput).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct PostFeedbackOutputBuilder {
    _request_id: Option<String>,
}
impl PostFeedbackOutputBuilder {
    pub(crate) fn _request_id(mut self, request_id: impl Into<String>) -> Self {
        self._request_id = Some(request_id.into());
        self
    }

    pub(crate) fn _set_request_id(&mut self, request_id: Option<String>) -> &mut Self {
        self._request_id = request_id;
        self
    }

    /// Consumes the builder and constructs a
    /// [`PostFeedbackOutput`](crate::operation::post_feedback::PostFeedbackOutput).
    pub fn build(self) -> crate::operation::post_feedback::PostFeedbackOutput {
        crate::operation::post_feedback::PostFeedbackOutput {
            _request_id: self._request_id,
        }
    }
}
