// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_customization_permission(
    object_3: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::CustomizationPermission,
) -> Result<(), ::aws_smithy_http::operation::error::SerializationError> {
    match input {
        crate::types::CustomizationPermission::User(inner) => {
            object_3.key("user").string(inner.as_str());
        },
        crate::types::CustomizationPermission::Group(inner) => {
            object_3.key("group").string(inner.as_str());
        },
        crate::types::CustomizationPermission::Unknown => {
            return Err(
                ::aws_smithy_http::operation::error::SerializationError::unknown_variant("CustomizationPermission"),
            );
        },
    }
    Ok(())
}

pub(crate) fn de_customization_permission<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> Result<Option<crate::types::CustomizationPermission>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<
        Item = Result<
            ::aws_smithy_json::deserialize::Token<'a>,
            ::aws_smithy_json::deserialize::error::DeserializeError,
        >,
    >,
{
    let mut variant = None;
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => return Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => loop {
            match tokens.next().transpose()? {
                Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => {
                    if variant.is_some() {
                        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
                            "encountered mixed variants in union",
                        ));
                    }
                    variant = match key.to_unescaped()?.as_ref() {
                        "user" => Some(crate::types::CustomizationPermission::User(
                            ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                .transpose()?
                                .unwrap_or_default(),
                        )),
                        "group" => Some(crate::types::CustomizationPermission::Group(
                            ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                .transpose()?
                                .unwrap_or_default(),
                        )),
                        _ => {
                            ::aws_smithy_json::deserialize::token::skip_value(tokens)?;
                            Some(crate::types::CustomizationPermission::Unknown)
                        },
                    };
                },
                other => {
                    return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
                        format!("expected object key or end object, found: {:?}", other),
                    ));
                },
            }
        },
        _ => {
            return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
                "expected start object or null",
            ));
        },
    }
    Ok(variant)
}
