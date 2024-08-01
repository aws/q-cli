// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_conversation_state(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ConversationState,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.conversation_id {
        object.key("conversationId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.history {
        let mut array_3 = object.key("history").start_array();
        for item_4 in var_2 {
            {
                #[allow(unused_mut)]
                let mut object_5 = array_3.value().start_object();
                crate::protocol_serde::shape_chat_message::ser_chat_message(&mut object_5, item_4)?;
                object_5.finish();
            }
        }
        array_3.finish();
    }
    {
        #[allow(unused_mut)]
        let mut object_6 = object.key("currentMessage").start_object();
        crate::protocol_serde::shape_chat_message::ser_chat_message(&mut object_6, &input.current_message)?;
        object_6.finish();
    }
    {
        object.key("chatTriggerType").string(input.chat_trigger_type.as_str());
    }
    if let Some(var_7) = &input.customization_arn {
        object.key("customizationArn").string(var_7.as_str());
    }
    Ok(())
}
