// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct AllowVendedLogDeliveryForResourceOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub message: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl AllowVendedLogDeliveryForResourceOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub fn message(&self) -> ::std::option::Option<&str> {
        self.message.as_deref()
    }
}
impl ::aws_http::request_id::RequestId for AllowVendedLogDeliveryForResourceOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl AllowVendedLogDeliveryForResourceOutput {
    /// Creates a new builder-style object to manufacture
    /// [`AllowVendedLogDeliveryForResourceOutput`](crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceOutput).
    ///
    pub fn builder() -> crate::operation::allow_vended_log_delivery_for_resource::builders::AllowVendedLogDeliveryForResourceOutputBuilder{
        crate::operation::allow_vended_log_delivery_for_resource::builders::AllowVendedLogDeliveryForResourceOutputBuilder::default()
    }
}

/// A builder for
/// [`AllowVendedLogDeliveryForResourceOutput`](crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceOutput).
///
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct AllowVendedLogDeliveryForResourceOutputBuilder {
    pub(crate) message: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl AllowVendedLogDeliveryForResourceOutputBuilder {
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

    pub(crate) fn _request_id(mut self, request_id: impl Into<String>) -> Self {
        self._request_id = Some(request_id.into());
        self
    }

    pub(crate) fn _set_request_id(&mut self, request_id: Option<String>) -> &mut Self {
        self._request_id = request_id;
        self
    }

    /// Consumes the builder and constructs a
    /// [`AllowVendedLogDeliveryForResourceOutput`](crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceOutput).
    ///
    pub fn build(
        self,
    ) -> crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceOutput {
        crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceOutput {
            message: self.message,
            _request_id: self._request_id,
        }
    }
}
