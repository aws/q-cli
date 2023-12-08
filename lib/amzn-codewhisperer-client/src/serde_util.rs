// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn internal_server_exception_correct_errors(
    mut builder: crate::types::error::builders::InternalServerErrorBuilder,
) -> crate::types::error::builders::InternalServerErrorBuilder {
    if builder.message.is_none() {
        builder.message = Some(Default::default())
    }
    builder
}

pub(crate) fn throttling_exception_correct_errors(
    mut builder: crate::types::error::builders::ThrottlingErrorBuilder,
) -> crate::types::error::builders::ThrottlingErrorBuilder {
    if builder.message.is_none() {
        builder.message = Some(Default::default())
    }
    builder
}

pub(crate) fn validation_exception_correct_errors(
    mut builder: crate::types::error::builders::ValidationErrorBuilder,
) -> crate::types::error::builders::ValidationErrorBuilder {
    if builder.message.is_none() {
        builder.message = Some(Default::default())
    }
    builder
}

pub(crate) fn access_denied_exception_correct_errors(
    mut builder: crate::types::error::builders::AccessDeniedErrorBuilder,
) -> crate::types::error::builders::AccessDeniedErrorBuilder {
    if builder.message.is_none() {
        builder.message = Some(Default::default())
    }
    builder
}

pub(crate) fn create_artifact_upload_url_output_output_correct_errors(
    mut builder: crate::operation::create_artifact_upload_url::builders::CreateArtifactUploadUrlOutputBuilder,
) -> crate::operation::create_artifact_upload_url::builders::CreateArtifactUploadUrlOutputBuilder {
    if builder.upload_id.is_none() {
        builder.upload_id = Some(Default::default())
    }
    if builder.upload_url.is_none() {
        builder.upload_url = Some(Default::default())
    }
    builder
}

pub(crate) fn create_task_assist_conversation_output_output_correct_errors(
    mut builder: crate::operation::create_task_assist_conversation::builders::CreateTaskAssistConversationOutputBuilder,
) -> crate::operation::create_task_assist_conversation::builders::CreateTaskAssistConversationOutputBuilder {
    if builder.conversation_id.is_none() {
        builder.conversation_id = Some(Default::default())
    }
    builder
}

pub(crate) fn conflict_exception_correct_errors(
    mut builder: crate::types::error::builders::ConflictErrorBuilder,
) -> crate::types::error::builders::ConflictErrorBuilder {
    if builder.message.is_none() {
        builder.message = Some(Default::default())
    }
    builder
}

pub(crate) fn resource_not_found_exception_correct_errors(
    mut builder: crate::types::error::builders::ResourceNotFoundErrorBuilder,
) -> crate::types::error::builders::ResourceNotFoundErrorBuilder {
    if builder.message.is_none() {
        builder.message = Some(Default::default())
    }
    builder
}

pub(crate) fn create_upload_url_output_output_correct_errors(
    mut builder: crate::operation::create_upload_url::builders::CreateUploadUrlOutputBuilder,
) -> crate::operation::create_upload_url::builders::CreateUploadUrlOutputBuilder {
    if builder.upload_id.is_none() {
        builder.upload_id = Some(Default::default())
    }
    if builder.upload_url.is_none() {
        builder.upload_url = Some(Default::default())
    }
    builder
}

pub(crate) fn delete_task_assist_conversation_output_output_correct_errors(
    mut builder: crate::operation::delete_task_assist_conversation::builders::DeleteTaskAssistConversationOutputBuilder,
) -> crate::operation::delete_task_assist_conversation::builders::DeleteTaskAssistConversationOutputBuilder {
    if builder.conversation_id.is_none() {
        builder.conversation_id = Some(Default::default())
    }
    builder
}

pub(crate) fn get_code_analysis_output_output_correct_errors(
    mut builder: crate::operation::get_code_analysis::builders::GetCodeAnalysisOutputBuilder,
) -> crate::operation::get_code_analysis::builders::GetCodeAnalysisOutputBuilder {
    if builder.status.is_none() {
        builder.status = "no value was set".parse::<crate::types::CodeAnalysisStatus>().ok()
    }
    builder
}

pub(crate) fn get_task_assist_code_generation_output_output_correct_errors(
    mut builder: crate::operation::get_task_assist_code_generation::builders::GetTaskAssistCodeGenerationOutputBuilder,
) -> crate::operation::get_task_assist_code_generation::builders::GetTaskAssistCodeGenerationOutputBuilder {
    if builder.conversation_id.is_none() {
        builder.conversation_id = Some(Default::default())
    }
    if builder.code_generation_status.is_none() {
        builder.code_generation_status = {
            let builder = crate::types::builders::CodeGenerationStatusBuilder::default();
            crate::serde_util::code_generation_status_correct_errors(builder)
                .build()
                .ok()
        }
    }
    builder
}

pub(crate) fn get_transformation_output_output_correct_errors(
    mut builder: crate::operation::get_transformation::builders::GetTransformationOutputBuilder,
) -> crate::operation::get_transformation::builders::GetTransformationOutputBuilder {
    if builder.transformation_job.is_none() {
        builder.transformation_job = {
            let builder = crate::types::builders::TransformationJobBuilder::default();
            Some(builder.build())
        }
    }
    builder
}

pub(crate) fn get_transformation_plan_output_output_correct_errors(
    mut builder: crate::operation::get_transformation_plan::builders::GetTransformationPlanOutputBuilder,
) -> crate::operation::get_transformation_plan::builders::GetTransformationPlanOutputBuilder {
    if builder.transformation_plan.is_none() {
        builder.transformation_plan = {
            let builder = crate::types::builders::TransformationPlanBuilder::default();
            crate::serde_util::transformation_plan_correct_errors(builder)
                .build()
                .ok()
        }
    }
    builder
}

pub(crate) fn list_available_customizations_output_output_correct_errors(
    mut builder: crate::operation::list_available_customizations::builders::ListAvailableCustomizationsOutputBuilder,
) -> crate::operation::list_available_customizations::builders::ListAvailableCustomizationsOutputBuilder {
    if builder.customizations.is_none() {
        builder.customizations = Some(Default::default())
    }
    builder
}

pub(crate) fn list_code_analysis_findings_output_output_correct_errors(
    mut builder: crate::operation::list_code_analysis_findings::builders::ListCodeAnalysisFindingsOutputBuilder,
) -> crate::operation::list_code_analysis_findings::builders::ListCodeAnalysisFindingsOutputBuilder {
    if builder.code_analysis_findings.is_none() {
        builder.code_analysis_findings = Some(Default::default())
    }
    builder
}

pub(crate) fn list_feature_evaluations_output_output_correct_errors(
    mut builder: crate::operation::list_feature_evaluations::builders::ListFeatureEvaluationsOutputBuilder,
) -> crate::operation::list_feature_evaluations::builders::ListFeatureEvaluationsOutputBuilder {
    if builder.feature_evaluations.is_none() {
        builder.feature_evaluations = Some(Default::default())
    }
    builder
}

pub(crate) fn start_code_analysis_output_output_correct_errors(
    mut builder: crate::operation::start_code_analysis::builders::StartCodeAnalysisOutputBuilder,
) -> crate::operation::start_code_analysis::builders::StartCodeAnalysisOutputBuilder {
    if builder.job_id.is_none() {
        builder.job_id = Some(Default::default())
    }
    if builder.status.is_none() {
        builder.status = "no value was set".parse::<crate::types::CodeAnalysisStatus>().ok()
    }
    builder
}

pub(crate) fn start_task_assist_code_generation_output_output_correct_errors(
    mut builder: crate::operation::start_task_assist_code_generation::builders::StartTaskAssistCodeGenerationOutputBuilder,
) -> crate::operation::start_task_assist_code_generation::builders::StartTaskAssistCodeGenerationOutputBuilder {
    if builder.conversation_id.is_none() {
        builder.conversation_id = Some(Default::default())
    }
    if builder.code_generation_id.is_none() {
        builder.code_generation_id = Some(Default::default())
    }
    builder
}

pub(crate) fn start_transformation_output_output_correct_errors(
    mut builder: crate::operation::start_transformation::builders::StartTransformationOutputBuilder,
) -> crate::operation::start_transformation::builders::StartTransformationOutputBuilder {
    if builder.transformation_job_id.is_none() {
        builder.transformation_job_id = Some(Default::default())
    }
    builder
}

pub(crate) fn stop_transformation_output_output_correct_errors(
    mut builder: crate::operation::stop_transformation::builders::StopTransformationOutputBuilder,
) -> crate::operation::stop_transformation::builders::StopTransformationOutputBuilder {
    if builder.transformation_status.is_none() {
        builder.transformation_status = "no value was set".parse::<crate::types::TransformationStatus>().ok()
    }
    builder
}

pub(crate) fn code_generation_status_correct_errors(
    mut builder: crate::types::builders::CodeGenerationStatusBuilder,
) -> crate::types::builders::CodeGenerationStatusBuilder {
    if builder.status.is_none() {
        builder.status = "no value was set"
            .parse::<crate::types::CodeGenerationWorkflowStatus>()
            .ok()
    }
    if builder.current_stage.is_none() {
        builder.current_stage = "no value was set"
            .parse::<crate::types::CodeGenerationWorkflowStage>()
            .ok()
    }
    builder
}

pub(crate) fn transformation_plan_correct_errors(
    mut builder: crate::types::builders::TransformationPlanBuilder,
) -> crate::types::builders::TransformationPlanBuilder {
    if builder.transformation_steps.is_none() {
        builder.transformation_steps = Some(Default::default())
    }
    builder
}

pub(crate) fn completion_correct_errors(
    mut builder: crate::types::builders::CompletionBuilder,
) -> crate::types::builders::CompletionBuilder {
    if builder.content.is_none() {
        builder.content = Some(Default::default())
    }
    builder
}

pub(crate) fn customization_correct_errors(
    mut builder: crate::types::builders::CustomizationBuilder,
) -> crate::types::builders::CustomizationBuilder {
    if builder.arn.is_none() {
        builder.arn = Some(Default::default())
    }
    builder
}

pub(crate) fn feature_evaluation_correct_errors(
    mut builder: crate::types::builders::FeatureEvaluationBuilder,
) -> crate::types::builders::FeatureEvaluationBuilder {
    if builder.feature.is_none() {
        builder.feature = Some(Default::default())
    }
    if builder.variation.is_none() {
        builder.variation = Some(Default::default())
    }
    if builder.value.is_none() {
        builder.value = Some(crate::types::FeatureValue::Unknown)
    }
    builder
}

pub(crate) fn transformation_step_correct_errors(
    mut builder: crate::types::builders::TransformationStepBuilder,
) -> crate::types::builders::TransformationStepBuilder {
    if builder.id.is_none() {
        builder.id = Some(Default::default())
    }
    if builder.name.is_none() {
        builder.name = Some(Default::default())
    }
    if builder.description.is_none() {
        builder.description = Some(Default::default())
    }
    builder
}
