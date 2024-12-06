// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct CodeFixGenerationEvent {
    #[allow(missing_docs)] // documentation missing in model
    pub job_id: ::std::string::String,
    #[allow(missing_docs)] // documentation missing in model
    pub rule_id: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub detector_id: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub finding_id: ::std::option::Option<::std::string::String>,
    /// Programming Languages supported by CodeWhisperer
    pub programming_language: ::std::option::Option<crate::types::ProgrammingLanguage>,
    #[allow(missing_docs)] // documentation missing in model
    pub lines_of_code_generated: ::std::option::Option<i32>,
    #[allow(missing_docs)] // documentation missing in model
    pub chars_of_code_generated: ::std::option::Option<i32>,
}
impl CodeFixGenerationEvent {
    #[allow(missing_docs)] // documentation missing in model
    pub fn job_id(&self) -> &str {
        use std::ops::Deref;
        self.job_id.deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn rule_id(&self) -> ::std::option::Option<&str> {
        self.rule_id.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn detector_id(&self) -> ::std::option::Option<&str> {
        self.detector_id.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn finding_id(&self) -> ::std::option::Option<&str> {
        self.finding_id.as_deref()
    }

    /// Programming Languages supported by CodeWhisperer
    pub fn programming_language(&self) -> ::std::option::Option<&crate::types::ProgrammingLanguage> {
        self.programming_language.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn lines_of_code_generated(&self) -> ::std::option::Option<i32> {
        self.lines_of_code_generated
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn chars_of_code_generated(&self) -> ::std::option::Option<i32> {
        self.chars_of_code_generated
    }
}
impl CodeFixGenerationEvent {
    /// Creates a new builder-style object to manufacture
    /// [`CodeFixGenerationEvent`](crate::types::CodeFixGenerationEvent).
    pub fn builder() -> crate::types::builders::CodeFixGenerationEventBuilder {
        crate::types::builders::CodeFixGenerationEventBuilder::default()
    }
}

/// A builder for [`CodeFixGenerationEvent`](crate::types::CodeFixGenerationEvent).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct CodeFixGenerationEventBuilder {
    pub(crate) job_id: ::std::option::Option<::std::string::String>,
    pub(crate) rule_id: ::std::option::Option<::std::string::String>,
    pub(crate) detector_id: ::std::option::Option<::std::string::String>,
    pub(crate) finding_id: ::std::option::Option<::std::string::String>,
    pub(crate) programming_language: ::std::option::Option<crate::types::ProgrammingLanguage>,
    pub(crate) lines_of_code_generated: ::std::option::Option<i32>,
    pub(crate) chars_of_code_generated: ::std::option::Option<i32>,
}
impl CodeFixGenerationEventBuilder {
    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn job_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.job_id = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_job_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.job_id = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_job_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.job_id
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn rule_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.rule_id = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_rule_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.rule_id = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_rule_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.rule_id
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn detector_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.detector_id = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_detector_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.detector_id = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_detector_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.detector_id
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn finding_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.finding_id = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_finding_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.finding_id = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_finding_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.finding_id
    }

    /// Programming Languages supported by CodeWhisperer
    pub fn programming_language(mut self, input: crate::types::ProgrammingLanguage) -> Self {
        self.programming_language = ::std::option::Option::Some(input);
        self
    }

    /// Programming Languages supported by CodeWhisperer
    pub fn set_programming_language(mut self, input: ::std::option::Option<crate::types::ProgrammingLanguage>) -> Self {
        self.programming_language = input;
        self
    }

    /// Programming Languages supported by CodeWhisperer
    pub fn get_programming_language(&self) -> &::std::option::Option<crate::types::ProgrammingLanguage> {
        &self.programming_language
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn lines_of_code_generated(mut self, input: i32) -> Self {
        self.lines_of_code_generated = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_lines_of_code_generated(mut self, input: ::std::option::Option<i32>) -> Self {
        self.lines_of_code_generated = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_lines_of_code_generated(&self) -> &::std::option::Option<i32> {
        &self.lines_of_code_generated
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn chars_of_code_generated(mut self, input: i32) -> Self {
        self.chars_of_code_generated = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_chars_of_code_generated(mut self, input: ::std::option::Option<i32>) -> Self {
        self.chars_of_code_generated = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_chars_of_code_generated(&self) -> &::std::option::Option<i32> {
        &self.chars_of_code_generated
    }

    /// Consumes the builder and constructs a
    /// [`CodeFixGenerationEvent`](crate::types::CodeFixGenerationEvent). This method will fail
    /// if any of the following fields are not set:
    /// - [`job_id`](crate::types::builders::CodeFixGenerationEventBuilder::job_id)
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::types::CodeFixGenerationEvent, ::aws_smithy_types::error::operation::BuildError>
    {
        ::std::result::Result::Ok(crate::types::CodeFixGenerationEvent {
            job_id: self.job_id.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "job_id",
                    "job_id was not specified but it is required when building CodeFixGenerationEvent",
                )
            })?,
            rule_id: self.rule_id,
            detector_id: self.detector_id,
            finding_id: self.finding_id,
            programming_language: self.programming_language,
            lines_of_code_generated: self.lines_of_code_generated,
            chars_of_code_generated: self.chars_of_code_generated,
        })
    }
}
