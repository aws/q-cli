// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_weaver_bird_planning_upload_context(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::WeaverBirdPlanningUploadContext,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.conversation_id {
        object.key("conversationId").string(var_1.as_str());
    }
    Ok(())
}
