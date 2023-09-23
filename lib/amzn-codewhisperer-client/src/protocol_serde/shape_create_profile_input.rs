// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_profile_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_profile::CreateProfileInput,
) -> Result<(), ::aws_smithy_http::operation::error::SerializationError> {
    if let Some(var_1) = &input.identity_source {
        #[allow(unused_mut)]
        let mut object_2 = object.key("identitySource").start_object();
        crate::protocol_serde::shape_identity_source::ser_identity_source(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.profile_name {
        object.key("profileName").string(var_3.as_str());
    }
    if let Some(var_4) = &input.reference_tracker_configuration {
        #[allow(unused_mut)]
        let mut object_5 = object.key("referenceTrackerConfiguration").start_object();
        crate::protocol_serde::shape_reference_tracker_configuration::ser_reference_tracker_configuration(&mut object_5, var_4)?;
        object_5.finish();
    }
    if let Some(var_6) = &input.client_token {
        object.key("clientToken").string(var_6.as_str());
    }
    if let Some(var_7) = &input.kms_key_arn {
        object.key("kmsKeyArn").string(var_7.as_str());
    }
    if let Some(var_8) = &input.tags {
        let mut array_9 = object.key("tags").start_array();
        for item_10 in var_8 {
            {
                #[allow(unused_mut)]
                let mut object_11 = array_9.value().start_object();
                crate::protocol_serde::shape_tag::ser_tag(&mut object_11, item_10)?;
                object_11.finish();
            }
        }
        array_9.finish();
    }
    Ok(())
}
