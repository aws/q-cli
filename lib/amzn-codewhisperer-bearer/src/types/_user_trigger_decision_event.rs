// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq)]
pub struct UserTriggerDecisionEvent {
    #[allow(missing_docs)] // documentation missing in model
    pub session_id: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub request_id: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub customization_arn: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub programming_language: ::std::option::Option<crate::types::ProgrammingLanguage>,
    #[allow(missing_docs)] // documentation missing in model
    pub completion_type: ::std::option::Option<crate::types::CompletionType>,
    #[allow(missing_docs)] // documentation missing in model
    pub suggestion_state: ::std::option::Option<crate::types::SuggestionState>,
    #[allow(missing_docs)] // documentation missing in model
    pub recommendation_latency_milliseconds: f64,
    #[allow(missing_docs)] // documentation missing in model
    pub timestamp: ::std::option::Option<::aws_smithy_types::DateTime>,
    #[allow(missing_docs)] // documentation missing in model
    pub suggestion_reference_count: i32,
    #[allow(missing_docs)] // documentation missing in model
    pub generated_line: i32,
}
impl UserTriggerDecisionEvent {
    #[allow(missing_docs)] // documentation missing in model
    pub fn session_id(&self) -> ::std::option::Option<&str> {
        self.session_id.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn request_id(&self) -> ::std::option::Option<&str> {
        self.request_id.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn customization_arn(&self) -> ::std::option::Option<&str> {
        self.customization_arn.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn programming_language(&self) -> ::std::option::Option<&crate::types::ProgrammingLanguage> {
        self.programming_language.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn completion_type(&self) -> ::std::option::Option<&crate::types::CompletionType> {
        self.completion_type.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn suggestion_state(&self) -> ::std::option::Option<&crate::types::SuggestionState> {
        self.suggestion_state.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn recommendation_latency_milliseconds(&self) -> f64 {
        self.recommendation_latency_milliseconds
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn timestamp(&self) -> ::std::option::Option<&::aws_smithy_types::DateTime> {
        self.timestamp.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn suggestion_reference_count(&self) -> i32 {
        self.suggestion_reference_count
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn generated_line(&self) -> i32 {
        self.generated_line
    }
}
impl ::std::fmt::Debug for UserTriggerDecisionEvent {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut formatter = f.debug_struct("UserTriggerDecisionEvent");
        formatter.field("session_id", &self.session_id);
        formatter.field("request_id", &self.request_id);
        formatter.field("customization_arn", &self.customization_arn);
        formatter.field("programming_language", &self.programming_language);
        formatter.field("completion_type", &"*** Sensitive Data Redacted ***");
        formatter.field("suggestion_state", &"*** Sensitive Data Redacted ***");
        formatter.field(
            "recommendation_latency_milliseconds",
            &"*** Sensitive Data Redacted ***",
        );
        formatter.field("timestamp", &self.timestamp);
        formatter.field("suggestion_reference_count", &"*** Sensitive Data Redacted ***");
        formatter.field("generated_line", &"*** Sensitive Data Redacted ***");
        formatter.finish()
    }
}
impl UserTriggerDecisionEvent {
    /// Creates a new builder-style object to manufacture
    /// [`UserTriggerDecisionEvent`](crate::types::UserTriggerDecisionEvent).
    pub fn builder() -> crate::types::builders::UserTriggerDecisionEventBuilder {
        crate::types::builders::UserTriggerDecisionEventBuilder::default()
    }
}

/// A builder for [`UserTriggerDecisionEvent`](crate::types::UserTriggerDecisionEvent).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default)]
pub struct UserTriggerDecisionEventBuilder {
    pub(crate) session_id: ::std::option::Option<::std::string::String>,
    pub(crate) request_id: ::std::option::Option<::std::string::String>,
    pub(crate) customization_arn: ::std::option::Option<::std::string::String>,
    pub(crate) programming_language: ::std::option::Option<crate::types::ProgrammingLanguage>,
    pub(crate) completion_type: ::std::option::Option<crate::types::CompletionType>,
    pub(crate) suggestion_state: ::std::option::Option<crate::types::SuggestionState>,
    pub(crate) recommendation_latency_milliseconds: ::std::option::Option<f64>,
    pub(crate) timestamp: ::std::option::Option<::aws_smithy_types::DateTime>,
    pub(crate) suggestion_reference_count: ::std::option::Option<i32>,
    pub(crate) generated_line: ::std::option::Option<i32>,
}
impl UserTriggerDecisionEventBuilder {
    #[allow(missing_docs)] // documentation missing in model
    pub fn session_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.session_id = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_session_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.session_id = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_session_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.session_id
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn request_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.request_id = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_request_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.request_id = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_request_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.request_id
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn customization_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.customization_arn = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_customization_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.customization_arn = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_customization_arn(&self) -> &::std::option::Option<::std::string::String> {
        &self.customization_arn
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
    pub fn completion_type(mut self, input: crate::types::CompletionType) -> Self {
        self.completion_type = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_completion_type(mut self, input: ::std::option::Option<crate::types::CompletionType>) -> Self {
        self.completion_type = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_completion_type(&self) -> &::std::option::Option<crate::types::CompletionType> {
        &self.completion_type
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn suggestion_state(mut self, input: crate::types::SuggestionState) -> Self {
        self.suggestion_state = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_suggestion_state(mut self, input: ::std::option::Option<crate::types::SuggestionState>) -> Self {
        self.suggestion_state = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_suggestion_state(&self) -> &::std::option::Option<crate::types::SuggestionState> {
        &self.suggestion_state
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn recommendation_latency_milliseconds(mut self, input: f64) -> Self {
        self.recommendation_latency_milliseconds = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_recommendation_latency_milliseconds(mut self, input: ::std::option::Option<f64>) -> Self {
        self.recommendation_latency_milliseconds = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_recommendation_latency_milliseconds(&self) -> &::std::option::Option<f64> {
        &self.recommendation_latency_milliseconds
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn timestamp(mut self, input: ::aws_smithy_types::DateTime) -> Self {
        self.timestamp = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_timestamp(mut self, input: ::std::option::Option<::aws_smithy_types::DateTime>) -> Self {
        self.timestamp = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_timestamp(&self) -> &::std::option::Option<::aws_smithy_types::DateTime> {
        &self.timestamp
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn suggestion_reference_count(mut self, input: i32) -> Self {
        self.suggestion_reference_count = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_suggestion_reference_count(mut self, input: ::std::option::Option<i32>) -> Self {
        self.suggestion_reference_count = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_suggestion_reference_count(&self) -> &::std::option::Option<i32> {
        &self.suggestion_reference_count
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn generated_line(mut self, input: i32) -> Self {
        self.generated_line = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_generated_line(mut self, input: ::std::option::Option<i32>) -> Self {
        self.generated_line = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_generated_line(&self) -> &::std::option::Option<i32> {
        &self.generated_line
    }

    /// Consumes the builder and constructs a
    /// [`UserTriggerDecisionEvent`](crate::types::UserTriggerDecisionEvent).
    pub fn build(self) -> crate::types::UserTriggerDecisionEvent {
        crate::types::UserTriggerDecisionEvent {
            session_id: self.session_id,
            request_id: self.request_id,
            customization_arn: self.customization_arn,
            programming_language: self.programming_language,
            completion_type: self.completion_type,
            suggestion_state: self.suggestion_state,
            recommendation_latency_milliseconds: self.recommendation_latency_milliseconds.unwrap_or_default(),
            timestamp: self.timestamp,
            suggestion_reference_count: self.suggestion_reference_count.unwrap_or_default(),
            generated_line: self.generated_line.unwrap_or_default(),
        }
    }
}
impl ::std::fmt::Debug for UserTriggerDecisionEventBuilder {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut formatter = f.debug_struct("UserTriggerDecisionEventBuilder");
        formatter.field("session_id", &self.session_id);
        formatter.field("request_id", &self.request_id);
        formatter.field("customization_arn", &self.customization_arn);
        formatter.field("programming_language", &self.programming_language);
        formatter.field("completion_type", &"*** Sensitive Data Redacted ***");
        formatter.field("suggestion_state", &"*** Sensitive Data Redacted ***");
        formatter.field(
            "recommendation_latency_milliseconds",
            &"*** Sensitive Data Redacted ***",
        );
        formatter.field("timestamp", &self.timestamp);
        formatter.field("suggestion_reference_count", &"*** Sensitive Data Redacted ***");
        formatter.field("generated_line", &"*** Sensitive Data Redacted ***");
        formatter.finish()
    }
}
