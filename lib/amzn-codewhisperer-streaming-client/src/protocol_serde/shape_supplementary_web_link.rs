// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_supplementary_web_link(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::SupplementaryWebLink,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("url").string(input.url.as_str());
    }
    {
        object.key("title").string(input.title.as_str());
    }
    if let Some(var_1) = &input.snippet {
        object.key("snippet").string(var_1.as_str());
    }
    Ok(())
}

pub(crate) fn de_supplementary_web_link<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> Result<Option<crate::types::SupplementaryWebLink>, ::aws_smithy_json::deserialize::error::DeserializeError>
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
            let mut builder = crate::types::builders::SupplementaryWebLinkBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => {
                        match key.to_unescaped()?.as_ref() {
                            "url" => {
                                builder = builder.set_url(
                                    ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                        .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                        .transpose()?,
                                );
                            },
                            "title" => {
                                builder = builder.set_title(
                                    ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                        .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                        .transpose()?,
                                );
                            },
                            "snippet" => {
                                builder = builder.set_snippet(
                                    ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                        .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                        .transpose()?,
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
                crate::serde_util::supplementary_web_link_correct_errors(builder)
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
