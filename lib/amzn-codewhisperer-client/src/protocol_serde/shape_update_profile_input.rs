// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_profile_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_profile::UpdateProfileInput,
) -> Result<(), ::aws_smithy_http::operation::error::SerializationError> {
    if let Some(var_1) = &input.profile_arn {
        object.key("profileArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.profile_name {
        object.key("profileName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.reference_tracker_configuration {
        #[allow(unused_mut)]
        let mut object_4 = object.key("referenceTrackerConfiguration").start_object();
        crate::protocol_serde::shape_reference_tracker_configuration::ser_reference_tracker_configuration(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.kms_key_arn {
        object.key("kmsKeyArn").string(var_5.as_str());
    }
    Ok(())
}
