// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Represents a test generation job
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq)]
pub struct TestGenerationJob {
    #[allow(missing_docs)] // documentation missing in model
    pub test_generation_job_id: ::std::string::String,
    /// Test generation job group name
    pub test_generation_job_group_name: ::std::string::String,
    #[allow(missing_docs)] // documentation missing in model
    pub status: crate::types::TestGenerationJobStatus,
    #[allow(missing_docs)] // documentation missing in model
    pub short_answer: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub creation_time: ::aws_smithy_types::DateTime,
    #[allow(missing_docs)] // documentation missing in model
    pub progress_rate: ::std::option::Option<i32>,
    #[allow(missing_docs)] // documentation missing in model
    pub job_status_reason: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub job_summary: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub job_plan: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub package_info_list: ::std::option::Option<::std::vec::Vec<crate::types::PackageInfo>>,
}
impl TestGenerationJob {
    #[allow(missing_docs)] // documentation missing in model
    pub fn test_generation_job_id(&self) -> &str {
        use std::ops::Deref;
        self.test_generation_job_id.deref()
    }

    /// Test generation job group name
    pub fn test_generation_job_group_name(&self) -> &str {
        use std::ops::Deref;
        self.test_generation_job_group_name.deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn status(&self) -> &crate::types::TestGenerationJobStatus {
        &self.status
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn short_answer(&self) -> ::std::option::Option<&str> {
        self.short_answer.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn creation_time(&self) -> &::aws_smithy_types::DateTime {
        &self.creation_time
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn progress_rate(&self) -> ::std::option::Option<i32> {
        self.progress_rate
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn job_status_reason(&self) -> ::std::option::Option<&str> {
        self.job_status_reason.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn job_summary(&self) -> ::std::option::Option<&str> {
        self.job_summary.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn job_plan(&self) -> ::std::option::Option<&str> {
        self.job_plan.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    /// If no value was sent for this field, a default will be set. If you want to determine if no
    /// value was sent, use `.package_info_list.is_none()`.
    pub fn package_info_list(&self) -> &[crate::types::PackageInfo] {
        self.package_info_list.as_deref().unwrap_or_default()
    }
}
impl ::std::fmt::Debug for TestGenerationJob {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut formatter = f.debug_struct("TestGenerationJob");
        formatter.field("test_generation_job_id", &self.test_generation_job_id);
        formatter.field("test_generation_job_group_name", &self.test_generation_job_group_name);
        formatter.field("status", &self.status);
        formatter.field("short_answer", &"*** Sensitive Data Redacted ***");
        formatter.field("creation_time", &self.creation_time);
        formatter.field("progress_rate", &self.progress_rate);
        formatter.field("job_status_reason", &self.job_status_reason);
        formatter.field("job_summary", &"*** Sensitive Data Redacted ***");
        formatter.field("job_plan", &"*** Sensitive Data Redacted ***");
        formatter.field("package_info_list", &self.package_info_list);
        formatter.finish()
    }
}
impl TestGenerationJob {
    /// Creates a new builder-style object to manufacture
    /// [`TestGenerationJob`](crate::types::TestGenerationJob).
    pub fn builder() -> crate::types::builders::TestGenerationJobBuilder {
        crate::types::builders::TestGenerationJobBuilder::default()
    }
}

/// A builder for [`TestGenerationJob`](crate::types::TestGenerationJob).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default)]
#[non_exhaustive]
pub struct TestGenerationJobBuilder {
    pub(crate) test_generation_job_id: ::std::option::Option<::std::string::String>,
    pub(crate) test_generation_job_group_name: ::std::option::Option<::std::string::String>,
    pub(crate) status: ::std::option::Option<crate::types::TestGenerationJobStatus>,
    pub(crate) short_answer: ::std::option::Option<::std::string::String>,
    pub(crate) creation_time: ::std::option::Option<::aws_smithy_types::DateTime>,
    pub(crate) progress_rate: ::std::option::Option<i32>,
    pub(crate) job_status_reason: ::std::option::Option<::std::string::String>,
    pub(crate) job_summary: ::std::option::Option<::std::string::String>,
    pub(crate) job_plan: ::std::option::Option<::std::string::String>,
    pub(crate) package_info_list: ::std::option::Option<::std::vec::Vec<crate::types::PackageInfo>>,
}
impl TestGenerationJobBuilder {
    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn test_generation_job_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.test_generation_job_id = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_test_generation_job_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.test_generation_job_id = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_test_generation_job_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.test_generation_job_id
    }

    /// Test generation job group name
    /// This field is required.
    pub fn test_generation_job_group_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.test_generation_job_group_name = ::std::option::Option::Some(input.into());
        self
    }

    /// Test generation job group name
    pub fn set_test_generation_job_group_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.test_generation_job_group_name = input;
        self
    }

    /// Test generation job group name
    pub fn get_test_generation_job_group_name(&self) -> &::std::option::Option<::std::string::String> {
        &self.test_generation_job_group_name
    }

    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn status(mut self, input: crate::types::TestGenerationJobStatus) -> Self {
        self.status = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_status(mut self, input: ::std::option::Option<crate::types::TestGenerationJobStatus>) -> Self {
        self.status = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_status(&self) -> &::std::option::Option<crate::types::TestGenerationJobStatus> {
        &self.status
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn short_answer(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.short_answer = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_short_answer(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.short_answer = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_short_answer(&self) -> &::std::option::Option<::std::string::String> {
        &self.short_answer
    }

    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn creation_time(mut self, input: ::aws_smithy_types::DateTime) -> Self {
        self.creation_time = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_creation_time(mut self, input: ::std::option::Option<::aws_smithy_types::DateTime>) -> Self {
        self.creation_time = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_creation_time(&self) -> &::std::option::Option<::aws_smithy_types::DateTime> {
        &self.creation_time
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn progress_rate(mut self, input: i32) -> Self {
        self.progress_rate = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_progress_rate(mut self, input: ::std::option::Option<i32>) -> Self {
        self.progress_rate = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_progress_rate(&self) -> &::std::option::Option<i32> {
        &self.progress_rate
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn job_status_reason(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.job_status_reason = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_job_status_reason(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.job_status_reason = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_job_status_reason(&self) -> &::std::option::Option<::std::string::String> {
        &self.job_status_reason
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn job_summary(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.job_summary = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_job_summary(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.job_summary = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_job_summary(&self) -> &::std::option::Option<::std::string::String> {
        &self.job_summary
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn job_plan(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.job_plan = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_job_plan(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.job_plan = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_job_plan(&self) -> &::std::option::Option<::std::string::String> {
        &self.job_plan
    }

    /// Appends an item to `package_info_list`.
    ///
    /// To override the contents of this collection use
    /// [`set_package_info_list`](Self::set_package_info_list).
    pub fn package_info_list(mut self, input: crate::types::PackageInfo) -> Self {
        let mut v = self.package_info_list.unwrap_or_default();
        v.push(input);
        self.package_info_list = ::std::option::Option::Some(v);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_package_info_list(
        mut self,
        input: ::std::option::Option<::std::vec::Vec<crate::types::PackageInfo>>,
    ) -> Self {
        self.package_info_list = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_package_info_list(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::PackageInfo>> {
        &self.package_info_list
    }

    /// Consumes the builder and constructs a
    /// [`TestGenerationJob`](crate::types::TestGenerationJob). This method will fail if any of
    /// the following fields are not set:
    /// - [`test_generation_job_id`](crate::types::builders::TestGenerationJobBuilder::test_generation_job_id)
    /// - [`test_generation_job_group_name`](crate::types::builders::TestGenerationJobBuilder::test_generation_job_group_name)
    /// - [`status`](crate::types::builders::TestGenerationJobBuilder::status)
    /// - [`creation_time`](crate::types::builders::TestGenerationJobBuilder::creation_time)
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::types::TestGenerationJob, ::aws_smithy_types::error::operation::BuildError> {
        ::std::result::Result::Ok(crate::types::TestGenerationJob {
            test_generation_job_id: self.test_generation_job_id.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "test_generation_job_id",
                    "test_generation_job_id was not specified but it is required when building TestGenerationJob",
                )
            })?,
            test_generation_job_group_name: self.test_generation_job_group_name.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "test_generation_job_group_name",
                    "test_generation_job_group_name was not specified but it is required when building TestGenerationJob",
                )
            })?,
            status: self.status.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "status",
                    "status was not specified but it is required when building TestGenerationJob",
                )
            })?,
            short_answer: self.short_answer,
            creation_time: self.creation_time.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "creation_time",
                    "creation_time was not specified but it is required when building TestGenerationJob",
                )
            })?,
            progress_rate: self.progress_rate,
            job_status_reason: self.job_status_reason,
            job_summary: self.job_summary,
            job_plan: self.job_plan,
            package_info_list: self.package_info_list,
        })
    }
}
impl ::std::fmt::Debug for TestGenerationJobBuilder {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut formatter = f.debug_struct("TestGenerationJobBuilder");
        formatter.field("test_generation_job_id", &self.test_generation_job_id);
        formatter.field("test_generation_job_group_name", &self.test_generation_job_group_name);
        formatter.field("status", &self.status);
        formatter.field("short_answer", &"*** Sensitive Data Redacted ***");
        formatter.field("creation_time", &self.creation_time);
        formatter.field("progress_rate", &self.progress_rate);
        formatter.field("job_status_reason", &self.job_status_reason);
        formatter.field("job_summary", &"*** Sensitive Data Redacted ***");
        formatter.field("job_plan", &"*** Sensitive Data Redacted ***");
        formatter.field("package_info_list", &self.package_info_list);
        formatter.finish()
    }
}
