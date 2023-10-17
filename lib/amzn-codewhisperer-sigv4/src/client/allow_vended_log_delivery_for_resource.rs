// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the
    /// [`AllowVendedLogDeliveryForResource`](crate::operation::allow_vended_log_delivery_for_resource::builders::AllowVendedLogDeliveryForResourceFluentBuilder)
    /// operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`resource_arn_being_authorized(impl Into<String>)`](crate::operation::allow_vended_log_delivery_for_resource::builders::AllowVendedLogDeliveryForResourceFluentBuilder::resource_arn_being_authorized) / [`set_resource_arn_being_authorized(Option<String>)`](crate::operation::allow_vended_log_delivery_for_resource::builders::AllowVendedLogDeliveryForResourceFluentBuilder::set_resource_arn_being_authorized): (undocumented)
    ///   - [`delivery_source_arn(impl Into<String>)`](crate::operation::allow_vended_log_delivery_for_resource::builders::AllowVendedLogDeliveryForResourceFluentBuilder::delivery_source_arn) / [`set_delivery_source_arn(Option<String>)`](crate::operation::allow_vended_log_delivery_for_resource::builders::AllowVendedLogDeliveryForResourceFluentBuilder::set_delivery_source_arn): (undocumented)
    /// - On success, responds with [`AllowVendedLogDeliveryForResourceOutput`](crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceOutput) with field(s):
    ///   - [`message(Option<String>)`](crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceOutput::message): (undocumented)
    /// - On failure, responds with [`SdkError<AllowVendedLogDeliveryForResourceError>`](crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceError)
    pub fn allow_vended_log_delivery_for_resource(
        &self,
    ) -> crate::operation::allow_vended_log_delivery_for_resource::builders::AllowVendedLogDeliveryForResourceFluentBuilder{
        crate::operation::allow_vended_log_delivery_for_resource::builders::AllowVendedLogDeliveryForResourceFluentBuilder::new(self.handle.clone())
    }
}
