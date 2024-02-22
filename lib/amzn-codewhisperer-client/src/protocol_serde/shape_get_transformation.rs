// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_get_transformation_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::get_transformation::GetTransformationOutput,
    crate::operation::get_transformation::GetTransformationError,
> {
    #[allow(unused_mut)]
    let mut generic_builder =
        crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
            .map_err(crate::operation::get_transformation::GetTransformationError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => {
            return Err(crate::operation::get_transformation::GetTransformationError::unhandled(
                generic,
            ));
        },
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "InternalServerException" => {
            crate::operation::get_transformation::GetTransformationError::InternalServerError({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InternalServerErrorBuilder::default();
                    output =
                        crate::protocol_serde::shape_internal_server_exception::de_internal_server_exception_json_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::get_transformation::GetTransformationError::unhandled)?;
                    let output = output.meta(generic);
                    crate::serde_util::internal_server_exception_correct_errors(output)
                        .build()
                        .map_err(crate::operation::get_transformation::GetTransformationError::unhandled)?
                };
                tmp
            })
        },
        "ThrottlingException" => crate::operation::get_transformation::GetTransformationError::ThrottlingError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ThrottlingErrorBuilder::default();
                output = crate::protocol_serde::shape_throttling_exception::de_throttling_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::get_transformation::GetTransformationError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::throttling_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::get_transformation::GetTransformationError::unhandled)?
            };
            tmp
        }),
        "ValidationException" => crate::operation::get_transformation::GetTransformationError::ValidationError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ValidationErrorBuilder::default();
                output = crate::protocol_serde::shape_validation_exception::de_validation_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::get_transformation::GetTransformationError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::validation_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::get_transformation::GetTransformationError::unhandled)?
            };
            tmp
        }),
        "ResourceNotFoundException" => {
            crate::operation::get_transformation::GetTransformationError::ResourceNotFoundError({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::ResourceNotFoundErrorBuilder::default();
                    output = crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::get_transformation::GetTransformationError::unhandled)?;
                    let output = output.meta(generic);
                    crate::serde_util::resource_not_found_exception_correct_errors(output)
                        .build()
                        .map_err(crate::operation::get_transformation::GetTransformationError::unhandled)?
                };
                tmp
            })
        },
        "AccessDeniedException" => crate::operation::get_transformation::GetTransformationError::AccessDeniedError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AccessDeniedErrorBuilder::default();
                output = crate::protocol_serde::shape_access_denied_exception::de_access_denied_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::get_transformation::GetTransformationError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::access_denied_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::get_transformation::GetTransformationError::unhandled)?
            };
            tmp
        }),
        _ => crate::operation::get_transformation::GetTransformationError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_get_transformation_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::get_transformation::GetTransformationOutput,
    crate::operation::get_transformation::GetTransformationError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::get_transformation::builders::GetTransformationOutputBuilder::default();
        output = crate::protocol_serde::shape_get_transformation::de_get_transformation(_response_body, output)
            .map_err(crate::operation::get_transformation::GetTransformationError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        crate::serde_util::get_transformation_output_output_correct_errors(output)
            .build()
            .map_err(crate::operation::get_transformation::GetTransformationError::unhandled)?
    })
}

pub fn ser_get_transformation_input(
    input: &crate::operation::get_transformation::GetTransformationInput,
) -> Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_get_transformation_input::ser_get_transformation_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_get_transformation(
    value: &[u8],
    mut builder: crate::operation::get_transformation::builders::GetTransformationOutputBuilder,
) -> Result<
    crate::operation::get_transformation::builders::GetTransformationOutputBuilder,
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
                "transformationJob" => {
                    builder = builder.set_transformation_job(
                        crate::protocol_serde::shape_transformation_job::de_transformation_job(tokens)?,
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
