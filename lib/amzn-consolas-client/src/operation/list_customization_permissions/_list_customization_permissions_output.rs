// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct ListCustomizationPermissionsOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub permissions: ::std::vec::Vec<crate::types::CustomizationPermission>,
    #[allow(missing_docs)] // documentation missing in model
    pub next_token: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl ListCustomizationPermissionsOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub fn permissions(&self) -> &[crate::types::CustomizationPermission] {
        use std::ops::Deref;
        self.permissions.deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn next_token(&self) -> ::std::option::Option<&str> {
        self.next_token.as_deref()
    }
}
impl ::aws_types::request_id::RequestId for ListCustomizationPermissionsOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl ListCustomizationPermissionsOutput {
    /// Creates a new builder-style object to manufacture
    /// [`ListCustomizationPermissionsOutput`](crate::operation::list_customization_permissions::ListCustomizationPermissionsOutput).
    pub fn builder()
    -> crate::operation::list_customization_permissions::builders::ListCustomizationPermissionsOutputBuilder {
        crate::operation::list_customization_permissions::builders::ListCustomizationPermissionsOutputBuilder::default()
    }
}

/// A builder for
/// [`ListCustomizationPermissionsOutput`](crate::operation::list_customization_permissions::ListCustomizationPermissionsOutput).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct ListCustomizationPermissionsOutputBuilder {
    pub(crate) permissions: ::std::option::Option<::std::vec::Vec<crate::types::CustomizationPermission>>,
    pub(crate) next_token: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl ListCustomizationPermissionsOutputBuilder {
    /// Appends an item to `permissions`.
    ///
    /// To override the contents of this collection use [`set_permissions`](Self::set_permissions).
    pub fn permissions(mut self, input: crate::types::CustomizationPermission) -> Self {
        let mut v = self.permissions.unwrap_or_default();
        v.push(input);
        self.permissions = ::std::option::Option::Some(v);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_permissions(
        mut self,
        input: ::std::option::Option<::std::vec::Vec<crate::types::CustomizationPermission>>,
    ) -> Self {
        self.permissions = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_permissions(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::CustomizationPermission>> {
        &self.permissions
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
    /// [`ListCustomizationPermissionsOutput`](crate::operation::list_customization_permissions::ListCustomizationPermissionsOutput).
    /// This method will fail if any of the following fields are not set:
    /// - [`permissions`](crate::operation::list_customization_permissions::builders::ListCustomizationPermissionsOutputBuilder::permissions)
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::list_customization_permissions::ListCustomizationPermissionsOutput,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(crate::operation::list_customization_permissions::ListCustomizationPermissionsOutput {
            permissions: self.permissions.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "permissions",
                    "permissions was not specified but it is required when building ListCustomizationPermissionsOutput",
                )
            })?,
            next_token: self.next_token,
            _request_id: self._request_id,
        })
    }
}
