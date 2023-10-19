pub mod cognito;
mod install_method;
mod util;

use amzn_toolkit_telemetry::error::SdkError;
use amzn_toolkit_telemetry::operation::post_metrics::PostMetricsError;
use amzn_toolkit_telemetry::types::{
    AwsProduct,
    MetricDatum,
};
use amzn_toolkit_telemetry::{
    Client,
    Config,
};
pub use aws_toolkit_telemetry_definitions::{
    metrics,
    types,
};
use cognito::{CognitoProvider, BETA_POOL};
use fig_util::system_info::os_version;
pub use install_method::{
    get_install_method,
    InstallMethod,
};
use once_cell::sync::Lazy;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("telemetry is disabled")]
    TelemetryDisabled,
    #[error(transparent)]
    ClientError(#[from] amzn_toolkit_telemetry::operation::post_metrics::PostMetricsError),
}

pub static CLIENT: Lazy<Client> = Lazy::new(|| Client::from_conf(Config::builder().build()));

// endpoints from <https://w.amazon.com/bin/view/AWS/DevEx/IDEToolkits/Telemetry/>
const BETA_ENDPOINT: &str = "https://7zftft3lj2.execute-api.us-east-1.amazonaws.com/Beta";
const INTERNAL_PROD_ENDPOINT: &str = "https://1ek5zo40ci.execute-api.us-east-1.amazonaws.com/InternalProd";
const EXTERNAL_PROD_ENDPOINT: &str = "https://client-telemetry.us-east-1.amazonaws.com";

#[derive(Debug, Clone)]
struct TelemetryClient {
    client_id: String,
    aws_client: Client,
}

impl TelemetryClient {
    pub async fn new() -> Self {
        let client_id = util::get_client_id().await;
        let aws_client = Client::from_conf(
            Config::builder()
                .endpoint_url(BETA_ENDPOINT)
                .credentials_provider(CognitoProvider::new(BETA_POOL))
                .build(),
        );
        Self { client_id, aws_client }
    }

    pub async fn post_metric(&self, inner: impl Into<MetricDatum>) -> Result<(), SdkError<PostMetricsError>> {
        let product = AwsProduct::Canary;
        let product_version = env!("CARGO_PKG_VERSION");
        let os = std::env::consts::OS;
        let os_architecture = std::env::consts::ARCH;
        let os_version = os_version().map(|v| v.to_string()).unwrap_or_default();

        self.aws_client
            .post_metrics()
            .aws_product(product)
            .aws_product_version(product_version)
            .client_id(&self.client_id)
            .os(os)
            .os_architecture(os_architecture)
            .os_version(os_version)
            .metric_data(inner.into())
            .send()
            .await?;

        Ok(())
    }
}

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

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_send() {
        let client = TelemetryClient::new().await;
        client
            .post_metric(metrics::UiClick {
                create_time: None,
                value: None,
                element_id: None,
            })
            .await
            .unwrap();
    }
}
