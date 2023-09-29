// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct UntagResourceOutput {
    _request_id: Option<String>,
}
impl ::aws_http::request_id::RequestId for UntagResourceOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl UntagResourceOutput {
    /// Creates a new builder-style object to manufacture
    /// [`UntagResourceOutput`](crate::operation::untag_resource::UntagResourceOutput).
    pub fn builder() -> crate::operation::untag_resource::builders::UntagResourceOutputBuilder {
        crate::operation::untag_resource::builders::UntagResourceOutputBuilder::default()
    }
}

/// A builder for [`UntagResourceOutput`](crate::operation::untag_resource::UntagResourceOutput).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct UntagResourceOutputBuilder {
    _request_id: Option<String>,
}
impl UntagResourceOutputBuilder {
    pub(crate) fn _request_id(mut self, request_id: impl Into<String>) -> Self {
        self._request_id = Some(request_id.into());
        self
    }

    pub(crate) fn _set_request_id(&mut self, request_id: Option<String>) -> &mut Self {
        self._request_id = request_id;
        self
    }

    /// Consumes the builder and constructs a
    /// [`UntagResourceOutput`](crate::operation::untag_resource::UntagResourceOutput).
    pub fn build(self) -> crate::operation::untag_resource::UntagResourceOutput {
        crate::operation::untag_resource::UntagResourceOutput {
            _request_id: self._request_id,
        }
    }
}
