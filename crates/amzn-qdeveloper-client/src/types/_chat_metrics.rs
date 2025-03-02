// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct ChatMetrics {
    #[allow(missing_docs)] // documentation missing in model
    pub messages_sent: ::std::option::Option<i64>,
    #[allow(missing_docs)] // documentation missing in model
    pub messages_interacted: ::std::option::Option<i64>,
    #[allow(missing_docs)] // documentation missing in model
    pub ai_code_lines: ::std::option::Option<i64>,
    #[allow(missing_docs)] // documentation missing in model
    pub characters_of_code_accepted: i32,
}
impl ChatMetrics {
    #[allow(missing_docs)] // documentation missing in model
    pub fn messages_sent(&self) -> ::std::option::Option<i64> {
        self.messages_sent
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn messages_interacted(&self) -> ::std::option::Option<i64> {
        self.messages_interacted
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn ai_code_lines(&self) -> ::std::option::Option<i64> {
        self.ai_code_lines
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn characters_of_code_accepted(&self) -> i32 {
        self.characters_of_code_accepted
    }
}
impl ChatMetrics {
    /// Creates a new builder-style object to manufacture
    /// [`ChatMetrics`](crate::types::ChatMetrics).
    pub fn builder() -> crate::types::builders::ChatMetricsBuilder {
        crate::types::builders::ChatMetricsBuilder::default()
    }
}

/// A builder for [`ChatMetrics`](crate::types::ChatMetrics).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct ChatMetricsBuilder {
    pub(crate) messages_sent: ::std::option::Option<i64>,
    pub(crate) messages_interacted: ::std::option::Option<i64>,
    pub(crate) ai_code_lines: ::std::option::Option<i64>,
    pub(crate) characters_of_code_accepted: ::std::option::Option<i32>,
}
impl ChatMetricsBuilder {
    #[allow(missing_docs)] // documentation missing in model
    pub fn messages_sent(mut self, input: i64) -> Self {
        self.messages_sent = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_messages_sent(mut self, input: ::std::option::Option<i64>) -> Self {
        self.messages_sent = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_messages_sent(&self) -> &::std::option::Option<i64> {
        &self.messages_sent
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn messages_interacted(mut self, input: i64) -> Self {
        self.messages_interacted = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_messages_interacted(mut self, input: ::std::option::Option<i64>) -> Self {
        self.messages_interacted = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_messages_interacted(&self) -> &::std::option::Option<i64> {
        &self.messages_interacted
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn ai_code_lines(mut self, input: i64) -> Self {
        self.ai_code_lines = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_ai_code_lines(mut self, input: ::std::option::Option<i64>) -> Self {
        self.ai_code_lines = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_ai_code_lines(&self) -> &::std::option::Option<i64> {
        &self.ai_code_lines
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn characters_of_code_accepted(mut self, input: i32) -> Self {
        self.characters_of_code_accepted = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_characters_of_code_accepted(mut self, input: ::std::option::Option<i32>) -> Self {
        self.characters_of_code_accepted = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_characters_of_code_accepted(&self) -> &::std::option::Option<i32> {
        &self.characters_of_code_accepted
    }

    /// Consumes the builder and constructs a [`ChatMetrics`](crate::types::ChatMetrics).
    pub fn build(self) -> crate::types::ChatMetrics {
        crate::types::ChatMetrics {
            messages_sent: self.messages_sent,
            messages_interacted: self.messages_interacted,
            ai_code_lines: self.ai_code_lines,
            characters_of_code_accepted: self.characters_of_code_accepted.unwrap_or_default(),
        }
    }
}
