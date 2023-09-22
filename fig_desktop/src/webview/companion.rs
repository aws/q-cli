use url::Url;

pub fn url() -> Url {
    Url::parse("http://localhost:3434").unwrap()
}
