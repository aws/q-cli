use amzn_toolkit_telemetry::config::endpoint::{
    Endpoint,
    EndpointFuture,
    Params,
    ResolveEndpoint,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct StaticEndpoint(pub &'static str);

impl ResolveEndpoint for StaticEndpoint {
    fn resolve_endpoint<'a>(&'a self, _params: &'a Params) -> EndpointFuture<'a> {
        let endpoint = Endpoint::builder().url(self.0).build();
        tracing::info!(?endpoint, "Resolving endpoint");
        EndpointFuture::ready(Ok(endpoint))
    }
}
