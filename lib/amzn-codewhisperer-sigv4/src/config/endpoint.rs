// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use ::aws_smithy_http::endpoint::{
    ResolveEndpoint,
    SharedEndpointResolver,
};

///
#[cfg(test)]
mod test {}

#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
/// Configuration parameters for resolving the correct endpoint
pub struct Params {}
impl Params {
    /// Create a builder for [`Params`]
    pub fn builder() -> crate::config::endpoint::ParamsBuilder {
        crate::config::endpoint::ParamsBuilder::default()
    }
}

#[derive(Debug)]
pub(crate) struct MissingResolver;
impl MissingResolver {
    pub(crate) fn new() -> Self {
        Self
    }
}
impl<T> ::aws_smithy_http::endpoint::ResolveEndpoint<T> for MissingResolver {
    fn resolve_endpoint(&self, _params: &T) -> ::aws_smithy_http::endpoint::Result {
        Err(::aws_smithy_http::endpoint::ResolveEndpointError::message(
            "an endpoint resolver must be provided.",
        ))
    }
}

/// Builder for [`Params`]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct ParamsBuilder {}
impl ParamsBuilder {
    /// Consume this builder, creating [`Params`].
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::config::endpoint::Params, crate::config::endpoint::InvalidParams> {
        Ok(
            #[allow(clippy::unnecessary_lazy_evaluations)]
            crate::config::endpoint::Params {},
        )
    }
}

/// An error that occurred during endpoint resolution
#[derive(Debug)]
pub struct InvalidParams {
    field: std::borrow::Cow<'static, str>,
}

impl InvalidParams {
    #[allow(dead_code)]
    fn missing(field: &'static str) -> Self {
        Self { field: field.into() }
    }
}

impl std::fmt::Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a required field was missing: `{}`", self.field)
    }
}

impl std::error::Error for InvalidParams {}
