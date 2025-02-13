// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_tool_result(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ToolResult,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("toolUseId").string(input.tool_use_id.as_str());
    }
    {
        let mut array_1 = object.key("content").start_array();
        for item_2 in &input.content {
            {
                #[allow(unused_mut)]
                let mut object_3 = array_1.value().start_object();
                crate::protocol_serde::shape_tool_result_content_block::ser_tool_result_content_block(
                    &mut object_3,
                    item_2,
                )?;
                object_3.finish();
            }
        }
        array_1.finish();
    }
    if let Some(var_4) = &input.status {
        object.key("status").string(var_4.as_str());
    }
    Ok(())
}
