use std::borrow::Cow;
use std::error::Error;

use anyhow::Result;
use wry::http::header::CONTENT_TYPE;
use wry::http::{
    Response,
    StatusCode,
};

pub fn res_404() -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(CONTENT_TYPE, "text/plain")
        .body(b"Not Found".as_ref().into())
        .unwrap()
}

pub fn res_400() -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(CONTENT_TYPE, "text/plain")
        .body(b"Bad Request".as_ref().into())
        .unwrap()
}

pub fn res_500(err: impl Error) -> Result<Response<Cow<'static, [u8]>>> {
    Ok(Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(CONTENT_TYPE, "text/plain")
        .body(err.to_string().into_bytes().into())?)
}
