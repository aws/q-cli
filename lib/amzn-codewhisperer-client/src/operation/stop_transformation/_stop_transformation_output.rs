// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Structure to represent stop code transformation response.
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct StopTransformationOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub transformation_status: crate::types::TransformationStatus,
    _request_id: Option<String>,
}
impl StopTransformationOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub fn transformation_status(&self) -> &crate::types::TransformationStatus {
        &self.transformation_status
    }
}
impl ::aws_types::request_id::RequestId for StopTransformationOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl StopTransformationOutput {
    /// Creates a new builder-style object to manufacture
    /// [`StopTransformationOutput`](crate::operation::stop_transformation::StopTransformationOutput).
    pub fn builder() -> crate::operation::stop_transformation::builders::StopTransformationOutputBuilder {
        crate::operation::stop_transformation::builders::StopTransformationOutputBuilder::default()
    }
}

/// A builder for
/// [`StopTransformationOutput`](crate::operation::stop_transformation::StopTransformationOutput).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct StopTransformationOutputBuilder {
    pub(crate) transformation_status: ::std::option::Option<crate::types::TransformationStatus>,
    _request_id: Option<String>,
}
impl StopTransformationOutputBuilder {
    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn transformation_status(mut self, input: crate::types::TransformationStatus) -> Self {
        self.transformation_status = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_transformation_status(
        mut self,
        input: ::std::option::Option<crate::types::TransformationStatus>,
    ) -> Self {
        self.transformation_status = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_transformation_status(&self) -> &::std::option::Option<crate::types::TransformationStatus> {
        &self.transformation_status
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
    /// [`StopTransformationOutput`](crate::operation::stop_transformation::StopTransformationOutput).
    /// This method will fail if any of the following fields are not set:
    /// - [`transformation_status`](crate::operation::stop_transformation::builders::StopTransformationOutputBuilder::transformation_status)
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::stop_transformation::StopTransformationOutput,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(crate::operation::stop_transformation::StopTransformationOutput {
            transformation_status: self.transformation_status.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "transformation_status",
                    "transformation_status was not specified but it is required when building StopTransformationOutput",
                )
            })?,
            _request_id: self._request_id,
        })
    }
}
