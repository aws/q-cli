// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Structure to represent ExportResultArchive response.
#[non_exhaustive]
#[derive(::std::fmt::Debug)]
pub struct ExportResultArchiveOutput {
    /// Response Stream
    pub body: crate::event_receiver::EventReceiver<
        crate::types::ResultArchiveStream,
        crate::types::error::ResultArchiveStreamError,
    >,
    _request_id: Option<String>,
}
impl ExportResultArchiveOutput {
    /// Response Stream
    pub fn body(
        &self,
    ) -> &crate::event_receiver::EventReceiver<
        crate::types::ResultArchiveStream,
        crate::types::error::ResultArchiveStreamError,
    > {
        &self.body
    }
}
impl ::aws_types::request_id::RequestId for ExportResultArchiveOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl ExportResultArchiveOutput {
    /// Creates a new builder-style object to manufacture
    /// [`ExportResultArchiveOutput`](crate::operation::export_result_archive::ExportResultArchiveOutput).
    pub fn builder() -> crate::operation::export_result_archive::builders::ExportResultArchiveOutputBuilder {
        crate::operation::export_result_archive::builders::ExportResultArchiveOutputBuilder::default()
    }
}

/// A builder for
/// [`ExportResultArchiveOutput`](crate::operation::export_result_archive::ExportResultArchiveOutput).
#[derive(::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct ExportResultArchiveOutputBuilder {
    pub(crate) body: ::std::option::Option<
        crate::event_receiver::EventReceiver<
            crate::types::ResultArchiveStream,
            crate::types::error::ResultArchiveStreamError,
        >,
    >,
    _request_id: Option<String>,
}
impl ExportResultArchiveOutputBuilder {
    /// Response Stream
    /// This field is required.
    pub fn body(
        mut self,
        input: crate::event_receiver::EventReceiver<
            crate::types::ResultArchiveStream,
            crate::types::error::ResultArchiveStreamError,
        >,
    ) -> Self {
        self.body = ::std::option::Option::Some(input);
        self
    }

    /// Response Stream
    pub fn set_body(
        mut self,
        input: ::std::option::Option<
            crate::event_receiver::EventReceiver<
                crate::types::ResultArchiveStream,
                crate::types::error::ResultArchiveStreamError,
            >,
        >,
    ) -> Self {
        self.body = input;
        self
    }

    /// Response Stream
    pub fn get_body(
        &self,
    ) -> &::std::option::Option<
        crate::event_receiver::EventReceiver<
            crate::types::ResultArchiveStream,
            crate::types::error::ResultArchiveStreamError,
        >,
    > {
        &self.body
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
    /// [`ExportResultArchiveOutput`](crate::operation::export_result_archive::ExportResultArchiveOutput).
    /// This method will fail if any of the following fields are not set:
    /// - [`body`](crate::operation::export_result_archive::builders::ExportResultArchiveOutputBuilder::body)
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::export_result_archive::ExportResultArchiveOutput,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(crate::operation::export_result_archive::ExportResultArchiveOutput {
            body: self.body.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "body",
                    "body was not specified but it is required when building ExportResultArchiveOutput",
                )
            })?,
            _request_id: self._request_id,
        })
    }
}
