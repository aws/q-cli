// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

pub(crate) mod internal {
    pub type BoxFuture<T> = ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = T> + ::std::marker::Send>>;

    pub type SendResult<T, E> =
        ::std::result::Result<T, ::aws_smithy_http::result::SdkError<E, ::aws_smithy_runtime_api::client::orchestrator::HttpResponse>>;

    pub trait CustomizableSend<T, E>: ::std::ops::FnOnce(crate::config::Builder) -> BoxFuture<SendResult<T, E>> {}

    impl<F, T, E> CustomizableSend<T, E> for F where F: ::std::ops::FnOnce(crate::config::Builder) -> BoxFuture<SendResult<T, E>> {}
}
/// Module for defining types for `CustomizableOperation` in the orchestrator
pub mod orchestrator {
    /// `CustomizableOperation` allows for configuring a single operation invocation before it is sent.
    pub struct CustomizableOperation<T, E> {
        pub(crate) customizable_send: ::std::boxed::Box<dyn crate::client::customize::internal::CustomizableSend<T, E>>,
        pub(crate) config_override: ::std::option::Option<crate::config::Builder>,
        pub(crate) interceptors: Vec<::aws_smithy_runtime_api::client::interceptors::SharedInterceptor>,
        pub(crate) runtime_plugins: Vec<::aws_smithy_runtime_api::client::runtime_plugin::SharedRuntimePlugin>,
    }

    impl<T, E> CustomizableOperation<T, E> {
        /// Adds an [`Interceptor`](::aws_smithy_runtime_api::client::interceptors::Interceptor) that runs at specific stages of the request execution pipeline.
        ///
        /// Note that interceptors can also be added to `CustomizableOperation` by `config_override`,
        /// `map_request`, and `mutate_request` (the last two are implemented via interceptors under the hood).
        /// The order in which those user-specified operation interceptors are invoked should not be relied upon
        /// as it is an implementation detail.
        pub fn interceptor(mut self, interceptor: impl ::aws_smithy_runtime_api::client::interceptors::Interceptor + 'static) -> Self {
            self.interceptors
                .push(::aws_smithy_runtime_api::client::interceptors::SharedInterceptor::new(interceptor));
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
                ) -> ::std::result::Result<::aws_smithy_runtime_api::client::orchestrator::HttpRequest, MapE>
                + ::std::marker::Send
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
            F: ::std::ops::Fn(&mut http::Request<::aws_smithy_http::body::SdkBody>) + ::std::marker::Send + ::std::marker::Sync + 'static,
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

            (self.customizable_send)(config_override).await
        }
    }
}
