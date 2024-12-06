// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_upload_context(
    object_8: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::UploadContext,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    match input {
        crate::types::UploadContext::TaskAssistPlanningUploadContext(inner) => {
            #[allow(unused_mut)]
            let mut object_1 = object_8.key("taskAssistPlanningUploadContext").start_object();
            crate::protocol_serde::shape_task_assist_planning_upload_context::ser_task_assist_planning_upload_context(
                &mut object_1,
                inner,
            )?;
            object_1.finish();
        },
        crate::types::UploadContext::TransformationUploadContext(inner) => {
            #[allow(unused_mut)]
            let mut object_2 = object_8.key("transformationUploadContext").start_object();
            crate::protocol_serde::shape_transformation_upload_context::ser_transformation_upload_context(
                &mut object_2,
                inner,
            )?;
            object_2.finish();
        },
        crate::types::UploadContext::CodeAnalysisUploadContext(inner) => {
            #[allow(unused_mut)]
            let mut object_3 = object_8.key("codeAnalysisUploadContext").start_object();
            crate::protocol_serde::shape_code_analysis_upload_context::ser_code_analysis_upload_context(
                &mut object_3,
                inner,
            )?;
            object_3.finish();
        },
        crate::types::UploadContext::CodeFixUploadContext(inner) => {
            #[allow(unused_mut)]
            let mut object_4 = object_8.key("codeFixUploadContext").start_object();
            crate::protocol_serde::shape_code_fix_upload_context::ser_code_fix_upload_context(&mut object_4, inner)?;
            object_4.finish();
        },
        crate::types::UploadContext::Unknown => {
            return Err(::aws_smithy_types::error::operation::SerializationError::unknown_variant("UploadContext"));
        },
    }
    Ok(())
}
