// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_upload_url_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_upload_url::CreateUploadUrlInput,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.content_md5 {
        object.key("contentMd5").string(var_1.as_str());
    }
    if let Some(var_2) = &input.content_checksum {
        object.key("contentChecksum").string(var_2.as_str());
    }
    if let Some(var_3) = &input.content_checksum_type {
        object.key("contentChecksumType").string(var_3.as_str());
    }
    if let Some(var_4) = &input.content_length {
        object.key("contentLength").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_4).into()),
        );
    }
    if let Some(var_5) = &input.artifact_type {
        object.key("artifactType").string(var_5.as_str());
    }
    if let Some(var_6) = &input.upload_intent {
        object.key("uploadIntent").string(var_6.as_str());
    }
    if let Some(var_7) = &input.upload_context {
        #[allow(unused_mut)]
        let mut object_8 = object.key("uploadContext").start_object();
        crate::protocol_serde::shape_upload_context::ser_upload_context(&mut object_8, var_7)?;
        object_8.finish();
    }
    Ok(())
}
