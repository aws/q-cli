// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_generate_recommendations_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::generate_recommendations::GenerateRecommendationsInput,
) -> Result<(), ::aws_smithy_http::operation::error::SerializationError> {
    if let Some(var_1) = &input.file_context {
        #[allow(unused_mut)]
        let mut object_2 = object.key("fileContext").start_object();
        crate::protocol_serde::shape_file_context::ser_file_context(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.max_results {
        object.key("maxResults").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_3).into()),
        );
    }
    if let Some(var_4) = &input.next_token {
        object.key("nextToken").string(var_4.as_str());
    }
    if let Some(var_5) = &input.reference_tracker_configuration {
        #[allow(unused_mut)]
        let mut object_6 = object.key("referenceTrackerConfiguration").start_object();
        crate::protocol_serde::shape_reference_tracker_configuration::ser_reference_tracker_configuration(&mut object_6, var_5)?;
        object_6.finish();
    }
    if let Some(var_7) = &input.supplemental_contexts {
        let mut array_8 = object.key("supplementalContexts").start_array();
        for item_9 in var_7 {
            {
                #[allow(unused_mut)]
                let mut object_10 = array_8.value().start_object();
                crate::protocol_serde::shape_supplemental_context::ser_supplemental_context(&mut object_10, item_9)?;
                object_10.finish();
            }
        }
        array_8.finish();
    }
    Ok(())
}
