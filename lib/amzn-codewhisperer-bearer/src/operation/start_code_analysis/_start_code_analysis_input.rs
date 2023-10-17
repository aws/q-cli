// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct StartCodeAnalysisInput {
    #[allow(missing_docs)] // documentation missing in model
    pub artifacts:
        ::std::option::Option<::std::collections::HashMap<crate::types::ArtifactType, ::std::string::String>>,
    #[allow(missing_docs)] // documentation missing in model
    pub programming_language: ::std::option::Option<crate::types::ProgrammingLanguage>,
    #[allow(missing_docs)] // documentation missing in model
    pub client_token: ::std::option::Option<::std::string::String>,
}
impl StartCodeAnalysisInput {
    #[allow(missing_docs)] // documentation missing in model
    pub fn artifacts(
        &self,
    ) -> ::std::option::Option<&::std::collections::HashMap<crate::types::ArtifactType, ::std::string::String>> {
        self.artifacts.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn programming_language(&self) -> ::std::option::Option<&crate::types::ProgrammingLanguage> {
        self.programming_language.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn client_token(&self) -> ::std::option::Option<&str> {
        self.client_token.as_deref()
    }
}
impl StartCodeAnalysisInput {
    /// Creates a new builder-style object to manufacture
    /// [`StartCodeAnalysisInput`](crate::operation::start_code_analysis::StartCodeAnalysisInput).
    pub fn builder() -> crate::operation::start_code_analysis::builders::StartCodeAnalysisInputBuilder {
        crate::operation::start_code_analysis::builders::StartCodeAnalysisInputBuilder::default()
    }
}

/// A builder for
/// [`StartCodeAnalysisInput`](crate::operation::start_code_analysis::StartCodeAnalysisInput).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct StartCodeAnalysisInputBuilder {
    pub(crate) artifacts:
        ::std::option::Option<::std::collections::HashMap<crate::types::ArtifactType, ::std::string::String>>,
    pub(crate) programming_language: ::std::option::Option<crate::types::ProgrammingLanguage>,
    pub(crate) client_token: ::std::option::Option<::std::string::String>,
}
impl StartCodeAnalysisInputBuilder {
    /// Adds a key-value pair to `artifacts`.
    ///
    /// To override the contents of this collection use [`set_artifacts`](Self::set_artifacts).
    pub fn artifacts(
        mut self,
        k: crate::types::ArtifactType,
        v: impl ::std::convert::Into<::std::string::String>,
    ) -> Self {
        let mut hash_map = self.artifacts.unwrap_or_default();
        hash_map.insert(k, v.into());
        self.artifacts = ::std::option::Option::Some(hash_map);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_artifacts(
        mut self,
        input: ::std::option::Option<::std::collections::HashMap<crate::types::ArtifactType, ::std::string::String>>,
    ) -> Self {
        self.artifacts = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_artifacts(
        &self,
    ) -> &::std::option::Option<::std::collections::HashMap<crate::types::ArtifactType, ::std::string::String>> {
        &self.artifacts
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn programming_language(mut self, input: crate::types::ProgrammingLanguage) -> Self {
        self.programming_language = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_programming_language(mut self, input: ::std::option::Option<crate::types::ProgrammingLanguage>) -> Self {
        self.programming_language = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_programming_language(&self) -> &::std::option::Option<crate::types::ProgrammingLanguage> {
        &self.programming_language
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn client_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.client_token = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_client_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.client_token = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_client_token(&self) -> &::std::option::Option<::std::string::String> {
        &self.client_token
    }

    /// Consumes the builder and constructs a
    /// [`StartCodeAnalysisInput`](crate::operation::start_code_analysis::StartCodeAnalysisInput).
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::start_code_analysis::StartCodeAnalysisInput,
        ::aws_smithy_http::operation::error::BuildError,
    > {
        ::std::result::Result::Ok(crate::operation::start_code_analysis::StartCodeAnalysisInput {
            artifacts: self.artifacts,
            programming_language: self.programming_language,
            client_token: self.client_token,
        })
    }
}
