use aws_config::Region;

const PROD_URL: &str = "https://rts.alpha-us-west-2.codewhisperer.ai.aws.dev";
const PROD_REGION: Region = Region::from_static("us-west-2");

const ALPHA_URL: &str = "https://codewhisperer.us-east-1.amazonaws.com";
const ALPHA_REGION: Region = Region::from_static("us-east-1");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Endpoint {
    /// Prod endpoint for RTS
    Prod,
    /// Alpha endpoint for RTS
    Alpha,
}

impl Endpoint {
    pub(crate) fn url(&self) -> &'static str {
        match self {
            Endpoint::Prod => PROD_URL,
            Endpoint::Alpha => ALPHA_URL,
        }
    }

    pub(crate) fn region(&self) -> Region {
        match self {
            Endpoint::Prod => PROD_REGION,
            Endpoint::Alpha => ALPHA_REGION,
        }
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[test]
    fn test_endpoints() {
        Url::parse(Endpoint::Prod.url()).unwrap();
        Url::parse(Endpoint::Alpha.url()).unwrap();
    }
}
