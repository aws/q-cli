// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_generate_code_from_commands_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::generate_code_from_commands::GenerateCodeFromCommandsInput,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.output_format {
        object.key("outputFormat").string(var_1.as_str());
    }
    if let Some(var_2) = &input.commands {
        #[allow(unused_mut)]
        let mut object_3 = object.key("commands").start_object();
        crate::protocol_serde::shape_command_input::ser_command_input(&mut object_3, var_2)?;
        object_3.finish();
    }
    Ok(())
}
