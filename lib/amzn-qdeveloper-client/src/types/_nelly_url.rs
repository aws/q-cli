// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq)]
pub struct NellyUrl {
    #[allow(missing_docs)] // documentation missing in model
    pub id: ::std::string::String,
    #[allow(missing_docs)] // documentation missing in model
    pub url: ::std::string::String,
    #[allow(missing_docs)] // documentation missing in model
    pub title: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub inline_text: ::std::option::Option<::std::string::String>,
}
impl NellyUrl {
    #[allow(missing_docs)] // documentation missing in model
    pub fn id(&self) -> &str {
        use std::ops::Deref;
        self.id.deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn url(&self) -> &str {
        use std::ops::Deref;
        self.url.deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn title(&self) -> ::std::option::Option<&str> {
        self.title.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn inline_text(&self) -> ::std::option::Option<&str> {
        self.inline_text.as_deref()
    }
}
impl ::std::fmt::Debug for NellyUrl {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut formatter = f.debug_struct("NellyUrl");
        formatter.field("id", &"*** Sensitive Data Redacted ***");
        formatter.field("url", &"*** Sensitive Data Redacted ***");
        formatter.field("title", &"*** Sensitive Data Redacted ***");
        formatter.field("inline_text", &"*** Sensitive Data Redacted ***");
        formatter.finish()
    }
}
impl NellyUrl {
    /// Creates a new builder-style object to manufacture [`NellyUrl`](crate::types::NellyUrl).
    pub fn builder() -> crate::types::builders::NellyUrlBuilder {
        crate::types::builders::NellyUrlBuilder::default()
    }
}

/// A builder for [`NellyUrl`](crate::types::NellyUrl).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default)]
#[non_exhaustive]
pub struct NellyUrlBuilder {
    pub(crate) id: ::std::option::Option<::std::string::String>,
    pub(crate) url: ::std::option::Option<::std::string::String>,
    pub(crate) title: ::std::option::Option<::std::string::String>,
    pub(crate) inline_text: ::std::option::Option<::std::string::String>,
}
impl NellyUrlBuilder {
    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.id = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.id = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.id
    }

    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn url(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.url = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_url(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.url = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_url(&self) -> &::std::option::Option<::std::string::String> {
        &self.url
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn title(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.title = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_title(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.title = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_title(&self) -> &::std::option::Option<::std::string::String> {
        &self.title
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn inline_text(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inline_text = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_inline_text(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inline_text = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_inline_text(&self) -> &::std::option::Option<::std::string::String> {
        &self.inline_text
    }

    /// Consumes the builder and constructs a [`NellyUrl`](crate::types::NellyUrl).
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](crate::types::builders::NellyUrlBuilder::id)
    /// - [`url`](crate::types::builders::NellyUrlBuilder::url)
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::types::NellyUrl, ::aws_smithy_types::error::operation::BuildError> {
        ::std::result::Result::Ok(crate::types::NellyUrl {
            id: self.id.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "id",
                    "id was not specified but it is required when building NellyUrl",
                )
            })?,
            url: self.url.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "url",
                    "url was not specified but it is required when building NellyUrl",
                )
            })?,
            title: self.title,
            inline_text: self.inline_text,
        })
    }
}
impl ::std::fmt::Debug for NellyUrlBuilder {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut formatter = f.debug_struct("NellyUrlBuilder");
        formatter.field("id", &"*** Sensitive Data Redacted ***");
        formatter.field("url", &"*** Sensitive Data Redacted ***");
        formatter.field("title", &"*** Sensitive Data Redacted ***");
        formatter.field("inline_text", &"*** Sensitive Data Redacted ***");
        formatter.finish()
    }
}
