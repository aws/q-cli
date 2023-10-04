// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

impl ListCustomizationsInput {}
/// Orchestration and serialization glue logic for `ListCustomizations`.
#[derive(::std::clone::Clone, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
#[doc(hidden)]
pub struct ListCustomizations;
impl ListCustomizations {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self
    }

    pub(crate) async fn orchestrate(
        runtime_plugins: &::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        input: crate::operation::list_customizations::ListCustomizationsInput,
    ) -> ::std::result::Result<
        crate::operation::list_customizations::ListCustomizationsOutput,
        ::aws_smithy_http::result::SdkError<
            crate::operation::list_customizations::ListCustomizationsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let map_err = |err: ::aws_smithy_http::result::SdkError<
            ::aws_smithy_runtime_api::client::interceptors::context::Error,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >| {
            err.map_service_error(|err| {
                err.downcast::<crate::operation::list_customizations::ListCustomizationsError>()
                    .expect("correct error type")
            })
        };
        let context = Self::orchestrate_with_stop_point(
            runtime_plugins,
            input,
            ::aws_smithy_runtime::client::orchestrator::StopPoint::None,
        )
        .await
        .map_err(map_err)?;
        let output = context.finalize().map_err(map_err)?;
        ::std::result::Result::Ok(
            output
                .downcast::<crate::operation::list_customizations::ListCustomizationsOutput>()
                .expect("correct output type"),
        )
    }

    pub(crate) async fn orchestrate_with_stop_point(
        runtime_plugins: &::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        input: crate::operation::list_customizations::ListCustomizationsInput,
        stop_point: ::aws_smithy_runtime::client::orchestrator::StopPoint,
    ) -> ::std::result::Result<
        ::aws_smithy_runtime_api::client::interceptors::context::InterceptorContext,
        ::aws_smithy_http::result::SdkError<
            ::aws_smithy_runtime_api::client::interceptors::context::Error,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = ::aws_smithy_runtime_api::client::interceptors::context::Input::erase(input);
        ::aws_smithy_runtime::client::orchestrator::invoke_with_stop_point(
            "codewhisperer",
            "ListCustomizations",
            input,
            runtime_plugins,
            stop_point,
        )
        .await
    }

    pub(crate) fn operation_runtime_plugins(
        client_runtime_plugins: ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        client_config: &crate::config::Config,
        config_override: ::std::option::Option<crate::config::Builder>,
    ) -> ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins {
        let mut runtime_plugins = client_runtime_plugins.with_operation_plugin(Self::new());

        if let ::std::option::Option::Some(config_override) = config_override {
            for plugin in config_override.runtime_plugins.iter().cloned() {
                runtime_plugins = runtime_plugins.with_operation_plugin(plugin);
            }
            runtime_plugins = runtime_plugins.with_operation_plugin(crate::config::ConfigOverrideRuntimePlugin::new(
                config_override,
                client_config.config.clone(),
                &client_config.runtime_components,
            ));
        }
        runtime_plugins
    }
}
impl ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin for ListCustomizations {
    fn config(&self) -> ::std::option::Option<::aws_smithy_types::config_bag::FrozenLayer> {
        let mut cfg = ::aws_smithy_types::config_bag::Layer::new("ListCustomizations");

        cfg.store_put(::aws_smithy_runtime_api::client::ser_de::SharedRequestSerializer::new(
            ListCustomizationsRequestSerializer,
        ));
        cfg.store_put(
            ::aws_smithy_runtime_api::client::ser_de::SharedResponseDeserializer::new(
                ListCustomizationsResponseDeserializer,
            ),
        );

        cfg.store_put(
            ::aws_smithy_runtime_api::client::auth::AuthSchemeOptionResolverParams::new(
                ::aws_smithy_runtime_api::client::auth::static_resolver::StaticAuthSchemeOptionResolverParams::new(),
            ),
        );

        cfg.store_put(::aws_smithy_http::operation::Metadata::new(
            "ListCustomizations",
            "codewhisperer",
        ));
        let mut signing_options = ::aws_runtime::auth::sigv4::SigningOptions::default();
        signing_options.double_uri_encode = true;
        signing_options.content_sha256_header = false;
        signing_options.normalize_uri_path = true;
        signing_options.payload_override = None;

        cfg.store_put(::aws_runtime::auth::sigv4::SigV4OperationSigningConfig {
            region: None,
            service: None,
            signing_options,
        });

        ::std::option::Option::Some(cfg.freeze())
    }

    fn runtime_components(
        &self,
    ) -> ::std::borrow::Cow<'_, ::aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder> {
        // Retry classifiers are operation-specific because they need to downcast operation-specific error
        // types.
        let retry_classifiers = ::aws_smithy_runtime_api::client::retries::RetryClassifiers::new()
            .with_classifier(
                ::aws_smithy_runtime::client::retries::classifier::SmithyErrorClassifier::<
                    crate::operation::list_customizations::ListCustomizationsError,
                >::new(),
            )
            .with_classifier(::aws_runtime::retries::classifier::AmzRetryAfterHeaderClassifier)
            .with_classifier(
                ::aws_smithy_runtime::client::retries::classifier::ModeledAsRetryableClassifier::<
                    crate::operation::list_customizations::ListCustomizationsError,
                >::new(),
            )
            .with_classifier(::aws_runtime::retries::classifier::AwsErrorCodeClassifier::<
                crate::operation::list_customizations::ListCustomizationsError,
            >::new())
            .with_classifier(::aws_smithy_runtime::client::retries::classifier::HttpStatusCodeClassifier::default());

        ::std::borrow::Cow::Owned(
            ::aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder::new("ListCustomizations")
                .with_retry_classifiers(::std::option::Option::Some(retry_classifiers))
                .with_auth_scheme_option_resolver(::std::option::Option::Some(
                    ::aws_smithy_runtime_api::client::auth::SharedAuthSchemeOptionResolver::new(
                        ::aws_smithy_runtime_api::client::auth::static_resolver::StaticAuthSchemeOptionResolver::new(
                            vec![::aws_runtime::auth::sigv4::SCHEME_ID],
                        ),
                    ),
                ))
                .with_interceptor(::aws_smithy_runtime_api::client::interceptors::SharedInterceptor::new(
                    ListCustomizationsEndpointParamsInterceptor,
                ) as _),
        )
    }
}

#[derive(Debug)]
struct ListCustomizationsResponseDeserializer;
impl ::aws_smithy_runtime_api::client::ser_de::ResponseDeserializer for ListCustomizationsResponseDeserializer {
    fn deserialize_nonstreaming(
        &self,
        response: &::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
    ) -> ::aws_smithy_runtime_api::client::interceptors::context::OutputOrError {
        let (success, status) = (response.status().is_success(), response.status().as_u16());
        let headers = response.headers();
        let body = response.body().bytes().expect("body loaded");
        ::tracing::debug!(request_id = ?::aws_http::request_id::RequestId::request_id(response));
        let parse_result = if !success && status != 200 {
            crate::protocol_serde::shape_list_customizations::de_list_customizations_http_error(status, headers, body)
        } else {
            crate::protocol_serde::shape_list_customizations::de_list_customizations_http_response(
                status, headers, body,
            )
        };
        crate::protocol_serde::type_erase_result(parse_result)
    }
}
#[derive(Debug)]
struct ListCustomizationsRequestSerializer;
impl ::aws_smithy_runtime_api::client::ser_de::RequestSerializer for ListCustomizationsRequestSerializer {
    #[allow(
        unused_mut,
        clippy::let_and_return,
        clippy::needless_borrow,
        clippy::useless_conversion
    )]
    fn serialize_input(
        &self,
        input: ::aws_smithy_runtime_api::client::interceptors::context::Input,
        _cfg: &mut ::aws_smithy_types::config_bag::ConfigBag,
    ) -> ::std::result::Result<
        ::aws_smithy_runtime_api::client::orchestrator::HttpRequest,
        ::aws_smithy_runtime_api::box_error::BoxError,
    > {
        let input = input
            .downcast::<crate::operation::list_customizations::ListCustomizationsInput>()
            .expect("correct type");
        let _header_serialization_settings = _cfg
            .load::<crate::serialization_settings::HeaderSerializationSettings>()
            .cloned()
            .unwrap_or_default();
        let mut request_builder = {
            fn uri_base(
                _input: &crate::operation::list_customizations::ListCustomizationsInput,
                output: &mut ::std::string::String,
            ) -> ::std::result::Result<(), ::aws_smithy_http::operation::error::BuildError> {
                use ::std::fmt::Write as _;
                ::std::write!(output, "/").expect("formatting should succeed");
                ::std::result::Result::Ok(())
            }
            #[allow(clippy::unnecessary_wraps)]
            fn update_http_builder(
                input: &crate::operation::list_customizations::ListCustomizationsInput,
                builder: ::http::request::Builder,
            ) -> ::std::result::Result<::http::request::Builder, ::aws_smithy_http::operation::error::BuildError>
            {
                let mut uri = ::std::string::String::new();
                uri_base(input, &mut uri)?;
                ::std::result::Result::Ok(builder.method("POST").uri(uri))
            }
            let mut builder = update_http_builder(&input, ::http::request::Builder::new())?;
            builder = _header_serialization_settings.set_default_header(
                builder,
                ::http::header::CONTENT_TYPE,
                "application/x-amz-json-1.0",
            );
            builder = _header_serialization_settings.set_default_header(
                builder,
                ::http::header::HeaderName::from_static("x-amz-target"),
                "AWSCodeWhispererService.ListCustomizations",
            );
            builder
        };
        let body = ::aws_smithy_http::body::SdkBody::from(
            crate::protocol_serde::shape_list_customizations::ser_list_customizations_input(&input)?,
        );
        if let Some(content_length) = body.content_length() {
            let content_length = content_length.to_string();
            request_builder = _header_serialization_settings.set_default_header(
                request_builder,
                ::http::header::CONTENT_LENGTH,
                &content_length,
            );
        }
        ::std::result::Result::Ok(request_builder.body(body).expect("valid request"))
    }
}
#[derive(Debug)]
struct ListCustomizationsEndpointParamsInterceptor;

impl ::aws_smithy_runtime_api::client::interceptors::Interceptor for ListCustomizationsEndpointParamsInterceptor {
    fn name(&self) -> &'static str {
        "ListCustomizationsEndpointParamsInterceptor"
    }

    fn read_before_execution(
        &self,
        context: &::aws_smithy_runtime_api::client::interceptors::context::BeforeSerializationInterceptorContextRef<
            '_,
            ::aws_smithy_runtime_api::client::interceptors::context::Input,
            ::aws_smithy_runtime_api::client::interceptors::context::Output,
            ::aws_smithy_runtime_api::client::interceptors::context::Error,
        >,
        cfg: &mut ::aws_smithy_types::config_bag::ConfigBag,
    ) -> ::std::result::Result<(), ::aws_smithy_runtime_api::box_error::BoxError> {
        let _input = context
            .input()
            .downcast_ref::<ListCustomizationsInput>()
            .ok_or("failed to downcast to ListCustomizationsInput")?;

        let params = crate::config::endpoint::Params::builder().build().map_err(|err| {
            ::aws_smithy_runtime_api::client::interceptors::error::ContextAttachedError::new(
                "endpoint params could not be built",
                err,
            )
        })?;
        cfg.interceptor_state()
            .store_put(::aws_smithy_runtime_api::client::endpoint::EndpointResolverParams::new(
                params,
            ));
        ::std::result::Result::Ok(())
    }
}

/// Do not use this.
///
/// Operation `*Error/*ErrorKind` types were combined into a single `*Error` enum. The `.kind` field
/// on `*Error` no longer exists and isn't needed anymore (you can just match on the error directly
/// since it's an enum now).
#[deprecated(
    note = "Operation `*Error/*ErrorKind` types were combined into a single `*Error` enum. The `.kind` field on `*Error` no longer exists and isn't needed anymore (you can just match on the error directly since it's an enum now)."
)]
pub type ListCustomizationsErrorKind = ListCustomizationsError;
/// Error type for the `ListCustomizationsError` operation.
#[non_exhaustive]
#[derive(::std::fmt::Debug)]
pub enum ListCustomizationsError {
    /// This exception is thrown when the input fails to satisfy the constraints specified by the
    /// service.
    ValidationError(crate::types::error::ValidationError),
    /// This exception is thrown when the user does not have sufficient access to perform this
    /// action.
    AccessDeniedError(crate::types::error::AccessDeniedError),
    /// This exception is thrown when an unexpected error occurred during the processing of a
    /// request.
    InternalServerError(crate::types::error::InternalServerError),
    /// This exception is thrown when request was denied due to request throttling.
    ThrottlingError(crate::types::error::ThrottlingError),
    /// An unexpected error occurred (e.g., invalid JSON returned by the service or an unknown error
    /// code).
    Unhandled(::aws_smithy_types::error::Unhandled),
}
impl ::aws_smithy_http::result::CreateUnhandledError for ListCustomizationsError {
    fn create_unhandled_error(
        source: ::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Send + ::std::marker::Sync + 'static>,
        meta: ::std::option::Option<::aws_smithy_types::error::ErrorMetadata>,
    ) -> Self {
        Self::Unhandled({
            let mut builder = ::aws_smithy_types::error::Unhandled::builder().source(source);
            builder.set_meta(meta);
            builder.build()
        })
    }
}
impl ::std::fmt::Display for ListCustomizationsError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::ValidationError(_inner) => _inner.fmt(f),
            Self::AccessDeniedError(_inner) => _inner.fmt(f),
            Self::InternalServerError(_inner) => _inner.fmt(f),
            Self::ThrottlingError(_inner) => _inner.fmt(f),
            Self::Unhandled(_inner) => _inner.fmt(f),
        }
    }
}
impl ::aws_smithy_types::error::metadata::ProvideErrorMetadata for ListCustomizationsError {
    fn meta(&self) -> &::aws_smithy_types::error::ErrorMetadata {
        match self {
            Self::ValidationError(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::AccessDeniedError(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::InternalServerError(_inner) => {
                ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner)
            },
            Self::ThrottlingError(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::Unhandled(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
        }
    }
}
impl ::aws_http::request_id::RequestId for crate::operation::list_customizations::ListCustomizationsError {
    fn request_id(&self) -> Option<&str> {
        self.meta().request_id()
    }
}
impl ::aws_smithy_types::retry::ProvideErrorKind for ListCustomizationsError {
    fn code(&self) -> ::std::option::Option<&str> {
        ::aws_smithy_types::error::metadata::ProvideErrorMetadata::code(self)
    }

    fn retryable_error_kind(&self) -> ::std::option::Option<::aws_smithy_types::retry::ErrorKind> {
        match self {
            Self::InternalServerError(inner) => ::std::option::Option::Some(inner.retryable_error_kind()),
            Self::ThrottlingError(inner) => ::std::option::Option::Some(inner.retryable_error_kind()),
            _ => ::std::option::Option::None,
        }
    }
}
impl ListCustomizationsError {
    /// Creates the `ListCustomizationsError::Unhandled` variant from any error type.
    pub fn unhandled(
        err: impl ::std::convert::Into<
            ::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Send + ::std::marker::Sync + 'static>,
        >,
    ) -> Self {
        Self::Unhandled(::aws_smithy_types::error::Unhandled::builder().source(err).build())
    }

    /// Creates the `ListCustomizationsError::Unhandled` variant from a
    /// `::aws_smithy_types::error::ErrorMetadata`.
    pub fn generic(err: ::aws_smithy_types::error::ErrorMetadata) -> Self {
        Self::Unhandled(
            ::aws_smithy_types::error::Unhandled::builder()
                .source(err.clone())
                .meta(err)
                .build(),
        )
    }

    /// Returns error metadata, which includes the error code, message,
    /// request ID, and potentially additional information.
    pub fn meta(&self) -> &::aws_smithy_types::error::ErrorMetadata {
        use ::aws_smithy_types::error::metadata::ProvideErrorMetadata;
        match self {
            Self::ValidationError(e) => e.meta(),
            Self::AccessDeniedError(e) => e.meta(),
            Self::InternalServerError(e) => e.meta(),
            Self::ThrottlingError(e) => e.meta(),
            Self::Unhandled(e) => e.meta(),
        }
    }

    /// Returns `true` if the error kind is `ListCustomizationsError::ValidationError`.
    pub fn is_validation_error(&self) -> bool {
        matches!(self, Self::ValidationError(_))
    }

    /// Returns `true` if the error kind is `ListCustomizationsError::AccessDeniedError`.
    pub fn is_access_denied_error(&self) -> bool {
        matches!(self, Self::AccessDeniedError(_))
    }

    /// Returns `true` if the error kind is `ListCustomizationsError::InternalServerError`.
    pub fn is_internal_server_error(&self) -> bool {
        matches!(self, Self::InternalServerError(_))
    }

    /// Returns `true` if the error kind is `ListCustomizationsError::ThrottlingError`.
    pub fn is_throttling_error(&self) -> bool {
        matches!(self, Self::ThrottlingError(_))
    }
}
impl ::std::error::Error for ListCustomizationsError {
    fn source(&self) -> ::std::option::Option<&(dyn ::std::error::Error + 'static)> {
        match self {
            Self::ValidationError(_inner) => ::std::option::Option::Some(_inner),
            Self::AccessDeniedError(_inner) => ::std::option::Option::Some(_inner),
            Self::InternalServerError(_inner) => ::std::option::Option::Some(_inner),
            Self::ThrottlingError(_inner) => ::std::option::Option::Some(_inner),
            Self::Unhandled(_inner) => ::std::option::Option::Some(_inner),
        }
    }
}

pub use crate::operation::list_customizations::_list_customizations_input::ListCustomizationsInput;
pub use crate::operation::list_customizations::_list_customizations_output::ListCustomizationsOutput;

mod _list_customizations_input;

mod _list_customizations_output;

/// Builders
pub mod builders;

/// Paginator for this operation
pub mod paginator;
