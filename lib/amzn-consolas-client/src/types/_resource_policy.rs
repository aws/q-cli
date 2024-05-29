// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct ResourcePolicy {
    #[allow(missing_docs)] // documentation missing in model
    pub effect: crate::types::ResourcePolicyEffect,
}
impl ResourcePolicy {
    #[allow(missing_docs)] // documentation missing in model
    pub fn effect(&self) -> &crate::types::ResourcePolicyEffect {
        &self.effect
    }
}
impl ResourcePolicy {
    /// Creates a new builder-style object to manufacture
    /// [`ResourcePolicy`](crate::types::ResourcePolicy).
    pub fn builder() -> crate::types::builders::ResourcePolicyBuilder {
        crate::types::builders::ResourcePolicyBuilder::default()
    }
}

/// A builder for [`ResourcePolicy`](crate::types::ResourcePolicy).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct ResourcePolicyBuilder {
    pub(crate) effect: ::std::option::Option<crate::types::ResourcePolicyEffect>,
}
impl ResourcePolicyBuilder {
    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn effect(mut self, input: crate::types::ResourcePolicyEffect) -> Self {
        self.effect = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_effect(mut self, input: ::std::option::Option<crate::types::ResourcePolicyEffect>) -> Self {
        self.effect = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_effect(&self) -> &::std::option::Option<crate::types::ResourcePolicyEffect> {
        &self.effect
    }

    /// Consumes the builder and constructs a [`ResourcePolicy`](crate::types::ResourcePolicy).
    /// This method will fail if any of the following fields are not set:
    /// - [`effect`](crate::types::builders::ResourcePolicyBuilder::effect)
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::types::ResourcePolicy, ::aws_smithy_types::error::operation::BuildError> {
        ::std::result::Result::Ok(crate::types::ResourcePolicy {
            effect: self.effect.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "effect",
                    "effect was not specified but it is required when building ResourcePolicy",
                )
            })?,
        })
    }
}
