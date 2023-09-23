// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct GetCustomizationOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub arn: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub version: ::std::option::Option<i64>,
    #[allow(missing_docs)] // documentation missing in model
    pub status: ::std::option::Option<crate::types::CustomizationStatus>,
    #[allow(missing_docs)] // documentation missing in model
    pub error_details: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub data_reference: ::std::option::Option<crate::types::DataReference>,
    #[allow(missing_docs)] // documentation missing in model
    pub customization_name: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub description: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub profile_arn: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub updated_at: ::std::option::Option<::aws_smithy_types::DateTime>,
    #[allow(missing_docs)] // documentation missing in model
    pub evaluation_metrics: ::std::option::Option<crate::types::EvaluationMetrics>,
    _request_id: Option<String>,
}
impl GetCustomizationOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub fn arn(&self) -> ::std::option::Option<&str> {
        self.arn.as_deref()
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn version(&self) -> ::std::option::Option<i64> {
        self.version
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn status(&self) -> ::std::option::Option<&crate::types::CustomizationStatus> {
        self.status.as_ref()
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn error_details(&self) -> ::std::option::Option<&str> {
        self.error_details.as_deref()
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn data_reference(&self) -> ::std::option::Option<&crate::types::DataReference> {
        self.data_reference.as_ref()
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn customization_name(&self) -> ::std::option::Option<&str> {
        self.customization_name.as_deref()
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn description(&self) -> ::std::option::Option<&str> {
        self.description.as_deref()
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn profile_arn(&self) -> ::std::option::Option<&str> {
        self.profile_arn.as_deref()
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn updated_at(&self) -> ::std::option::Option<&::aws_smithy_types::DateTime> {
        self.updated_at.as_ref()
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn evaluation_metrics(&self) -> ::std::option::Option<&crate::types::EvaluationMetrics> {
        self.evaluation_metrics.as_ref()
    }
}
impl ::aws_http::request_id::RequestId for GetCustomizationOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl GetCustomizationOutput {
    /// Creates a new builder-style object to manufacture [`GetCustomizationOutput`](crate::operation::get_customization::GetCustomizationOutput).
    pub fn builder() -> crate::operation::get_customization::builders::GetCustomizationOutputBuilder {
        crate::operation::get_customization::builders::GetCustomizationOutputBuilder::default()
    }
}

/// A builder for [`GetCustomizationOutput`](crate::operation::get_customization::GetCustomizationOutput).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct GetCustomizationOutputBuilder {
    pub(crate) arn: ::std::option::Option<::std::string::String>,
    pub(crate) version: ::std::option::Option<i64>,
    pub(crate) status: ::std::option::Option<crate::types::CustomizationStatus>,
    pub(crate) error_details: ::std::option::Option<::std::string::String>,
    pub(crate) data_reference: ::std::option::Option<crate::types::DataReference>,
    pub(crate) customization_name: ::std::option::Option<::std::string::String>,
    pub(crate) description: ::std::option::Option<::std::string::String>,
    pub(crate) profile_arn: ::std::option::Option<::std::string::String>,
    pub(crate) updated_at: ::std::option::Option<::aws_smithy_types::DateTime>,
    pub(crate) evaluation_metrics: ::std::option::Option<crate::types::EvaluationMetrics>,
    _request_id: Option<String>,
}
impl GetCustomizationOutputBuilder {
    #[allow(missing_docs)] // documentation missing in model
    pub fn arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.arn = ::std::option::Option::Some(input.into());
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.arn = input;
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_arn(&self) -> &::std::option::Option<::std::string::String> {
        &self.arn
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn version(mut self, input: i64) -> Self {
        self.version = ::std::option::Option::Some(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_version(mut self, input: ::std::option::Option<i64>) -> Self {
        self.version = input;
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_version(&self) -> &::std::option::Option<i64> {
        &self.version
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn status(mut self, input: crate::types::CustomizationStatus) -> Self {
        self.status = ::std::option::Option::Some(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_status(mut self, input: ::std::option::Option<crate::types::CustomizationStatus>) -> Self {
        self.status = input;
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_status(&self) -> &::std::option::Option<crate::types::CustomizationStatus> {
        &self.status
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn error_details(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.error_details = ::std::option::Option::Some(input.into());
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_error_details(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.error_details = input;
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_error_details(&self) -> &::std::option::Option<::std::string::String> {
        &self.error_details
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn data_reference(mut self, input: crate::types::DataReference) -> Self {
        self.data_reference = ::std::option::Option::Some(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_data_reference(mut self, input: ::std::option::Option<crate::types::DataReference>) -> Self {
        self.data_reference = input;
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_data_reference(&self) -> &::std::option::Option<crate::types::DataReference> {
        &self.data_reference
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn customization_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.customization_name = ::std::option::Option::Some(input.into());
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_customization_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.customization_name = input;
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_customization_name(&self) -> &::std::option::Option<::std::string::String> {
        &self.customization_name
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn description(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.description = ::std::option::Option::Some(input.into());
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_description(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.description = input;
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_description(&self) -> &::std::option::Option<::std::string::String> {
        &self.description
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn profile_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.profile_arn = ::std::option::Option::Some(input.into());
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_profile_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.profile_arn = input;
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_profile_arn(&self) -> &::std::option::Option<::std::string::String> {
        &self.profile_arn
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn updated_at(mut self, input: ::aws_smithy_types::DateTime) -> Self {
        self.updated_at = ::std::option::Option::Some(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_updated_at(mut self, input: ::std::option::Option<::aws_smithy_types::DateTime>) -> Self {
        self.updated_at = input;
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_updated_at(&self) -> &::std::option::Option<::aws_smithy_types::DateTime> {
        &self.updated_at
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn evaluation_metrics(mut self, input: crate::types::EvaluationMetrics) -> Self {
        self.evaluation_metrics = ::std::option::Option::Some(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_evaluation_metrics(mut self, input: ::std::option::Option<crate::types::EvaluationMetrics>) -> Self {
        self.evaluation_metrics = input;
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_evaluation_metrics(&self) -> &::std::option::Option<crate::types::EvaluationMetrics> {
        &self.evaluation_metrics
    }
    pub(crate) fn _request_id(mut self, request_id: impl Into<String>) -> Self {
        self._request_id = Some(request_id.into());
        self
    }

    pub(crate) fn _set_request_id(&mut self, request_id: Option<String>) -> &mut Self {
        self._request_id = request_id;
        self
    }
    /// Consumes the builder and constructs a [`GetCustomizationOutput`](crate::operation::get_customization::GetCustomizationOutput).
    pub fn build(self) -> crate::operation::get_customization::GetCustomizationOutput {
        crate::operation::get_customization::GetCustomizationOutput {
            arn: self.arn,
            version: self.version,
            status: self.status,
            error_details: self.error_details,
            data_reference: self.data_reference,
            customization_name: self.customization_name,
            description: self.description,
            profile_arn: self.profile_arn,
            updated_at: self.updated_at,
            evaluation_metrics: self.evaluation_metrics,
            _request_id: self._request_id,
        }
    }
}
