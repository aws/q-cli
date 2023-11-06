// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct PostFeedbackInput {
    #[allow(missing_docs)] // documentation missing in model
    pub aws_product: ::std::option::Option<crate::types::AwsProduct>,
    #[allow(missing_docs)] // documentation missing in model
    pub aws_product_version: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub os: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub os_version: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub parent_product: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub parent_product_version: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub metadata: ::std::option::Option<::std::vec::Vec<crate::types::MetadataEntry>>,
    #[allow(missing_docs)] // documentation missing in model
    pub sentiment: ::std::option::Option<crate::types::Sentiment>,
    #[allow(missing_docs)] // documentation missing in model
    pub comment: ::std::option::Option<::std::string::String>,
}
impl PostFeedbackInput {
    #[allow(missing_docs)] // documentation missing in model
    pub fn aws_product(&self) -> ::std::option::Option<&crate::types::AwsProduct> {
        self.aws_product.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn aws_product_version(&self) -> ::std::option::Option<&str> {
        self.aws_product_version.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn os(&self) -> ::std::option::Option<&str> {
        self.os.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn os_version(&self) -> ::std::option::Option<&str> {
        self.os_version.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn parent_product(&self) -> ::std::option::Option<&str> {
        self.parent_product.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn parent_product_version(&self) -> ::std::option::Option<&str> {
        self.parent_product_version.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn metadata(&self) -> ::std::option::Option<&[crate::types::MetadataEntry]> {
        self.metadata.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn sentiment(&self) -> ::std::option::Option<&crate::types::Sentiment> {
        self.sentiment.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn comment(&self) -> ::std::option::Option<&str> {
        self.comment.as_deref()
    }
}
impl PostFeedbackInput {
    /// Creates a new builder-style object to manufacture
    /// [`PostFeedbackInput`](crate::operation::post_feedback::PostFeedbackInput).
    pub fn builder() -> crate::operation::post_feedback::builders::PostFeedbackInputBuilder {
        crate::operation::post_feedback::builders::PostFeedbackInputBuilder::default()
    }
}

/// A builder for [`PostFeedbackInput`](crate::operation::post_feedback::PostFeedbackInput).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct PostFeedbackInputBuilder {
    pub(crate) aws_product: ::std::option::Option<crate::types::AwsProduct>,
    pub(crate) aws_product_version: ::std::option::Option<::std::string::String>,
    pub(crate) os: ::std::option::Option<::std::string::String>,
    pub(crate) os_version: ::std::option::Option<::std::string::String>,
    pub(crate) parent_product: ::std::option::Option<::std::string::String>,
    pub(crate) parent_product_version: ::std::option::Option<::std::string::String>,
    pub(crate) metadata: ::std::option::Option<::std::vec::Vec<crate::types::MetadataEntry>>,
    pub(crate) sentiment: ::std::option::Option<crate::types::Sentiment>,
    pub(crate) comment: ::std::option::Option<::std::string::String>,
}
impl PostFeedbackInputBuilder {
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

    #[allow(missing_docs)] // documentation missing in model
    pub fn os(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.os = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_os(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.os = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_os(&self) -> &::std::option::Option<::std::string::String> {
        &self.os
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn os_version(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.os_version = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_os_version(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.os_version = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_os_version(&self) -> &::std::option::Option<::std::string::String> {
        &self.os_version
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn parent_product(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.parent_product = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_parent_product(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.parent_product = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_parent_product(&self) -> &::std::option::Option<::std::string::String> {
        &self.parent_product
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn parent_product_version(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.parent_product_version = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_parent_product_version(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.parent_product_version = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_parent_product_version(&self) -> &::std::option::Option<::std::string::String> {
        &self.parent_product_version
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
    pub fn sentiment(mut self, input: crate::types::Sentiment) -> Self {
        self.sentiment = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_sentiment(mut self, input: ::std::option::Option<crate::types::Sentiment>) -> Self {
        self.sentiment = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_sentiment(&self) -> &::std::option::Option<crate::types::Sentiment> {
        &self.sentiment
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn comment(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.comment = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_comment(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.comment = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_comment(&self) -> &::std::option::Option<::std::string::String> {
        &self.comment
    }

    /// Consumes the builder and constructs a
    /// [`PostFeedbackInput`](crate::operation::post_feedback::PostFeedbackInput).
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::post_feedback::PostFeedbackInput,
        ::aws_smithy_http::operation::error::BuildError,
    > {
        ::std::result::Result::Ok(crate::operation::post_feedback::PostFeedbackInput {
            aws_product: self.aws_product,
            aws_product_version: self.aws_product_version,
            os: self.os,
            os_version: self.os_version,
            parent_product: self.parent_product,
            parent_product_version: self.parent_product_version,
            metadata: self.metadata,
            sentiment: self.sentiment,
            comment: self.comment,
        })
    }
}
