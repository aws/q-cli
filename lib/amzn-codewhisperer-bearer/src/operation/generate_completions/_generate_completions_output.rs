// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct GenerateCompletionsOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub completions: ::std::option::Option<::std::vec::Vec<crate::types::Completion>>,
    #[allow(missing_docs)] // documentation missing in model
    pub next_token: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl GenerateCompletionsOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub fn completions(&self) -> ::std::option::Option<&[crate::types::Completion]> {
        self.completions.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn next_token(&self) -> ::std::option::Option<&str> {
        self.next_token.as_deref()
    }
}
impl ::aws_http::request_id::RequestId for GenerateCompletionsOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl GenerateCompletionsOutput {
    /// Creates a new builder-style object to manufacture
    /// [`GenerateCompletionsOutput`](crate::operation::generate_completions::GenerateCompletionsOutput).
    ///
    pub fn builder() -> crate::operation::generate_completions::builders::GenerateCompletionsOutputBuilder {
        crate::operation::generate_completions::builders::GenerateCompletionsOutputBuilder::default()
    }
}

/// A builder for
/// [`GenerateCompletionsOutput`](crate::operation::generate_completions::GenerateCompletionsOutput).
///
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct GenerateCompletionsOutputBuilder {
    pub(crate) completions: ::std::option::Option<::std::vec::Vec<crate::types::Completion>>,
    pub(crate) next_token: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl GenerateCompletionsOutputBuilder {
    /// Appends an item to `completions`.
    ///
    /// To override the contents of this collection use [`set_completions`](Self::set_completions).
    pub fn completions(mut self, input: crate::types::Completion) -> Self {
        let mut v = self.completions.unwrap_or_default();
        v.push(input);
        self.completions = ::std::option::Option::Some(v);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_completions(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Completion>>) -> Self {
        self.completions = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_completions(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Completion>> {
        &self.completions
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn next_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.next_token = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_next_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.next_token = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_next_token(&self) -> &::std::option::Option<::std::string::String> {
        &self.next_token
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
    /// [`GenerateCompletionsOutput`](crate::operation::generate_completions::GenerateCompletionsOutput).
    ///
    pub fn build(self) -> crate::operation::generate_completions::GenerateCompletionsOutput {
        crate::operation::generate_completions::GenerateCompletionsOutput {
            completions: self.completions,
            next_token: self.next_token,
            _request_id: self._request_id,
        }
    }
}
