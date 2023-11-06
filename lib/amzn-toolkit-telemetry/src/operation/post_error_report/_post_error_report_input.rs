// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct PostErrorReportInput {
    #[allow(missing_docs)] // documentation missing in model
    pub aws_product: ::std::option::Option<crate::types::AwsProduct>,
    #[allow(missing_docs)] // documentation missing in model
    pub aws_product_version: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub metadata: ::std::option::Option<::std::vec::Vec<crate::types::MetadataEntry>>,
    #[allow(missing_docs)] // documentation missing in model
    pub userdata: ::std::option::Option<crate::types::Userdata>,
    #[allow(missing_docs)] // documentation missing in model
    pub error_details: ::std::option::Option<crate::types::ErrorDetails>,
}
impl PostErrorReportInput {
    #[allow(missing_docs)] // documentation missing in model
    pub fn aws_product(&self) -> ::std::option::Option<&crate::types::AwsProduct> {
        self.aws_product.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn aws_product_version(&self) -> ::std::option::Option<&str> {
        self.aws_product_version.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn metadata(&self) -> ::std::option::Option<&[crate::types::MetadataEntry]> {
        self.metadata.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn userdata(&self) -> ::std::option::Option<&crate::types::Userdata> {
        self.userdata.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn error_details(&self) -> ::std::option::Option<&crate::types::ErrorDetails> {
        self.error_details.as_ref()
    }
}
impl PostErrorReportInput {
    /// Creates a new builder-style object to manufacture
    /// [`PostErrorReportInput`](crate::operation::post_error_report::PostErrorReportInput).
    pub fn builder() -> crate::operation::post_error_report::builders::PostErrorReportInputBuilder {
        crate::operation::post_error_report::builders::PostErrorReportInputBuilder::default()
    }
}

/// A builder for
/// [`PostErrorReportInput`](crate::operation::post_error_report::PostErrorReportInput).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct PostErrorReportInputBuilder {
    pub(crate) aws_product: ::std::option::Option<crate::types::AwsProduct>,
    pub(crate) aws_product_version: ::std::option::Option<::std::string::String>,
    pub(crate) metadata: ::std::option::Option<::std::vec::Vec<crate::types::MetadataEntry>>,
    pub(crate) userdata: ::std::option::Option<crate::types::Userdata>,
    pub(crate) error_details: ::std::option::Option<crate::types::ErrorDetails>,
}
impl PostErrorReportInputBuilder {
    #[allow(missing_docs)] // documentation missing in model
    pub fn aws_product(mut self, input: crate::types::AwsProduct) -> Self {
        self.aws_product = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_aws_product(mut self, input: ::std::option::Option<crate::types::AwsProduct>) -> Self {
        self.aws_product = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_aws_product(&self) -> &::std::option::Option<crate::types::AwsProduct> {
        &self.aws_product
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn aws_product_version(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.aws_product_version = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_aws_product_version(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.aws_product_version = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_aws_product_version(&self) -> &::std::option::Option<::std::string::String> {
        &self.aws_product_version
    }

    /// Appends an item to `metadata`.
    ///
    /// To override the contents of this collection use [`set_metadata`](Self::set_metadata).
    pub fn metadata(mut self, input: crate::types::MetadataEntry) -> Self {
        let mut v = self.metadata.unwrap_or_default();
        v.push(input);
        self.metadata = ::std::option::Option::Some(v);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_metadata(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::MetadataEntry>>) -> Self {
        self.metadata = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_metadata(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::MetadataEntry>> {
        &self.metadata
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn userdata(mut self, input: crate::types::Userdata) -> Self {
        self.userdata = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_userdata(mut self, input: ::std::option::Option<crate::types::Userdata>) -> Self {
        self.userdata = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_userdata(&self) -> &::std::option::Option<crate::types::Userdata> {
        &self.userdata
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn error_details(mut self, input: crate::types::ErrorDetails) -> Self {
        self.error_details = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_error_details(mut self, input: ::std::option::Option<crate::types::ErrorDetails>) -> Self {
        self.error_details = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_error_details(&self) -> &::std::option::Option<crate::types::ErrorDetails> {
        &self.error_details
    }

    /// Consumes the builder and constructs a
    /// [`PostErrorReportInput`](crate::operation::post_error_report::PostErrorReportInput).
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::post_error_report::PostErrorReportInput,
        ::aws_smithy_http::operation::error::BuildError,
    > {
        ::std::result::Result::Ok(crate::operation::post_error_report::PostErrorReportInput {
            aws_product: self.aws_product,
            aws_product_version: self.aws_product_version,
            metadata: self.metadata,
            userdata: self.userdata,
            error_details: self.error_details,
        })
    }
}
