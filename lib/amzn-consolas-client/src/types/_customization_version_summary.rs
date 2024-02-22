// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct CustomizationVersionSummary {
    #[allow(missing_docs)] // documentation missing in model
    pub version: i64,
    #[allow(missing_docs)] // documentation missing in model
    pub base_version: ::std::option::Option<i64>,
    #[allow(missing_docs)] // documentation missing in model
    pub status: crate::types::CustomizationStatus,
    #[allow(missing_docs)] // documentation missing in model
    pub data_reference: crate::types::DataReference,
    #[allow(missing_docs)] // documentation missing in model
    pub updated_at: ::aws_smithy_types::DateTime,
    #[allow(missing_docs)] // documentation missing in model
    pub evaluation_metrics: ::std::option::Option<crate::types::EvaluationMetrics>,
}
impl CustomizationVersionSummary {
    #[allow(missing_docs)] // documentation missing in model
    pub fn version(&self) -> i64 {
        self.version
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn base_version(&self) -> ::std::option::Option<i64> {
        self.base_version
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn status(&self) -> &crate::types::CustomizationStatus {
        &self.status
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn data_reference(&self) -> &crate::types::DataReference {
        &self.data_reference
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn updated_at(&self) -> &::aws_smithy_types::DateTime {
        &self.updated_at
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn evaluation_metrics(&self) -> ::std::option::Option<&crate::types::EvaluationMetrics> {
        self.evaluation_metrics.as_ref()
    }
}
impl CustomizationVersionSummary {
    /// Creates a new builder-style object to manufacture
    /// [`CustomizationVersionSummary`](crate::types::CustomizationVersionSummary).
    pub fn builder() -> crate::types::builders::CustomizationVersionSummaryBuilder {
        crate::types::builders::CustomizationVersionSummaryBuilder::default()
    }
}

/// A builder for [`CustomizationVersionSummary`](crate::types::CustomizationVersionSummary).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct CustomizationVersionSummaryBuilder {
    pub(crate) version: ::std::option::Option<i64>,
    pub(crate) base_version: ::std::option::Option<i64>,
    pub(crate) status: ::std::option::Option<crate::types::CustomizationStatus>,
    pub(crate) data_reference: ::std::option::Option<crate::types::DataReference>,
    pub(crate) updated_at: ::std::option::Option<::aws_smithy_types::DateTime>,
    pub(crate) evaluation_metrics: ::std::option::Option<crate::types::EvaluationMetrics>,
}
impl CustomizationVersionSummaryBuilder {
    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
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
    pub fn base_version(mut self, input: i64) -> Self {
        self.base_version = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_base_version(mut self, input: ::std::option::Option<i64>) -> Self {
        self.base_version = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_base_version(&self) -> &::std::option::Option<i64> {
        &self.base_version
    }

    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
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
    /// This field is required.
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
    /// This field is required.
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

    /// Consumes the builder and constructs a
    /// [`CustomizationVersionSummary`](crate::types::CustomizationVersionSummary). This method
    /// will fail if any of the following fields are not set:
    /// - [`version`](crate::types::builders::CustomizationVersionSummaryBuilder::version)
    /// - [`status`](crate::types::builders::CustomizationVersionSummaryBuilder::status)
    /// - [`data_reference`](crate::types::builders::CustomizationVersionSummaryBuilder::data_reference)
    /// - [`updated_at`](crate::types::builders::CustomizationVersionSummaryBuilder::updated_at)
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::types::CustomizationVersionSummary,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(crate::types::CustomizationVersionSummary {
            version: self.version.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "version",
                    "version was not specified but it is required when building CustomizationVersionSummary",
                )
            })?,
            base_version: self.base_version,
            status: self.status.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "status",
                    "status was not specified but it is required when building CustomizationVersionSummary",
                )
            })?,
            data_reference: self.data_reference.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "data_reference",
                    "data_reference was not specified but it is required when building CustomizationVersionSummary",
                )
            })?,
            updated_at: self.updated_at.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "updated_at",
                    "updated_at was not specified but it is required when building CustomizationVersionSummary",
                )
            })?,
            evaluation_metrics: self.evaluation_metrics,
        })
    }
}
