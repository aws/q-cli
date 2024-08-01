// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Transformation export context
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct TransformationExportContext {
    #[allow(missing_docs)] // documentation missing in model
    pub download_artifact_id: ::std::string::String,
    #[allow(missing_docs)] // documentation missing in model
    pub download_artifact_type: crate::types::TransformationDownloadArtifactType,
}
impl TransformationExportContext {
    #[allow(missing_docs)] // documentation missing in model
    pub fn download_artifact_id(&self) -> &str {
        use std::ops::Deref;
        self.download_artifact_id.deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn download_artifact_type(&self) -> &crate::types::TransformationDownloadArtifactType {
        &self.download_artifact_type
    }
}
impl TransformationExportContext {
    /// Creates a new builder-style object to manufacture
    /// [`TransformationExportContext`](crate::types::TransformationExportContext).
    pub fn builder() -> crate::types::builders::TransformationExportContextBuilder {
        crate::types::builders::TransformationExportContextBuilder::default()
    }
}

/// A builder for [`TransformationExportContext`](crate::types::TransformationExportContext).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct TransformationExportContextBuilder {
    pub(crate) download_artifact_id: ::std::option::Option<::std::string::String>,
    pub(crate) download_artifact_type: ::std::option::Option<crate::types::TransformationDownloadArtifactType>,
}
impl TransformationExportContextBuilder {
    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn download_artifact_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.download_artifact_id = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_download_artifact_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.download_artifact_id = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_download_artifact_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.download_artifact_id
    }

    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn download_artifact_type(mut self, input: crate::types::TransformationDownloadArtifactType) -> Self {
        self.download_artifact_type = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_download_artifact_type(
        mut self,
        input: ::std::option::Option<crate::types::TransformationDownloadArtifactType>,
    ) -> Self {
        self.download_artifact_type = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_download_artifact_type(
        &self,
    ) -> &::std::option::Option<crate::types::TransformationDownloadArtifactType> {
        &self.download_artifact_type
    }

    /// Consumes the builder and constructs a
    /// [`TransformationExportContext`](crate::types::TransformationExportContext). This method
    /// will fail if any of the following fields are not set:
    /// - [`download_artifact_id`](crate::types::builders::TransformationExportContextBuilder::download_artifact_id)
    /// - [`download_artifact_type`](crate::types::builders::TransformationExportContextBuilder::download_artifact_type)
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::types::TransformationExportContext,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(crate::types::TransformationExportContext {
            download_artifact_id: self.download_artifact_id.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "download_artifact_id",
                    "download_artifact_id was not specified but it is required when building TransformationExportContext",
                )
            })?,
            download_artifact_type: self.download_artifact_type.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "download_artifact_type",
                    "download_artifact_type was not specified but it is required when building TransformationExportContext",
                )
            })?,
        })
    }
}
