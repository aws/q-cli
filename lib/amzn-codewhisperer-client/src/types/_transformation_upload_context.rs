// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct TransformationUploadContext {
    /// Identifier for the Transformation Job
    pub job_id: ::std::string::String,
    #[allow(missing_docs)] // documentation missing in model
    pub upload_artifact_type: crate::types::TransformationUploadArtifactType,
}
impl TransformationUploadContext {
    /// Identifier for the Transformation Job
    pub fn job_id(&self) -> &str {
        use std::ops::Deref;
        self.job_id.deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn upload_artifact_type(&self) -> &crate::types::TransformationUploadArtifactType {
        &self.upload_artifact_type
    }
}
impl TransformationUploadContext {
    /// Creates a new builder-style object to manufacture
    /// [`TransformationUploadContext`](crate::types::TransformationUploadContext).
    pub fn builder() -> crate::types::builders::TransformationUploadContextBuilder {
        crate::types::builders::TransformationUploadContextBuilder::default()
    }
}

/// A builder for [`TransformationUploadContext`](crate::types::TransformationUploadContext).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct TransformationUploadContextBuilder {
    pub(crate) job_id: ::std::option::Option<::std::string::String>,
    pub(crate) upload_artifact_type: ::std::option::Option<crate::types::TransformationUploadArtifactType>,
}
impl TransformationUploadContextBuilder {
    /// Identifier for the Transformation Job
    /// This field is required.
    pub fn job_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.job_id = ::std::option::Option::Some(input.into());
        self
    }

    /// Identifier for the Transformation Job
    pub fn set_job_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.job_id = input;
        self
    }

    /// Identifier for the Transformation Job
    pub fn get_job_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.job_id
    }

    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn upload_artifact_type(mut self, input: crate::types::TransformationUploadArtifactType) -> Self {
        self.upload_artifact_type = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_upload_artifact_type(
        mut self,
        input: ::std::option::Option<crate::types::TransformationUploadArtifactType>,
    ) -> Self {
        self.upload_artifact_type = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_upload_artifact_type(&self) -> &::std::option::Option<crate::types::TransformationUploadArtifactType> {
        &self.upload_artifact_type
    }

    /// Consumes the builder and constructs a
    /// [`TransformationUploadContext`](crate::types::TransformationUploadContext). This method
    /// will fail if any of the following fields are not set:
    /// - [`job_id`](crate::types::builders::TransformationUploadContextBuilder::job_id)
    /// - [`upload_artifact_type`](crate::types::builders::TransformationUploadContextBuilder::upload_artifact_type)
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::types::TransformationUploadContext,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(crate::types::TransformationUploadContext {
            job_id: self.job_id.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "job_id",
                    "job_id was not specified but it is required when building TransformationUploadContext",
                )
            })?,
            upload_artifact_type: self.upload_artifact_type.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "upload_artifact_type",
                    "upload_artifact_type was not specified but it is required when building TransformationUploadContext",
                )
            })?,
        })
    }
}
