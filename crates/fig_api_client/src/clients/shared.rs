use aws_config::Region;
use aws_credential_types::Credentials;
use aws_credential_types::provider::ProvideCredentials;
use aws_types::SdkConfig;
use fig_aws_common::behavior_version;

use crate::credentials::CredentialsChain;
use crate::{
    Endpoint,
    Error,
};

async fn base_sdk_config(region: Region, credentials_provider: impl ProvideCredentials + 'static) -> SdkConfig {
    aws_config::defaults(behavior_version())
        .region(region)
        .credentials_provider(credentials_provider)
        .load()
        .await
}

pub(crate) async fn bearer_sdk_config(endpoint: &Endpoint) -> SdkConfig {
    let credentials = Credentials::new("xxx", "xxx", None, None, "xxx");
    base_sdk_config(endpoint.region().clone(), credentials).await
}

pub(crate) async fn sigv4_sdk_config(endpoint: &Endpoint) -> Result<SdkConfig, Error> {
    let credentials_chain = CredentialsChain::new().await;

    if let Err(err) = credentials_chain.provide_credentials().await {
        return Err(Error::Credentials(err));
    };

    Ok(base_sdk_config(endpoint.region().clone(), credentials_chain).await)
}
