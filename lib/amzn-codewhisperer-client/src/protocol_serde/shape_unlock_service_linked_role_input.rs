// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_unlock_service_linked_role_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::unlock_service_linked_role::UnlockServiceLinkedRoleInput,
) -> Result<(), ::aws_smithy_http::operation::error::SerializationError> {
    if let Some(var_1) = &input.role_arn {
        object.key("RoleArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.deletion_status {
        object.key("DeletionStatus").string(var_2.as_str());
    }
    Ok(())
}
