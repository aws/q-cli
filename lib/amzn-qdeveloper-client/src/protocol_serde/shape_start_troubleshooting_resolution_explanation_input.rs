// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_start_troubleshooting_resolution_explanation_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::start_troubleshooting_resolution_explanation::StartTroubleshootingResolutionExplanationInput,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.session_id {
        object.key("sessionId").string(var_1.as_str());
    }
    Ok(())
}
