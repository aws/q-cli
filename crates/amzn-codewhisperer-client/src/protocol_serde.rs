// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn type_erase_result<O, E>(
    result: ::std::result::Result<O, E>,
) -> ::std::result::Result<
    ::aws_smithy_runtime_api::client::interceptors::context::Output,
    ::aws_smithy_runtime_api::client::orchestrator::OrchestratorError<
        ::aws_smithy_runtime_api::client::interceptors::context::Error,
    >,
>
where
    O: ::std::fmt::Debug + ::std::marker::Send + ::std::marker::Sync + 'static,
    E: ::std::error::Error + std::fmt::Debug + ::std::marker::Send + ::std::marker::Sync + 'static,
{
    result
        .map(|output| ::aws_smithy_runtime_api::client::interceptors::context::Output::erase(output))
        .map_err(|error| ::aws_smithy_runtime_api::client::interceptors::context::Error::erase(error))
        .map_err(::std::convert::Into::into)
}

pub fn parse_http_error_metadata(
    _response_status: u16,
    response_headers: &::aws_smithy_runtime_api::http::Headers,
    response_body: &[u8],
) -> Result<::aws_smithy_types::error::metadata::Builder, ::aws_smithy_json::deserialize::error::DeserializeError> {
    crate::json_errors::parse_error_metadata(response_body, response_headers)
}

pub(crate) mod shape_create_artifact_upload_url;

pub(crate) mod shape_create_task_assist_conversation;

pub(crate) mod shape_create_upload_url;

pub(crate) mod shape_delete_task_assist_conversation;

pub(crate) mod shape_generate_completions;

pub(crate) mod shape_get_code_analysis;

pub(crate) mod shape_get_code_fix_job;

pub(crate) mod shape_get_task_assist_code_generation;

pub(crate) mod shape_get_test_generation;

pub(crate) mod shape_get_transformation;

pub(crate) mod shape_get_transformation_plan;

pub(crate) mod shape_list_available_customizations;

pub(crate) mod shape_list_code_analysis_findings;

pub(crate) mod shape_list_feature_evaluations;

pub(crate) mod shape_resume_transformation;

pub(crate) mod shape_send_telemetry_event;

pub(crate) mod shape_start_code_analysis;

pub(crate) mod shape_start_code_fix_job;

pub(crate) mod shape_start_task_assist_code_generation;

pub(crate) mod shape_start_test_generation;

pub(crate) mod shape_start_transformation;

pub(crate) mod shape_stop_transformation;

pub(crate) fn or_empty_doc(data: &[u8]) -> &[u8] {
    if data.is_empty() { b"{}" } else { data }
}

pub(crate) mod shape_access_denied_exception;

pub(crate) mod shape_conflict_exception;

pub(crate) mod shape_create_artifact_upload_url_input;

pub(crate) mod shape_create_upload_url_input;

pub(crate) mod shape_delete_task_assist_conversation_input;

pub(crate) mod shape_generate_completions_input;

pub(crate) mod shape_get_code_analysis_input;

pub(crate) mod shape_get_code_fix_job_input;

pub(crate) mod shape_get_task_assist_code_generation_input;

pub(crate) mod shape_get_test_generation_input;

pub(crate) mod shape_get_transformation_input;

pub(crate) mod shape_get_transformation_plan_input;

pub(crate) mod shape_internal_server_exception;

pub(crate) mod shape_list_available_customizations_input;

pub(crate) mod shape_list_code_analysis_findings_input;

pub(crate) mod shape_list_feature_evaluations_input;

pub(crate) mod shape_resource_not_found_exception;

pub(crate) mod shape_resume_transformation_input;

pub(crate) mod shape_send_telemetry_event_input;

pub(crate) mod shape_service_quota_exceeded_exception;

pub(crate) mod shape_start_code_analysis_input;

pub(crate) mod shape_start_code_fix_job_input;

pub(crate) mod shape_start_task_assist_code_generation_input;

pub(crate) mod shape_start_test_generation_input;

pub(crate) mod shape_start_transformation_input;

pub(crate) mod shape_stop_transformation_input;

pub(crate) mod shape_throttling_exception;

pub(crate) mod shape_validation_exception;

pub(crate) mod shape_code_generation_status;

pub(crate) mod shape_completions;

pub(crate) mod shape_conversation_state;

pub(crate) mod shape_customizations;

pub(crate) mod shape_feature_evaluations_list;

pub(crate) mod shape_file_context;

pub(crate) mod shape_intent_context;

pub(crate) mod shape_programming_language;

pub(crate) mod shape_range;

pub(crate) mod shape_reference_tracker_configuration;

pub(crate) mod shape_request_headers;

pub(crate) mod shape_suggested_fix;

pub(crate) mod shape_supplemental_context;

pub(crate) mod shape_target_code;

pub(crate) mod shape_task_assist_plan_step;

pub(crate) mod shape_telemetry_event;

pub(crate) mod shape_test_generation_job;

pub(crate) mod shape_transformation_job;

pub(crate) mod shape_transformation_plan;

pub(crate) mod shape_transformation_spec;

pub(crate) mod shape_upload_context;

pub(crate) mod shape_user_context;

pub(crate) mod shape_workspace_state;

pub(crate) mod shape_chat_add_message_event;

pub(crate) mod shape_chat_interact_with_message_event;

pub(crate) mod shape_chat_message;

pub(crate) mod shape_chat_user_modification_event;

pub(crate) mod shape_code_analysis_upload_context;

pub(crate) mod shape_code_coverage_event;

pub(crate) mod shape_code_fix_acceptance_event;

pub(crate) mod shape_code_fix_generation_event;

pub(crate) mod shape_code_fix_upload_context;

pub(crate) mod shape_code_scan_event;

pub(crate) mod shape_code_scan_failed_event;

pub(crate) mod shape_code_scan_remediations_event;

pub(crate) mod shape_code_scan_succeeded_event;

pub(crate) mod shape_completion;

pub(crate) mod shape_customization;

pub(crate) mod shape_doc_generation_event;

pub(crate) mod shape_doc_v2_acceptance_event;

pub(crate) mod shape_doc_v2_generation_event;

pub(crate) mod shape_documentation_intent_context;

pub(crate) mod shape_feature_dev_code_acceptance_event;

pub(crate) mod shape_feature_dev_code_generation_event;

pub(crate) mod shape_feature_dev_event;

pub(crate) mod shape_feature_evaluation;

pub(crate) mod shape_inline_chat_event;

pub(crate) mod shape_metric_data;

pub(crate) mod shape_package_info_list;

pub(crate) mod shape_position;

pub(crate) mod shape_references;

pub(crate) mod shape_task_assist_planning_upload_context;

pub(crate) mod shape_terminal_user_interaction_event;

pub(crate) mod shape_test_generation_event;

pub(crate) mod shape_transform_event;

pub(crate) mod shape_transformation_project_state;

pub(crate) mod shape_transformation_steps;

pub(crate) mod shape_transformation_upload_context;

pub(crate) mod shape_user_modification_event;

pub(crate) mod shape_user_trigger_decision_event;

pub(crate) mod shape_assistant_response_message;

pub(crate) mod shape_dimension;

pub(crate) mod shape_feature_value;

pub(crate) mod shape_imports;

pub(crate) mod shape_package_info;

pub(crate) mod shape_reference;

pub(crate) mod shape_transformation_platform_config;

pub(crate) mod shape_transformation_project_artifact_descriptor;

pub(crate) mod shape_transformation_runtime_env;

pub(crate) mod shape_transformation_step;

pub(crate) mod shape_user_input_message;

pub(crate) mod shape_followup_prompt;

pub(crate) mod shape_import;

pub(crate) mod shape_progress_updates;

pub(crate) mod shape_span;

pub(crate) mod shape_supplementary_web_link;

pub(crate) mod shape_target_file_info_list;

pub(crate) mod shape_transformation_source_code_artifact_descriptor;

pub(crate) mod shape_user_input_message_context;

pub(crate) mod shape_additional_content_entry;

pub(crate) mod shape_app_studio_state;

pub(crate) mod shape_console_state;

pub(crate) mod shape_diagnostic;

pub(crate) mod shape_editor_state;

pub(crate) mod shape_env_state;

pub(crate) mod shape_git_state;

pub(crate) mod shape_shell_state;

pub(crate) mod shape_target_file_info;

pub(crate) mod shape_tool;

pub(crate) mod shape_tool_result;

pub(crate) mod shape_transformation_progress_update;

pub(crate) mod shape_user_settings;

pub(crate) mod shape_cursor_state;

pub(crate) mod shape_environment_variable;

pub(crate) mod shape_relevant_text_document;

pub(crate) mod shape_runtime_diagnostic;

pub(crate) mod shape_shell_history_entry;

pub(crate) mod shape_text_document;

pub(crate) mod shape_text_document_diagnostic;

pub(crate) mod shape_tool_result_content_block;

pub(crate) mod shape_tool_specification;

pub(crate) mod shape_transformation_download_artifacts;

pub(crate) mod shape_transformation_languages;

pub(crate) mod shape_document_symbol;

pub(crate) mod shape_tool_input_schema;

pub(crate) mod shape_transformation_download_artifact;
