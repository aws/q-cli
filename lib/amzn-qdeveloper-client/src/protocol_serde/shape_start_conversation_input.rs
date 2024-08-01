// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_start_conversation_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::start_conversation::StartConversationInput,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.origin {
        object.key("origin").string(var_1.as_str());
    }
    if let Some(var_2) = &input.source {
        object.key("source").string(var_2.as_str());
    }
    if let Some(var_3) = &input.dry_run {
        object.key("dryRun").boolean(*var_3);
    }
    Ok(())
}
