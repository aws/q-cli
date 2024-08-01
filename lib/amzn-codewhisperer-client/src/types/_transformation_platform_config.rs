// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct TransformationPlatformConfig {
    #[allow(missing_docs)] // documentation missing in model
    pub operating_system_family: ::std::option::Option<crate::types::TransformationOperatingSystemFamily>,
}
impl TransformationPlatformConfig {
    #[allow(missing_docs)] // documentation missing in model
    pub fn operating_system_family(&self) -> ::std::option::Option<&crate::types::TransformationOperatingSystemFamily> {
        self.operating_system_family.as_ref()
    }
}
impl TransformationPlatformConfig {
    /// Creates a new builder-style object to manufacture
    /// [`TransformationPlatformConfig`](crate::types::TransformationPlatformConfig).
    pub fn builder() -> crate::types::builders::TransformationPlatformConfigBuilder {
        crate::types::builders::TransformationPlatformConfigBuilder::default()
    }
}

/// A builder for [`TransformationPlatformConfig`](crate::types::TransformationPlatformConfig).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct TransformationPlatformConfigBuilder {
    pub(crate) operating_system_family: ::std::option::Option<crate::types::TransformationOperatingSystemFamily>,
}
impl TransformationPlatformConfigBuilder {
    #[allow(missing_docs)] // documentation missing in model
    pub fn operating_system_family(mut self, input: crate::types::TransformationOperatingSystemFamily) -> Self {
        self.operating_system_family = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_operating_system_family(
        mut self,
        input: ::std::option::Option<crate::types::TransformationOperatingSystemFamily>,
    ) -> Self {
        self.operating_system_family = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_operating_system_family(
        &self,
    ) -> &::std::option::Option<crate::types::TransformationOperatingSystemFamily> {
        &self.operating_system_family
    }

    /// Consumes the builder and constructs a
    /// [`TransformationPlatformConfig`](crate::types::TransformationPlatformConfig).
    pub fn build(self) -> crate::types::TransformationPlatformConfig {
        crate::types::TransformationPlatformConfig {
            operating_system_family: self.operating_system_family,
        }
    }
}
