// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_customization_version_summary<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> Result<Option<crate::types::CustomizationVersionSummary>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<
        Item = Result<
            ::aws_smithy_json::deserialize::Token<'a>,
            ::aws_smithy_json::deserialize::error::DeserializeError,
        >,
    >,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::CustomizationVersionSummaryBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => {
                        match key.to_unescaped()?.as_ref() {
                            "version" => {
                                builder = builder.set_version(
                                    ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                        .map(i64::try_from)
                                        .transpose()?,
                                );
                            },
                            "status" => {
                                builder = builder.set_status(
                                    ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                        .map(|s| {
                                            s.to_unescaped()
                                                .map(|u| crate::types::CustomizationStatus::from(u.as_ref()))
                                        })
                                        .transpose()?,
                                );
                            },
                            "dataReference" => {
                                builder = builder.set_data_reference(
                                    crate::protocol_serde::shape_data_reference::de_data_reference(tokens)?,
                                );
                            },
                            "updatedAt" => {
                                builder = builder.set_updated_at(
                                    ::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                        tokens.next(),
                                        ::aws_smithy_types::date_time::Format::EpochSeconds,
                                    )?,
                                );
                            },
                            "evaluationMetrics" => {
                                builder = builder.set_evaluation_metrics(
                                    crate::protocol_serde::shape_evaluation_metrics::de_evaluation_metrics(tokens)?,
                                );
                            },
                            _ => ::aws_smithy_json::deserialize::token::skip_value(tokens)?,
                        }
                    },
                    other => {
                        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
                            format!("expected object key or end object, found: {:?}", other),
                        ));
                    },
                }
            }
            Ok(Some(
                crate::serde_util::customization_version_summary_correct_errors(builder)
                    .build()
                    .map_err(|err| {
                        ::aws_smithy_json::deserialize::error::DeserializeError::custom_source(
                            "Response was invalid",
                            err,
                        )
                    })?,
            ))
        },
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}
