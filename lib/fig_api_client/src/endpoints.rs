use std::borrow::Cow;

use aws_config::Region;

const PROD_URL: &str = "https://codewhisperer.us-east-1.amazonaws.com";
const PROD_REGION: Region = Region::from_static("us-east-1");

// const ALPHA_URL: &str = "https://rts.alpha-us-west-2.codewhisperer.ai.aws.dev";
// const ALPHA_REGION: Region = Region::from_static("us-west-2");

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Endpoint {
    /// Prod endpoint for RTS
    Prod,
    Custom {
        url: Cow<'static, str>,
        region: Cow<'static, str>,
    },
}

impl Endpoint {
    pub(crate) fn url(&self) -> &str {
        match self {
            Endpoint::Prod => PROD_URL,
            Endpoint::Custom { url, .. } => url,
        }
    }

    pub(crate) fn region(&self) -> Region {
        match self {
            Endpoint::Prod => PROD_REGION,
            Endpoint::Custom { region, .. } => Region::new(region.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[test]
    fn test_endpoints() {
        let prod = Endpoint::Prod;
        Url::parse(prod.url()).unwrap();
        assert_eq!(prod.region(), PROD_REGION);

        let custom = Endpoint::Custom {
            region: "us-west-2".into(),
            url: "https://example.com".into(),
        };
        Url::parse(custom.url()).unwrap();
        assert_eq!(custom.region(), Region::new("us-west-2"));
    }
}
