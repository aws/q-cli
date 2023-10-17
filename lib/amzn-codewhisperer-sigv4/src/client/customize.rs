// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

pub(crate) mod internal {
    pub type BoxFuture<T> =
        ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = T> + ::std::marker::Send>>;

    pub type SendResult<T, E> = ::std::result::Result<
        T,
        ::aws_smithy_http::result::SdkError<E, ::aws_smithy_runtime_api::client::orchestrator::HttpResponse>,
    >;

    pub trait CustomizableSend<T, E>: ::std::marker::Send + ::std::marker::Sync {
        // Takes an owned `self` as the implementation will internally call methods that take `self`.
        // If it took `&self`, that would make this trait object safe, but some implementing types do not
        // derive `Clone`, unable to yield `self` from `&self`.
        fn send(self, config_override: crate::config::Builder) -> BoxFuture<SendResult<T, E>>;
    }
}
/// Module for defining types for `CustomizableOperation` in the orchestrator
pub mod orchestrator {
    /// `CustomizableOperation` allows for configuring a single operation invocation before it is
    /// sent.
    pub struct CustomizableOperation<T, E, B> {
        customizable_send: B,
        config_override: ::std::option::Option<crate::config::Builder>,
        interceptors: Vec<::aws_smithy_runtime_api::client::interceptors::SharedInterceptor>,
        runtime_plugins: Vec<::aws_smithy_runtime_api::client::runtime_plugin::SharedRuntimePlugin>,
        _output: ::std::marker::PhantomData<T>,
        _error: ::std::marker::PhantomData<E>,
    }

    impl<T, E, B> CustomizableOperation<T, E, B> {
        /// Creates a new `CustomizableOperation` from `customizable_send`.
        pub(crate) fn new(customizable_send: B) -> Self {
            Self {
                customizable_send,
                config_override: ::std::option::Option::None,
                interceptors: vec![],
                runtime_plugins: vec![],
                _output: ::std::marker::PhantomData,
                _error: ::std::marker::PhantomData,
            }
        }

        /// Adds an [`Interceptor`](::aws_smithy_runtime_api::client::interceptors::Interceptor)
        /// that runs at specific stages of the request execution pipeline.
        ///
        /// Note that interceptors can also be added to `CustomizableOperation` by
        /// `config_override`, `map_request`, and `mutate_request` (the last two are
        /// implemented via interceptors under the hood). The order in which those
        /// user-specified operation interceptors are invoked should not be relied upon
        /// as it is an implementation detail.
        pub fn interceptor(
            mut self,
            interceptor: impl ::aws_smithy_runtime_api::client::interceptors::Interceptor + 'static,
        ) -> Self {
            self.interceptors
                .push(::aws_smithy_runtime_api::client::interceptors::SharedInterceptor::new(
                    interceptor,
                ));
            self
        }

        /// Adds a runtime plugin.
        #[allow(unused)]
        pub(crate) fn runtime_plugin(
            mut self,
            runtime_plugin: impl ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin + 'static,
        ) -> Self {
            self.runtime_plugins
                .push(::aws_smithy_runtime_api::client::runtime_plugin::SharedRuntimePlugin::new(runtime_plugin));
            self
        }

        /// Allows for customizing the operation's request.
        pub fn map_request<F, MapE>(mut self, f: F) -> Self
        where
            F: ::std::ops::Fn(
                    ::aws_smithy_runtime_api::client::orchestrator::HttpRequest,
                ) -> ::std::result::Result<
                    ::aws_smithy_runtime_api::client::orchestrator::HttpRequest,
                    MapE,
                > + ::std::marker::Send
                + ::std::marker::Sync
                + 'static,
            MapE: ::std::error::Error + ::std::marker::Send + ::std::marker::Sync + 'static,
        {
            self.interceptors
                .push(::aws_smithy_runtime_api::client::interceptors::SharedInterceptor::new(
                    ::aws_smithy_runtime::client::interceptors::MapRequestInterceptor::new(f),
                ));
            self
        }

        /// Convenience for `map_request` where infallible direct mutation of request is acceptable.
        pub fn mutate_request<F>(mut self, f: F) -> Self
        where
            F: ::std::ops::Fn(&mut ::aws_smithy_runtime_api::client::orchestrator::HttpRequest)
                + ::std::marker::Send
                + ::std::marker::Sync
                + 'static,
        {
            self.interceptors
                .push(::aws_smithy_runtime_api::client::interceptors::SharedInterceptor::new(
                    ::aws_smithy_runtime::client::interceptors::MutateRequestInterceptor::new(f),
                ));
            self
        }

        /// Overrides config for a single operation invocation.
        ///
        /// `config_override` is applied to the operation configuration level.
        /// The fields in the builder that are `Some` override those applied to the service
        /// configuration level. For instance,
        ///
        /// Config A     overridden by    Config B          ==        Config C
        /// field_1: None,                field_1: Some(v2),          field_1: Some(v2),
        /// field_2: Some(v1),            field_2: Some(v2),          field_2: Some(v2),
        /// field_3: Some(v1),            field_3: None,              field_3: Some(v1),
        pub fn config_override(mut self, config_override: impl ::std::convert::Into<crate::config::Builder>) -> Self {
            self.config_override = Some(config_override.into());
            self
        }

        /// Sends the request and returns the response.
        pub async fn send(self) -> crate::client::customize::internal::SendResult<T, E>
        where
            E: std::error::Error + ::std::marker::Send + ::std::marker::Sync + 'static,
            B: crate::client::customize::internal::CustomizableSend<T, E>,
        {
            let mut config_override = if let Some(config_override) = self.config_override {
                config_override
            } else {
                crate::config::Builder::new()
            };

            self.interceptors.into_iter().for_each(|interceptor| {
                config_override.push_interceptor(interceptor);
            });
            self.runtime_plugins.into_iter().for_each(|plugin| {
                config_override.push_runtime_plugin(plugin);
            });

            self.customizable_send.send(config_override).await
        }

        #[doc(hidden)]
        // This is a temporary method for testing. NEVER use it in production
        pub fn request_time_for_tests(self, request_time: ::std::time::SystemTime) -> Self {
            self.runtime_plugin(
                ::aws_smithy_runtime_api::client::runtime_plugin::StaticRuntimePlugin::new().with_runtime_components(
                    ::aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder::new(
                        "request_time_for_tests",
                    )
                    .with_time_source(Some(::aws_smithy_async::time::SharedTimeSource::new(
                        ::aws_smithy_async::time::StaticTimeSource::new(request_time),
                    ))),
                ),
            )
        }

        #[doc(hidden)]
        // This is a temporary method for testing. NEVER use it in production
        pub fn user_agent_for_tests(mut self) -> Self {
            let interceptor = crate::client::customize::TestParamsSetterInterceptor::new(
                |context: &mut ::aws_smithy_runtime_api::client::interceptors::context::BeforeTransmitInterceptorContextMut<'_>,
                 _: &mut ::aws_smithy_types::config_bag::ConfigBag| {
                    let headers = context.request_mut().headers_mut();
                    let user_agent = ::aws_http::user_agent::AwsUserAgent::for_tests();
                    headers.insert(::http::header::USER_AGENT, ::http::HeaderValue::try_from(user_agent.ua_header()).unwrap());
                    headers.insert(
                        ::http::HeaderName::from_static("x-amz-user-agent"),
                        ::http::HeaderValue::try_from(user_agent.aws_ua_header()).unwrap(),
                    );
                },
            );
            self.interceptors
                .push(::aws_smithy_runtime_api::client::interceptors::SharedInterceptor::new(
                    interceptor,
                ));
            self
        }

        #[doc(hidden)]
        // This is a temporary method for testing. NEVER use it in production
        pub fn remove_invocation_id_for_tests(mut self) -> Self {
            let interceptor = crate::client::customize::TestParamsSetterInterceptor::new(
                |context: &mut ::aws_smithy_runtime_api::client::interceptors::context::BeforeTransmitInterceptorContextMut<'_>,
                 _: &mut ::aws_smithy_types::config_bag::ConfigBag| {
                    context.request_mut().headers_mut().remove("amz-sdk-invocation-id");
                },
            );
            self.interceptors
                .push(::aws_smithy_runtime_api::client::interceptors::SharedInterceptor::new(
                    interceptor,
                ));
            self
        }
    }
}

mod test_params_setter_interceptor {
    use std::fmt;

    use aws_smithy_runtime_api::box_error::BoxError;
    use aws_smithy_runtime_api::client::interceptors::context::BeforeTransmitInterceptorContextMut;
    use aws_smithy_runtime_api::client::interceptors::Interceptor;
    use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
    use aws_smithy_types::config_bag::ConfigBag;

    pub(super) struct TestParamsSetterInterceptor<F> {
        f: F,
    }

    impl<F> fmt::Debug for TestParamsSetterInterceptor<F> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "TestParamsSetterInterceptor")
        }
    }

    impl<F> TestParamsSetterInterceptor<F> {
        pub fn new(f: F) -> Self {
            Self { f }
        }
    }

    impl<F> Interceptor for TestParamsSetterInterceptor<F>
    where
        F: Fn(&mut BeforeTransmitInterceptorContextMut<'_>, &mut ConfigBag) + Send + Sync + 'static,
    {
        fn name(&self) -> &'static str {
            "TestParamsSetterInterceptor"
        }

        fn modify_before_signing(
            &self,
            context: &mut BeforeTransmitInterceptorContextMut<'_>,
            _runtime_components: &RuntimeComponents,
            cfg: &mut ConfigBag,
        ) -> Result<(), BoxError> {
            (self.f)(context, cfg);
            Ok(())
        }
    }
}
use test_params_setter_interceptor::TestParamsSetterInterceptor;
