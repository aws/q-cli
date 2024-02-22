// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
/// Orchestration and serialization glue logic for `CreateCustomization`.
#[derive(::std::clone::Clone, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct CreateCustomization;
impl CreateCustomization {
    /// Creates a new `CreateCustomization`
    pub fn new() -> Self {
        Self
    }

    pub(crate) async fn orchestrate(
        runtime_plugins: &::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        input: crate::operation::create_customization::CreateCustomizationInput,
    ) -> ::std::result::Result<
        crate::operation::create_customization::CreateCustomizationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_customization::CreateCustomizationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let map_err = |err: ::aws_smithy_runtime_api::client::result::SdkError<
            ::aws_smithy_runtime_api::client::interceptors::context::Error,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >| {
            err.map_service_error(|err| {
                err.downcast::<crate::operation::create_customization::CreateCustomizationError>()
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
                .downcast::<crate::operation::create_customization::CreateCustomizationOutput>()
                .expect("correct output type"),
        )
    }

    pub(crate) async fn orchestrate_with_stop_point(
        runtime_plugins: &::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        input: crate::operation::create_customization::CreateCustomizationInput,
        stop_point: ::aws_smithy_runtime::client::orchestrator::StopPoint,
    ) -> ::std::result::Result<
        ::aws_smithy_runtime_api::client::interceptors::context::InterceptorContext,
        ::aws_smithy_runtime_api::client::result::SdkError<
            ::aws_smithy_runtime_api::client::interceptors::context::Error,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = ::aws_smithy_runtime_api::client::interceptors::context::Input::erase(input);
        ::aws_smithy_runtime::client::orchestrator::invoke_with_stop_point(
            "codewhisperer",
            "CreateCustomization",
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
        runtime_plugins = runtime_plugins.with_client_plugin(crate::auth_plugin::DefaultAuthOptionsPlugin::new(vec![
            ::aws_runtime::auth::sigv4::SCHEME_ID,
        ]));
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
impl ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin for CreateCustomization {
    fn config(&self) -> ::std::option::Option<::aws_smithy_types::config_bag::FrozenLayer> {
        let mut cfg = ::aws_smithy_types::config_bag::Layer::new("CreateCustomization");

        cfg.store_put(::aws_smithy_runtime_api::client::ser_de::SharedRequestSerializer::new(
            CreateCustomizationRequestSerializer,
        ));
        cfg.store_put(
            ::aws_smithy_runtime_api::client::ser_de::SharedResponseDeserializer::new(
                CreateCustomizationResponseDeserializer,
            ),
        );

        cfg.store_put(
            ::aws_smithy_runtime_api::client::auth::AuthSchemeOptionResolverParams::new(
                ::aws_smithy_runtime_api::client::auth::static_resolver::StaticAuthSchemeOptionResolverParams::new(),
            ),
        );

        cfg.store_put(::aws_smithy_runtime_api::client::orchestrator::Metadata::new(
            "CreateCustomization",
            "codewhisperer",
        ));
        let mut signing_options = ::aws_runtime::auth::SigningOptions::default();
        signing_options.double_uri_encode = true;
        signing_options.content_sha256_header = false;
        signing_options.normalize_uri_path = true;
        signing_options.payload_override = None;

        cfg.store_put(::aws_runtime::auth::SigV4OperationSigningConfig {
            signing_options,
            ..::std::default::Default::default()
        });

        ::std::option::Option::Some(cfg.freeze())
    }

    fn runtime_components(
        &self,
        _: &::aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder,
    ) -> ::std::borrow::Cow<'_, ::aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder> {
        #[allow(unused_mut)]
        let mut rcb = ::aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder::new("CreateCustomization")
            .with_interceptor(
                ::aws_smithy_runtime::client::stalled_stream_protection::StalledStreamProtectionInterceptor::new(
                    ::aws_smithy_runtime::client::stalled_stream_protection::StalledStreamProtectionInterceptorKind::ResponseBody,
                ),
            )
            .with_interceptor(CreateCustomizationEndpointParamsInterceptor)
            .with_retry_classifier(::aws_smithy_runtime::client::retries::classifiers::TransientErrorClassifier::<
                crate::operation::create_customization::CreateCustomizationError,
            >::new())
            .with_retry_classifier(::aws_smithy_runtime::client::retries::classifiers::ModeledAsRetryableClassifier::<
                crate::operation::create_customization::CreateCustomizationError,
            >::new())
            .with_retry_classifier(::aws_runtime::retries::classifiers::AwsErrorCodeClassifier::<
                crate::operation::create_customization::CreateCustomizationError,
            >::new());

        ::std::borrow::Cow::Owned(rcb)
    }
}

#[derive(Debug)]
struct CreateCustomizationResponseDeserializer;
impl ::aws_smithy_runtime_api::client::ser_de::DeserializeResponse for CreateCustomizationResponseDeserializer {
    fn deserialize_nonstreaming(
        &self,
        response: &::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
    ) -> ::aws_smithy_runtime_api::client::interceptors::context::OutputOrError {
        let (success, status) = (response.status().is_success(), response.status().as_u16());
        let headers = response.headers();
        let body = response.body().bytes().expect("body loaded");
        #[allow(unused_mut)]
        let mut force_error = false;
        ::tracing::debug!(request_id = ?::aws_types::request_id::RequestId::request_id(response));
        let parse_result = if !success && status != 200 || force_error {
            crate::protocol_serde::shape_create_customization::de_create_customization_http_error(status, headers, body)
        } else {
            crate::protocol_serde::shape_create_customization::de_create_customization_http_response(
                status, headers, body,
            )
        };
        crate::protocol_serde::type_erase_result(parse_result)
    }
}
#[derive(Debug)]
struct CreateCustomizationRequestSerializer;
impl ::aws_smithy_runtime_api::client::ser_de::SerializeRequest for CreateCustomizationRequestSerializer {
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
            .downcast::<crate::operation::create_customization::CreateCustomizationInput>()
            .expect("correct type");
        let _header_serialization_settings = _cfg
            .load::<crate::serialization_settings::HeaderSerializationSettings>()
            .cloned()
            .unwrap_or_default();
        let mut request_builder = {
            fn uri_base(
                _input: &crate::operation::create_customization::CreateCustomizationInput,
                output: &mut ::std::string::String,
            ) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::BuildError> {
                use ::std::fmt::Write as _;
                ::std::write!(output, "/").expect("formatting should succeed");
                ::std::result::Result::Ok(())
            }
            #[allow(clippy::unnecessary_wraps)]
            fn update_http_builder(
                input: &crate::operation::create_customization::CreateCustomizationInput,
                builder: ::http::request::Builder,
            ) -> ::std::result::Result<::http::request::Builder, ::aws_smithy_types::error::operation::BuildError>
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
                "AWSCodeWhispererService.CreateCustomization",
            );
            builder
        };
        let body = ::aws_smithy_types::body::SdkBody::from(
            crate::protocol_serde::shape_create_customization::ser_create_customization_input(&input)?,
        );
        if let Some(content_length) = body.content_length() {
            let content_length = content_length.to_string();
            request_builder = _header_serialization_settings.set_default_header(
                request_builder,
                ::http::header::CONTENT_LENGTH,
                &content_length,
            );
        }
        ::std::result::Result::Ok(request_builder.body(body).expect("valid request").try_into().unwrap())
    }
}
#[derive(Debug)]
struct CreateCustomizationEndpointParamsInterceptor;

impl ::aws_smithy_runtime_api::client::interceptors::Intercept for CreateCustomizationEndpointParamsInterceptor {
    fn name(&self) -> &'static str {
        "CreateCustomizationEndpointParamsInterceptor"
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
            .downcast_ref::<CreateCustomizationInput>()
            .ok_or("failed to downcast to CreateCustomizationInput")?;

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

/// Error type for the `CreateCustomizationError` operation.
#[non_exhaustive]
#[derive(::std::fmt::Debug)]
pub enum CreateCustomizationError {
    /// This exception is thrown when an unexpected error occurred during the processing of a
    /// request.
    InternalServerError(crate::types::error::InternalServerError),
    /// This exception is thrown when request was denied due to request throttling.
    ThrottlingError(crate::types::error::ThrottlingError),
    /// This exception is thrown when the input fails to satisfy the constraints specified by the
    /// service.
    ValidationError(crate::types::error::ValidationError),
    /// This exception is thrown when the action to perform could not be completed because the
    /// resource is in a conflicting state.
    ConflictError(crate::types::error::ConflictError),
    /// This exception is thrown when the user does not have sufficient access to perform this
    /// action.
    AccessDeniedError(crate::types::error::AccessDeniedError),
    /// An unexpected error occurred (e.g., invalid JSON returned by the service or an unknown error
    /// code).
    #[deprecated(
        note = "Matching `Unhandled` directly is not forwards compatible. Instead, match using a \
    variable wildcard pattern and check `.code()`:
     \
    &nbsp;&nbsp;&nbsp;`err if err.code() == Some(\"SpecificExceptionCode\") => { /* handle the error */ }`
     \
    See [`ProvideErrorMetadata`](#impl-ProvideErrorMetadata-for-CreateCustomizationError) for what information is available for the error."
    )]
    Unhandled(crate::error::sealed_unhandled::Unhandled),
}
impl CreateCustomizationError {
    /// Creates the `CreateCustomizationError::Unhandled` variant from any error type.
    pub fn unhandled(
        err: impl ::std::convert::Into<
            ::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Send + ::std::marker::Sync + 'static>,
        >,
    ) -> Self {
        Self::Unhandled(crate::error::sealed_unhandled::Unhandled {
            source: err.into(),
            meta: ::std::default::Default::default(),
        })
    }

    /// Creates the `CreateCustomizationError::Unhandled` variant from an
    /// [`ErrorMetadata`](::aws_smithy_types::error::ErrorMetadata).
    pub fn generic(err: ::aws_smithy_types::error::ErrorMetadata) -> Self {
        Self::Unhandled(crate::error::sealed_unhandled::Unhandled {
            source: err.clone().into(),
            meta: err,
        })
    }

    /// Returns error metadata, which includes the error code, message,
    /// request ID, and potentially additional information.
    pub fn meta(&self) -> &::aws_smithy_types::error::ErrorMetadata {
        match self {
            Self::InternalServerError(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::ThrottlingError(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::ValidationError(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::ConflictError(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::AccessDeniedError(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::Unhandled(e) => &e.meta,
        }
    }

    /// Returns `true` if the error kind is `CreateCustomizationError::InternalServerError`.
    pub fn is_internal_server_error(&self) -> bool {
        matches!(self, Self::InternalServerError(_))
    }

    /// Returns `true` if the error kind is `CreateCustomizationError::ThrottlingError`.
    pub fn is_throttling_error(&self) -> bool {
        matches!(self, Self::ThrottlingError(_))
    }

    /// Returns `true` if the error kind is `CreateCustomizationError::ValidationError`.
    pub fn is_validation_error(&self) -> bool {
        matches!(self, Self::ValidationError(_))
    }

    /// Returns `true` if the error kind is `CreateCustomizationError::ConflictError`.
    pub fn is_conflict_error(&self) -> bool {
        matches!(self, Self::ConflictError(_))
    }

    /// Returns `true` if the error kind is `CreateCustomizationError::AccessDeniedError`.
    pub fn is_access_denied_error(&self) -> bool {
        matches!(self, Self::AccessDeniedError(_))
    }
}
impl ::std::error::Error for CreateCustomizationError {
    fn source(&self) -> ::std::option::Option<&(dyn ::std::error::Error + 'static)> {
        match self {
            Self::InternalServerError(_inner) => ::std::option::Option::Some(_inner),
            Self::ThrottlingError(_inner) => ::std::option::Option::Some(_inner),
            Self::ValidationError(_inner) => ::std::option::Option::Some(_inner),
            Self::ConflictError(_inner) => ::std::option::Option::Some(_inner),
            Self::AccessDeniedError(_inner) => ::std::option::Option::Some(_inner),
            Self::Unhandled(_inner) => ::std::option::Option::Some(&*_inner.source),
        }
    }
}
impl ::std::fmt::Display for CreateCustomizationError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::InternalServerError(_inner) => _inner.fmt(f),
            Self::ThrottlingError(_inner) => _inner.fmt(f),
            Self::ValidationError(_inner) => _inner.fmt(f),
            Self::ConflictError(_inner) => _inner.fmt(f),
            Self::AccessDeniedError(_inner) => _inner.fmt(f),
            Self::Unhandled(_inner) => {
                if let ::std::option::Option::Some(code) =
                    ::aws_smithy_types::error::metadata::ProvideErrorMetadata::code(self)
                {
                    write!(f, "unhandled error ({code})")
                } else {
                    f.write_str("unhandled error")
                }
            },
        }
    }
}
impl ::aws_smithy_types::retry::ProvideErrorKind for CreateCustomizationError {
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
impl ::aws_smithy_types::error::metadata::ProvideErrorMetadata for CreateCustomizationError {
    fn meta(&self) -> &::aws_smithy_types::error::ErrorMetadata {
        match self {
            Self::InternalServerError(_inner) => {
                ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner)
            },
            Self::ThrottlingError(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::ValidationError(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::ConflictError(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::AccessDeniedError(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::Unhandled(_inner) => &_inner.meta,
        }
    }
}
impl ::aws_smithy_runtime_api::client::result::CreateUnhandledError for CreateCustomizationError {
    fn create_unhandled_error(
        source: ::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Send + ::std::marker::Sync + 'static>,
        meta: ::std::option::Option<::aws_smithy_types::error::ErrorMetadata>,
    ) -> Self {
        Self::Unhandled(crate::error::sealed_unhandled::Unhandled {
            source,
            meta: meta.unwrap_or_default(),
        })
    }
}
impl ::aws_types::request_id::RequestId for crate::operation::create_customization::CreateCustomizationError {
    fn request_id(&self) -> Option<&str> {
        self.meta().request_id()
    }
}

pub use crate::operation::create_customization::_create_customization_input::CreateCustomizationInput;
pub use crate::operation::create_customization::_create_customization_output::CreateCustomizationOutput;

mod _create_customization_input;

mod _create_customization_output;

/// Builders
pub mod builders;
