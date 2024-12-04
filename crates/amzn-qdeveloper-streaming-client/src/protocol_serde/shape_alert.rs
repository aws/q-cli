// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_alert<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> Result<Option<crate::types::Alert>, ::aws_smithy_json::deserialize::error::DeserializeError>
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
            let mut builder = crate::types::builders::AlertBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => {
                        match key.to_unescaped()?.as_ref() {
                            "type" => {
                                builder = builder.set_type(
                                    ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                        .map(|s| s.to_unescaped().map(|u| crate::types::AlertType::from(u.as_ref())))
                                        .transpose()?,
                                );
                            },
                            "content" => {
                                builder = builder.set_content(
                                    crate::protocol_serde::shape_alert_component_list::de_alert_component_list(tokens)?,
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
            Ok(Some(crate::serde_util::alert_correct_errors(builder).build().map_err(
                |err| {
                    ::aws_smithy_json::deserialize::error::DeserializeError::custom_source("Response was invalid", err)
                },
            )?))
        },
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}
