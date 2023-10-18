pub mod cognito;
mod install_method;
mod util;

use fig_util::system_info::os_version;
pub use install_method::{
    get_install_method,
    InstallMethod,
};
use once_cell::sync::Lazy;
use telemetry_client::error::SdkError;
use telemetry_client::operation::post_metrics::builders::PostMetricsFluentBuilder;
use telemetry_client::operation::post_metrics::PostMetricsError;
use telemetry_client::types::{
    AwsProduct,
    MetricDatum,
};
use telemetry_client::{
    Client,
    Config,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("telemetry is disabled")]
    TelemetryDisabled,
    #[error(transparent)]
    ClientError(#[from] telemetry_client::operation::post_metrics::PostMetricsError),
}

pub static CLIENT: Lazy<Client> = Lazy::new(|| Client::from_conf(Config::builder().build()));

struct TelemetryClient {
    client_id: String,
    aws_client: Client,
}

impl TelemetryClient {
    pub async fn new() -> Self {
        let client_id = util::get_client_id().await;
        let aws_client = Client::from_conf(Config::builder().build());
        Self { client_id, aws_client }
    }

    pub async fn post_metric(&self, inner: impl Into<MetricDatum>) -> Result<(), SdkError<PostMetricsError>> {
        self.aws_client
            .post_metrics()
            .aws_product(AwsProduct::Canary)
            .aws_product_version(env!("CARGO_PKG_VERSION"))
            .client_id(&self.client_id)
            .os(std::env::consts::OS)
            .os_architecture(std::env::consts::ARCH)
            .os_version(os_version().map(|v| v.to_string()).unwrap_or_default())
            .metric_data(inner.into())
            .send()
            .await?;

        Ok(())
    }
}

struct TelemetryPublisher {}

// structure PostMetricsRequest {
//     @required
//     awsProduct: AWSProduct
//     @required
//     awsProductVersion: AWSProductVersion
//     @required
//     clientId: ClientID
//     os: Value
//     osArchitecture: Value
//     osVersion: Value
//     parentProduct: Value
//     parentProductVersion: Value
//     @required
//     metricData: MetricData
// }

// pub async fn send_user_logged_in() -> Result<(), Error> {
//     base_request()
//         .set_metric_data(Some(vec![MetricDatum::builder().set_metric_name().build()]))
//         .send()
//         .await
//         .map_err(|err| err.into_service_error())?;

//     Ok(())
// }

// fn base_request() -> PostMetricsFluentBuilder {
//     CLIENT
//         .post_metrics()
//         .aws_product(AwsProduct::Terminal)
//         .aws_product_version(env!("CARGO_PKG_VERSION"))
//         .client_id(input)
//         .os(std::env::consts::OS)
//         .os_architecture(std::env::consts::ARCH)
//         .os_version(os_version().map(|v| v.to_string()).unwrap_or_default())
//         .metric_data(MetricDatum::builder().metric_name(input).value(input))
// }
