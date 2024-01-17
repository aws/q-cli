use bytes::Bytes;
use http::Response;

/// Converts a [reqwest::Response] to a [http::Response<Bytes>]
pub async fn reqwest_response_to_http_response(res: reqwest::Response) -> Result<Response<Bytes>, crate::Error> {
    let mut builder = Response::builder();

    builder = builder.status(res.status());
    builder = builder.version(res.version());

    for (key, value) in res.headers() {
        builder = builder.header(key, value);
    }

    Ok(builder.body(res.bytes().await?)?)
}
