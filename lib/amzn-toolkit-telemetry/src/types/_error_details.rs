// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct ErrorDetails {
    #[allow(missing_docs)] // documentation missing in model
    pub command: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub epoch_timestamp: i64,
    #[allow(missing_docs)] // documentation missing in model
    pub r#type: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub message: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub stack_trace: ::std::option::Option<::std::string::String>,
}
impl ErrorDetails {
    #[allow(missing_docs)] // documentation missing in model
    pub fn command(&self) -> ::std::option::Option<&str> {
        self.command.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn epoch_timestamp(&self) -> i64 {
        self.epoch_timestamp
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn r#type(&self) -> ::std::option::Option<&str> {
        self.r#type.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn message(&self) -> ::std::option::Option<&str> {
        self.message.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn stack_trace(&self) -> ::std::option::Option<&str> {
        self.stack_trace.as_deref()
    }
}
impl ErrorDetails {
    /// Creates a new builder-style object to manufacture
    /// [`ErrorDetails`](crate::types::ErrorDetails).
    pub fn builder() -> crate::types::builders::ErrorDetailsBuilder {
        crate::types::builders::ErrorDetailsBuilder::default()
    }
}

/// A builder for [`ErrorDetails`](crate::types::ErrorDetails).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct ErrorDetailsBuilder {
    pub(crate) command: ::std::option::Option<::std::string::String>,
    pub(crate) epoch_timestamp: ::std::option::Option<i64>,
    pub(crate) r#type: ::std::option::Option<::std::string::String>,
    pub(crate) message: ::std::option::Option<::std::string::String>,
    pub(crate) stack_trace: ::std::option::Option<::std::string::String>,
}
impl ErrorDetailsBuilder {
    #[allow(missing_docs)] // documentation missing in model
    pub fn command(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.command = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_command(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.command = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_command(&self) -> &::std::option::Option<::std::string::String> {
        &self.command
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn epoch_timestamp(mut self, input: i64) -> Self {
        self.epoch_timestamp = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_epoch_timestamp(mut self, input: ::std::option::Option<i64>) -> Self {
        self.epoch_timestamp = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_epoch_timestamp(&self) -> &::std::option::Option<i64> {
        &self.epoch_timestamp
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn r#type(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.r#type = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_type(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.r#type = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_type(&self) -> &::std::option::Option<::std::string::String> {
        &self.r#type
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn message(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.message = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_message(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.message = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_message(&self) -> &::std::option::Option<::std::string::String> {
        &self.message
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn stack_trace(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.stack_trace = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_stack_trace(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.stack_trace = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_stack_trace(&self) -> &::std::option::Option<::std::string::String> {
        &self.stack_trace
    }

    /// Consumes the builder and constructs a [`ErrorDetails`](crate::types::ErrorDetails).
    pub fn build(self) -> crate::types::ErrorDetails {
        crate::types::ErrorDetails {
            command: self.command,
            epoch_timestamp: self.epoch_timestamp.unwrap_or_default(),
            r#type: self.r#type,
            message: self.message,
            stack_trace: self.stack_trace,
        }
    }
}
