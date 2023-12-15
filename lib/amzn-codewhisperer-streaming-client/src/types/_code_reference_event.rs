// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Streaming Response Event for CodeReferences
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct CodeReferenceEvent {
    /// Code References for Assistant Response Message
    pub references: ::std::option::Option<::std::vec::Vec<crate::types::Reference>>,
}
impl CodeReferenceEvent {
    /// Code References for Assistant Response Message
    ///
    /// If no value was sent for this field, a default will be set. If you want to determine if no value was sent, use `.references.is_none()`.
    pub fn references(&self) -> &[crate::types::Reference] {
        self.references.as_deref().unwrap_or_default()
    }
}
impl CodeReferenceEvent {
    /// Creates a new builder-style object to manufacture [`CodeReferenceEvent`](crate::types::CodeReferenceEvent).
    pub fn builder() -> crate::types::builders::CodeReferenceEventBuilder {
        crate::types::builders::CodeReferenceEventBuilder::default()
    }
}

/// A builder for [`CodeReferenceEvent`](crate::types::CodeReferenceEvent).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct CodeReferenceEventBuilder {
    pub(crate) references: ::std::option::Option<::std::vec::Vec<crate::types::Reference>>,
}
impl CodeReferenceEventBuilder {
    /// Appends an item to `references`.
    ///
    /// To override the contents of this collection use [`set_references`](Self::set_references).
    ///
    /// Code References for Assistant Response Message
    pub fn references(mut self, input: crate::types::Reference) -> Self {
        let mut v = self.references.unwrap_or_default();
        v.push(input);
        self.references = ::std::option::Option::Some(v);
        self
    }
    /// Code References for Assistant Response Message
    pub fn set_references(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Reference>>) -> Self {
        self.references = input;
        self
    }
    /// Code References for Assistant Response Message
    pub fn get_references(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Reference>> {
        &self.references
    }
    /// Consumes the builder and constructs a [`CodeReferenceEvent`](crate::types::CodeReferenceEvent).
    pub fn build(self) -> crate::types::CodeReferenceEvent {
        crate::types::CodeReferenceEvent { references: self.references }
    }
}
