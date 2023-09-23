// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_associate_customization_permission_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::associate_customization_permission::AssociateCustomizationPermissionInput,
) -> Result<(), ::aws_smithy_http::operation::error::SerializationError> {
    if let Some(var_1) = &input.identifier {
        object.key("identifier").string(var_1.as_str());
    }
    if let Some(var_2) = &input.permission {
        #[allow(unused_mut)]
        let mut object_3 = object.key("permission").start_object();
        crate::protocol_serde::shape_customization_permission::ser_customization_permission(&mut object_3, var_2)?;
        object_3.finish();
    }
    Ok(())
}
