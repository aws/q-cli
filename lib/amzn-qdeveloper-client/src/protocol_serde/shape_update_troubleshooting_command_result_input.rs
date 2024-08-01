// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_troubleshooting_command_result_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_troubleshooting_command_result::UpdateTroubleshootingCommandResultInput,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.session_id {
        object.key("sessionId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.command_id {
        object.key("commandId").string(var_2.as_str());
    }
    if let Some(var_3) = &input.status {
        object.key("status").string(var_3.as_str());
    }
    if let Some(var_4) = &input.result {
        object.key("result").string(var_4.as_str());
    }
    Ok(())
}
