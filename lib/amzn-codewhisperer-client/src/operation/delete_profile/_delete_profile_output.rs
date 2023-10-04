// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct DeleteProfileOutput {
    _request_id: Option<String>,
}
impl ::aws_http::request_id::RequestId for DeleteProfileOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl DeleteProfileOutput {
    /// Creates a new builder-style object to manufacture
    /// [`DeleteProfileOutput`](crate::operation::delete_profile::DeleteProfileOutput).
    pub fn builder() -> crate::operation::delete_profile::builders::DeleteProfileOutputBuilder {
        crate::operation::delete_profile::builders::DeleteProfileOutputBuilder::default()
    }
}

/// A builder for [`DeleteProfileOutput`](crate::operation::delete_profile::DeleteProfileOutput).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct DeleteProfileOutputBuilder {
    _request_id: Option<String>,
}
impl DeleteProfileOutputBuilder {
    pub(crate) fn _request_id(mut self, request_id: impl Into<String>) -> Self {
        self._request_id = Some(request_id.into());
        self
    }

    pub(crate) fn _set_request_id(&mut self, request_id: Option<String>) -> &mut Self {
        self._request_id = request_id;
        self
    }

    /// Consumes the builder and constructs a
    /// [`DeleteProfileOutput`](crate::operation::delete_profile::DeleteProfileOutput).
    pub fn build(self) -> crate::operation::delete_profile::DeleteProfileOutput {
        crate::operation::delete_profile::DeleteProfileOutput {
            _request_id: self._request_id,
        }
    }
}
