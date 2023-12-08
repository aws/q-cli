// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq)]
pub struct CreateUploadUrlInput {
    #[allow(missing_docs)] // documentation missing in model
    pub content_md5: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub content_checksum: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub content_checksum_type: ::std::option::Option<crate::types::ContentChecksumType>,
    #[allow(missing_docs)] // documentation missing in model
    pub content_length: ::std::option::Option<i64>,
    #[allow(missing_docs)] // documentation missing in model
    pub artifact_type: ::std::option::Option<crate::types::ArtifactType>,
    /// Upload Intent
    pub upload_intent: ::std::option::Option<crate::types::UploadIntent>,
    #[allow(missing_docs)] // documentation missing in model
    pub upload_context: ::std::option::Option<crate::types::UploadContext>,
}
impl CreateUploadUrlInput {
    #[allow(missing_docs)] // documentation missing in model
    pub fn content_md5(&self) -> ::std::option::Option<&str> {
        self.content_md5.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn content_checksum(&self) -> ::std::option::Option<&str> {
        self.content_checksum.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn content_checksum_type(&self) -> ::std::option::Option<&crate::types::ContentChecksumType> {
        self.content_checksum_type.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn content_length(&self) -> ::std::option::Option<i64> {
        self.content_length
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn artifact_type(&self) -> ::std::option::Option<&crate::types::ArtifactType> {
        self.artifact_type.as_ref()
    }

    /// Upload Intent
    pub fn upload_intent(&self) -> ::std::option::Option<&crate::types::UploadIntent> {
        self.upload_intent.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn upload_context(&self) -> ::std::option::Option<&crate::types::UploadContext> {
        self.upload_context.as_ref()
    }
}
impl ::std::fmt::Debug for CreateUploadUrlInput {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut formatter = f.debug_struct("CreateUploadUrlInput");
        formatter.field("content_md5", &"*** Sensitive Data Redacted ***");
        formatter.field("content_checksum", &"*** Sensitive Data Redacted ***");
        formatter.field("content_checksum_type", &self.content_checksum_type);
        formatter.field("content_length", &self.content_length);
        formatter.field("artifact_type", &self.artifact_type);
        formatter.field("upload_intent", &self.upload_intent);
        formatter.field("upload_context", &self.upload_context);
        formatter.finish()
    }
}
impl CreateUploadUrlInput {
    /// Creates a new builder-style object to manufacture
    /// [`CreateUploadUrlInput`](crate::operation::create_upload_url::CreateUploadUrlInput).
    pub fn builder() -> crate::operation::create_upload_url::builders::CreateUploadUrlInputBuilder {
        crate::operation::create_upload_url::builders::CreateUploadUrlInputBuilder::default()
    }
}

/// A builder for
/// [`CreateUploadUrlInput`](crate::operation::create_upload_url::CreateUploadUrlInput).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default)]
pub struct CreateUploadUrlInputBuilder {
    pub(crate) content_md5: ::std::option::Option<::std::string::String>,
    pub(crate) content_checksum: ::std::option::Option<::std::string::String>,
    pub(crate) content_checksum_type: ::std::option::Option<crate::types::ContentChecksumType>,
    pub(crate) content_length: ::std::option::Option<i64>,
    pub(crate) artifact_type: ::std::option::Option<crate::types::ArtifactType>,
    pub(crate) upload_intent: ::std::option::Option<crate::types::UploadIntent>,
    pub(crate) upload_context: ::std::option::Option<crate::types::UploadContext>,
}
impl CreateUploadUrlInputBuilder {
    #[allow(missing_docs)] // documentation missing in model
    pub fn content_md5(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.content_md5 = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_content_md5(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.content_md5 = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_content_md5(&self) -> &::std::option::Option<::std::string::String> {
        &self.content_md5
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn content_checksum(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.content_checksum = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_content_checksum(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.content_checksum = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_content_checksum(&self) -> &::std::option::Option<::std::string::String> {
        &self.content_checksum
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn content_checksum_type(mut self, input: crate::types::ContentChecksumType) -> Self {
        self.content_checksum_type = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_content_checksum_type(
        mut self,
        input: ::std::option::Option<crate::types::ContentChecksumType>,
    ) -> Self {
        self.content_checksum_type = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_content_checksum_type(&self) -> &::std::option::Option<crate::types::ContentChecksumType> {
        &self.content_checksum_type
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn content_length(mut self, input: i64) -> Self {
        self.content_length = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_content_length(mut self, input: ::std::option::Option<i64>) -> Self {
        self.content_length = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_content_length(&self) -> &::std::option::Option<i64> {
        &self.content_length
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn artifact_type(mut self, input: crate::types::ArtifactType) -> Self {
        self.artifact_type = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_artifact_type(mut self, input: ::std::option::Option<crate::types::ArtifactType>) -> Self {
        self.artifact_type = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_artifact_type(&self) -> &::std::option::Option<crate::types::ArtifactType> {
        &self.artifact_type
    }

    /// Upload Intent
    pub fn upload_intent(mut self, input: crate::types::UploadIntent) -> Self {
        self.upload_intent = ::std::option::Option::Some(input);
        self
    }

    /// Upload Intent
    pub fn set_upload_intent(mut self, input: ::std::option::Option<crate::types::UploadIntent>) -> Self {
        self.upload_intent = input;
        self
    }

    /// Upload Intent
    pub fn get_upload_intent(&self) -> &::std::option::Option<crate::types::UploadIntent> {
        &self.upload_intent
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn upload_context(mut self, input: crate::types::UploadContext) -> Self {
        self.upload_context = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_upload_context(mut self, input: ::std::option::Option<crate::types::UploadContext>) -> Self {
        self.upload_context = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_upload_context(&self) -> &::std::option::Option<crate::types::UploadContext> {
        &self.upload_context
    }

    /// Consumes the builder and constructs a
    /// [`CreateUploadUrlInput`](crate::operation::create_upload_url::CreateUploadUrlInput).
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::create_upload_url::CreateUploadUrlInput,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(crate::operation::create_upload_url::CreateUploadUrlInput {
            content_md5: self.content_md5,
            content_checksum: self.content_checksum,
            content_checksum_type: self.content_checksum_type,
            content_length: self.content_length,
            artifact_type: self.artifact_type,
            upload_intent: self.upload_intent,
            upload_context: self.upload_context,
        })
    }
}
impl ::std::fmt::Debug for CreateUploadUrlInputBuilder {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut formatter = f.debug_struct("CreateUploadUrlInputBuilder");
        formatter.field("content_md5", &"*** Sensitive Data Redacted ***");
        formatter.field("content_checksum", &"*** Sensitive Data Redacted ***");
        formatter.field("content_checksum_type", &self.content_checksum_type);
        formatter.field("content_length", &self.content_length);
        formatter.field("artifact_type", &self.artifact_type);
        formatter.field("upload_intent", &self.upload_intent);
        formatter.field("upload_context", &self.upload_context);
        formatter.finish()
    }
}
