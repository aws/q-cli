// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_get_task_assist_code_generation_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::get_task_assist_code_generation::GetTaskAssistCodeGenerationInput,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.conversation_id {
        object.key("conversationId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.code_generation_id {
        object.key("codeGenerationId").string(var_2.as_str());
    }
    Ok(())
}
