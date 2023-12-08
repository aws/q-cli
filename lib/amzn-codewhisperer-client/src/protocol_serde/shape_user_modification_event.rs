// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_user_modification_event(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::UserModificationEvent,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("sessionId").string(input.session_id.as_str());
    }
    {
        object.key("requestId").string(input.request_id.as_str());
    }
    {
        #[allow(unused_mut)]
        let mut object_1 = object.key("programmingLanguage").start_object();
        crate::protocol_serde::shape_programming_language::ser_programming_language(
            &mut object_1,
            &input.programming_language,
        )?;
        object_1.finish();
    }
    {
        object.key("modificationPercentage").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::Float((input.modification_percentage).into()),
        );
    }
    if let Some(var_2) = &input.customization_arn {
        object.key("customizationArn").string(var_2.as_str());
    }
    {
        object
            .key("timestamp")
            .date_time(&input.timestamp, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    Ok(())
}
