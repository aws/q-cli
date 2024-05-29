// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_create_upload_url_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::create_upload_url::CreateUploadUrlOutput,
    crate::operation::create_upload_url::CreateUploadUrlError,
> {
    #[allow(unused_mut)]
    let mut generic_builder =
        crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
            .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => {
            return Err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled(
                generic,
            ));
        },
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "InternalServerException" => crate::operation::create_upload_url::CreateUploadUrlError::InternalServerError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerErrorBuilder::default();
                output = crate::protocol_serde::shape_internal_server_exception::de_internal_server_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::internal_server_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?
            };
            tmp
        }),
        "ServiceQuotaExceededException" => {
            crate::operation::create_upload_url::CreateUploadUrlError::ServiceQuotaExceededError({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::ServiceQuotaExceededErrorBuilder::default();
                    output = crate::protocol_serde::shape_service_quota_exceeded_exception::de_service_quota_exceeded_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?;
                    let output = output.meta(generic);
                    crate::serde_util::service_quota_exceeded_exception_correct_errors(output)
                        .build()
                        .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?
                };
                tmp
            })
        },
        "ThrottlingException" => crate::operation::create_upload_url::CreateUploadUrlError::ThrottlingError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ThrottlingErrorBuilder::default();
                output = crate::protocol_serde::shape_throttling_exception::de_throttling_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::throttling_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?
            };
            tmp
        }),
        "ValidationException" => crate::operation::create_upload_url::CreateUploadUrlError::ValidationError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ValidationErrorBuilder::default();
                output = crate::protocol_serde::shape_validation_exception::de_validation_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::validation_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?
            };
            tmp
        }),
        "ConflictException" => crate::operation::create_upload_url::CreateUploadUrlError::ConflictError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ConflictErrorBuilder::default();
                output = crate::protocol_serde::shape_conflict_exception::de_conflict_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::conflict_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?
            };
            tmp
        }),
        "ResourceNotFoundException" => {
            crate::operation::create_upload_url::CreateUploadUrlError::ResourceNotFoundError({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::ResourceNotFoundErrorBuilder::default();
                    output = crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?;
                    let output = output.meta(generic);
                    crate::serde_util::resource_not_found_exception_correct_errors(output)
                        .build()
                        .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?
                };
                tmp
            })
        },
        "AccessDeniedException" => crate::operation::create_upload_url::CreateUploadUrlError::AccessDeniedError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AccessDeniedErrorBuilder::default();
                output = crate::protocol_serde::shape_access_denied_exception::de_access_denied_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::access_denied_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?
            };
            tmp
        }),
        _ => crate::operation::create_upload_url::CreateUploadUrlError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_create_upload_url_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::create_upload_url::CreateUploadUrlOutput,
    crate::operation::create_upload_url::CreateUploadUrlError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::create_upload_url::builders::CreateUploadUrlOutputBuilder::default();
        output = crate::protocol_serde::shape_create_upload_url::de_create_upload_url(_response_body, output)
            .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        crate::serde_util::create_upload_url_output_output_correct_errors(output)
            .build()
            .map_err(crate::operation::create_upload_url::CreateUploadUrlError::unhandled)?
    })
}

pub fn ser_create_upload_url_input(
    input: &crate::operation::create_upload_url::CreateUploadUrlInput,
) -> Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_create_upload_url_input::ser_create_upload_url_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_create_upload_url(
    value: &[u8],
    mut builder: crate::operation::create_upload_url::builders::CreateUploadUrlOutputBuilder,
) -> Result<
    crate::operation::create_upload_url::builders::CreateUploadUrlOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned =
        ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "uploadId" => {
                    builder = builder.set_upload_id(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                },
                "uploadUrl" => {
                    builder = builder.set_upload_url(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                },
                "kmsKeyArn" => {
                    builder = builder.set_kms_key_arn(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                },
                "requestHeaders" => {
                    builder = builder.set_request_headers(
                        crate::protocol_serde::shape_request_headers::de_request_headers(tokens)?,
                    );
                },
                _ => ::aws_smithy_json::deserialize::token::skip_value(tokens)?,
            },
            other => {
                return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
                    format!("expected object key or end object, found: {:?}", other),
                ));
            },
        }
    }
    if tokens.next().is_some() {
        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "found more JSON tokens after completing parsing",
        ));
    }
    Ok(builder)
}
