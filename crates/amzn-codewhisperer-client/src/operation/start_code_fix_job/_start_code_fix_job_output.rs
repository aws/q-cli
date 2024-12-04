// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct StartCodeFixJobOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub job_id: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub status: ::std::option::Option<crate::types::CodeFixJobStatus>,
    _request_id: Option<String>,
}
impl StartCodeFixJobOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub fn job_id(&self) -> ::std::option::Option<&str> {
        self.job_id.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn status(&self) -> ::std::option::Option<&crate::types::CodeFixJobStatus> {
        self.status.as_ref()
    }
}
impl ::aws_types::request_id::RequestId for StartCodeFixJobOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl StartCodeFixJobOutput {
    /// Creates a new builder-style object to manufacture
    /// [`StartCodeFixJobOutput`](crate::operation::start_code_fix_job::StartCodeFixJobOutput).
    pub fn builder() -> crate::operation::start_code_fix_job::builders::StartCodeFixJobOutputBuilder {
        crate::operation::start_code_fix_job::builders::StartCodeFixJobOutputBuilder::default()
    }
}

/// A builder for
/// [`StartCodeFixJobOutput`](crate::operation::start_code_fix_job::StartCodeFixJobOutput).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct StartCodeFixJobOutputBuilder {
    pub(crate) job_id: ::std::option::Option<::std::string::String>,
    pub(crate) status: ::std::option::Option<crate::types::CodeFixJobStatus>,
    _request_id: Option<String>,
}
impl StartCodeFixJobOutputBuilder {
    #[allow(missing_docs)] // documentation missing in model
    pub fn job_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.job_id = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_job_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.job_id = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_job_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.job_id
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn status(mut self, input: crate::types::CodeFixJobStatus) -> Self {
        self.status = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_status(mut self, input: ::std::option::Option<crate::types::CodeFixJobStatus>) -> Self {
        self.status = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_status(&self) -> &::std::option::Option<crate::types::CodeFixJobStatus> {
        &self.status
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
    /// [`StartCodeFixJobOutput`](crate::operation::start_code_fix_job::StartCodeFixJobOutput).
    pub fn build(self) -> crate::operation::start_code_fix_job::StartCodeFixJobOutput {
        crate::operation::start_code_fix_job::StartCodeFixJobOutput {
            job_id: self.job_id,
            status: self.status,
            _request_id: self._request_id,
        }
    }
}
