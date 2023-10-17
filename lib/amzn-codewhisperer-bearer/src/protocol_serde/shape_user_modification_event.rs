// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_user_modification_event(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::UserModificationEvent,
) -> Result<(), ::aws_smithy_http::operation::error::SerializationError> {
    if let Some(var_1) = &input.session_id {
        object.key("sessionId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.request_id {
        object.key("requestId").string(var_2.as_str());
    }
    if let Some(var_3) = &input.programming_language {
        #[allow(unused_mut)]
        let mut object_4 = object.key("programmingLanguage").start_object();
        crate::protocol_serde::shape_programming_language::ser_programming_language(&mut object_4, var_3)?;
        object_4.finish();
    }
    {
        object.key("modificationPercentage").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::Float((input.modification_percentage).into()),
        );
    }
    if let Some(var_5) = &input.customization_arn {
        object.key("customizationArn").string(var_5.as_str());
    }
    if let Some(var_6) = &input.timestamp {
        object
            .key("timestamp")
            .date_time(var_6, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    Ok(())
}
