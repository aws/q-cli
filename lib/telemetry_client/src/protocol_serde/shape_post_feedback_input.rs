// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_post_feedback_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::post_feedback::PostFeedbackInput,
) -> Result<(), ::aws_smithy_http::operation::error::SerializationError> {
    if let Some(var_1) = &input.aws_product {
        object.key("awsProduct").string(var_1.as_str());
    }
    if let Some(var_2) = &input.aws_product_version {
        object.key("awsProductVersion").string(var_2.as_str());
    }
    if let Some(var_3) = &input.comment {
        object.key("comment").string(var_3.as_str());
    }
    if let Some(var_4) = &input.metadata {
        let mut array_5 = object.key("metadata").start_array();
        for item_6 in var_4 {
            {
                #[allow(unused_mut)]
                let mut object_7 = array_5.value().start_object();
                crate::protocol_serde::shape_metadata_entry::ser_metadata_entry(&mut object_7, item_6)?;
                object_7.finish();
            }
        }
        array_5.finish();
    }
    if let Some(var_8) = &input.os {
        object.key("os").string(var_8.as_str());
    }
    if let Some(var_9) = &input.os_version {
        object.key("osVersion").string(var_9.as_str());
    }
    if let Some(var_10) = &input.parent_product {
        object.key("parentProduct").string(var_10.as_str());
    }
    if let Some(var_11) = &input.parent_product_version {
        object.key("parentProductVersion").string(var_11.as_str());
    }
    if let Some(var_12) = &input.sentiment {
        object.key("sentiment").string(var_12.as_str());
    }
    Ok(())
}
