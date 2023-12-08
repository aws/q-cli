// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct ListFeatureEvaluationsInput {
    #[allow(missing_docs)] // documentation missing in model
    pub user_context: ::std::option::Option<crate::types::UserContext>,
}
impl ListFeatureEvaluationsInput {
    #[allow(missing_docs)] // documentation missing in model
    pub fn user_context(&self) -> ::std::option::Option<&crate::types::UserContext> {
        self.user_context.as_ref()
    }
}
impl ListFeatureEvaluationsInput {
    /// Creates a new builder-style object to manufacture
    /// [`ListFeatureEvaluationsInput`](crate::operation::list_feature_evaluations::ListFeatureEvaluationsInput).
    pub fn builder() -> crate::operation::list_feature_evaluations::builders::ListFeatureEvaluationsInputBuilder {
        crate::operation::list_feature_evaluations::builders::ListFeatureEvaluationsInputBuilder::default()
    }
}

/// A builder for
/// [`ListFeatureEvaluationsInput`](crate::operation::list_feature_evaluations::ListFeatureEvaluationsInput).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct ListFeatureEvaluationsInputBuilder {
    pub(crate) user_context: ::std::option::Option<crate::types::UserContext>,
}
impl ListFeatureEvaluationsInputBuilder {
    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn user_context(mut self, input: crate::types::UserContext) -> Self {
        self.user_context = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_user_context(mut self, input: ::std::option::Option<crate::types::UserContext>) -> Self {
        self.user_context = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_user_context(&self) -> &::std::option::Option<crate::types::UserContext> {
        &self.user_context
    }

    /// Consumes the builder and constructs a
    /// [`ListFeatureEvaluationsInput`](crate::operation::list_feature_evaluations::ListFeatureEvaluationsInput).
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::list_feature_evaluations::ListFeatureEvaluationsInput,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(
            crate::operation::list_feature_evaluations::ListFeatureEvaluationsInput {
                user_context: self.user_context,
            },
        )
    }
}
