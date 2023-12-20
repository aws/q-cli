// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Represents span in a text
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct Span {
    #[allow(missing_docs)] // documentation missing in model
    pub start: ::std::option::Option<i32>,
    #[allow(missing_docs)] // documentation missing in model
    pub end: ::std::option::Option<i32>,
}
impl Span {
    #[allow(missing_docs)] // documentation missing in model
    pub fn start(&self) -> ::std::option::Option<i32> {
        self.start
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn end(&self) -> ::std::option::Option<i32> {
        self.end
    }
}
impl Span {
    /// Creates a new builder-style object to manufacture [`Span`](crate::types::Span).
    pub fn builder() -> crate::types::builders::SpanBuilder {
        crate::types::builders::SpanBuilder::default()
    }
}

/// A builder for [`Span`](crate::types::Span).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct SpanBuilder {
    pub(crate) start: ::std::option::Option<i32>,
    pub(crate) end: ::std::option::Option<i32>,
}
impl SpanBuilder {
    #[allow(missing_docs)] // documentation missing in model
    pub fn start(mut self, input: i32) -> Self {
        self.start = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_start(mut self, input: ::std::option::Option<i32>) -> Self {
        self.start = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_start(&self) -> &::std::option::Option<i32> {
        &self.start
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn end(mut self, input: i32) -> Self {
        self.end = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_end(mut self, input: ::std::option::Option<i32>) -> Self {
        self.end = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_end(&self) -> &::std::option::Option<i32> {
        &self.end
    }

    /// Consumes the builder and constructs a [`Span`](crate::types::Span).
    pub fn build(self) -> crate::types::Span {
        crate::types::Span {
            start: self.start,
            end: self.end,
        }
    }
}
