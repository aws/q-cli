// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn resource_not_found_exception_correct_errors(
    mut builder: crate::types::error::builders::ResourceNotFoundErrorBuilder,
) -> crate::types::error::builders::ResourceNotFoundErrorBuilder {
    if builder.message.is_none() {
        builder.message = Some(Default::default())
    }
    builder
}

pub(crate) fn internal_server_exception_correct_errors(
    mut builder: crate::types::error::builders::InternalServerErrorBuilder,
) -> crate::types::error::builders::InternalServerErrorBuilder {
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

pub(crate) fn conflict_exception_correct_errors(
    mut builder: crate::types::error::builders::ConflictErrorBuilder,
) -> crate::types::error::builders::ConflictErrorBuilder {
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
    if builder.reason.is_none() {
        builder.reason = "no value was set"
            .parse::<crate::types::ValidationExceptionReason>()
            .ok()
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

pub(crate) fn associate_connector_resource_output_output_correct_errors(
    mut builder: crate::operation::associate_connector_resource::builders::AssociateConnectorResourceOutputBuilder,
) -> crate::operation::associate_connector_resource::builders::AssociateConnectorResourceOutputBuilder {
    if builder.connector_id.is_none() {
        builder.connector_id = Some(Default::default())
    }
    if builder.connector_name.is_none() {
        builder.connector_name = Some(Default::default())
    }
    if builder.connector_type.is_none() {
        builder.connector_type = Some(Default::default())
    }
    if builder.account_connection.is_none() {
        builder.account_connection = Some(crate::types::AccountConnection::Unknown)
    }
    builder
}

pub(crate) fn create_extension_output_output_correct_errors(
    mut builder: crate::operation::create_extension::builders::CreateExtensionOutputBuilder,
) -> crate::operation::create_extension::builders::CreateExtensionOutputBuilder {
    if builder.extension_id.is_none() {
        builder.extension_id = Some(Default::default())
    }
    builder
}

pub(crate) fn create_plugin_output_output_correct_errors(
    mut builder: crate::operation::create_plugin::builders::CreatePluginOutputBuilder,
) -> crate::operation::create_plugin::builders::CreatePluginOutputBuilder {
    if builder.plugin_id.is_none() {
        builder.plugin_id = Some(Default::default())
    }
    builder
}

pub(crate) fn get_connector_output_output_correct_errors(
    mut builder: crate::operation::get_connector::builders::GetConnectorOutputBuilder,
) -> crate::operation::get_connector::builders::GetConnectorOutputBuilder {
    if builder.connector_id.is_none() {
        builder.connector_id = Some(Default::default())
    }
    if builder.workspace_id.is_none() {
        builder.workspace_id = Some(Default::default())
    }
    if builder.workspace_name.is_none() {
        builder.workspace_name = Some(Default::default())
    }
    if builder.connector_name.is_none() {
        builder.connector_name = Some(Default::default())
    }
    if builder.user_id.is_none() {
        builder.user_id = Some(Default::default())
    }
    if builder.source_account.is_none() {
        builder.source_account = Some(Default::default())
    }
    if builder.description.is_none() {
        builder.description = Some(Default::default())
    }
    if builder.connector_type.is_none() {
        builder.connector_type = Some(Default::default())
    }
    if builder.account_connection.is_none() {
        builder.account_connection = Some(crate::types::AccountConnection::Unknown)
    }
    if builder.connector_configuration.is_none() {
        builder.connector_configuration = Some(Default::default())
    }
    builder
}

pub(crate) fn get_conversation_output_output_correct_errors(
    mut builder: crate::operation::get_conversation::builders::GetConversationOutputBuilder,
) -> crate::operation::get_conversation::builders::GetConversationOutputBuilder {
    if builder.conversation_id.is_none() {
        builder.conversation_id = Some(Default::default())
    }
    if builder.messages.is_none() {
        builder.messages = Some(Default::default())
    }
    builder
}

pub(crate) fn get_extension_output_output_correct_errors(
    mut builder: crate::operation::get_extension::builders::GetExtensionOutputBuilder,
) -> crate::operation::get_extension::builders::GetExtensionOutputBuilder {
    if builder.extension_provider.is_none() {
        builder.extension_provider = Some(Default::default())
    }
    if builder.extension_id.is_none() {
        builder.extension_id = Some(Default::default())
    }
    builder
}

pub(crate) fn get_plugin_output_output_correct_errors(
    mut builder: crate::operation::get_plugin::builders::GetPluginOutputBuilder,
) -> crate::operation::get_plugin::builders::GetPluginOutputBuilder {
    if builder.plugin_provider.is_none() {
        builder.plugin_provider = Some(Default::default())
    }
    if builder.plugin_id.is_none() {
        builder.plugin_id = Some(Default::default())
    }
    builder
}

pub(crate) fn get_task_output_output_correct_errors(
    mut builder: crate::operation::get_task::builders::GetTaskOutputBuilder,
) -> crate::operation::get_task::builders::GetTaskOutputBuilder {
    if builder.task_id.is_none() {
        builder.task_id = Some(Default::default())
    }
    if builder.state.is_none() {
        builder.state = "no value was set".parse::<crate::types::TaskState>().ok()
    }
    if builder.task_details.is_none() {
        builder.task_details = {
            let builder = crate::types::builders::TaskDetailsBuilder::default();
            crate::serde_util::task_details_correct_errors(builder).build().ok()
        }
    }
    if builder.last_updated_at.is_none() {
        builder.last_updated_at = Some(::aws_smithy_types::DateTime::from_fractional_secs(0, 0_f64))
    }
    builder
}

pub(crate) fn invoke_task_output_output_correct_errors(
    mut builder: crate::operation::invoke_task::builders::InvokeTaskOutputBuilder,
) -> crate::operation::invoke_task::builders::InvokeTaskOutputBuilder {
    if builder.task_id.is_none() {
        builder.task_id = Some(Default::default())
    }
    builder
}

pub(crate) fn list_conversations_output_output_correct_errors(
    mut builder: crate::operation::list_conversations::builders::ListConversationsOutputBuilder,
) -> crate::operation::list_conversations::builders::ListConversationsOutputBuilder {
    if builder.conversations.is_none() {
        builder.conversations = Some(Default::default())
    }
    builder
}

pub(crate) fn list_dashboard_metrics_output_output_correct_errors(
    mut builder: crate::operation::list_dashboard_metrics::builders::ListDashboardMetricsOutputBuilder,
) -> crate::operation::list_dashboard_metrics::builders::ListDashboardMetricsOutputBuilder {
    if builder.metrics.is_none() {
        builder.metrics = Some(Default::default())
    }
    builder
}

pub(crate) fn list_extension_providers_output_output_correct_errors(
    mut builder: crate::operation::list_extension_providers::builders::ListExtensionProvidersOutputBuilder,
) -> crate::operation::list_extension_providers::builders::ListExtensionProvidersOutputBuilder {
    if builder.extension_providers.is_none() {
        builder.extension_providers = Some(Default::default())
    }
    builder
}

pub(crate) fn list_extensions_output_output_correct_errors(
    mut builder: crate::operation::list_extensions::builders::ListExtensionsOutputBuilder,
) -> crate::operation::list_extensions::builders::ListExtensionsOutputBuilder {
    if builder.extensions.is_none() {
        builder.extensions = Some(Default::default())
    }
    builder
}

pub(crate) fn list_plugin_providers_output_output_correct_errors(
    mut builder: crate::operation::list_plugin_providers::builders::ListPluginProvidersOutputBuilder,
) -> crate::operation::list_plugin_providers::builders::ListPluginProvidersOutputBuilder {
    if builder.plugin_providers.is_none() {
        builder.plugin_providers = Some(Default::default())
    }
    builder
}

pub(crate) fn list_plugins_output_output_correct_errors(
    mut builder: crate::operation::list_plugins::builders::ListPluginsOutputBuilder,
) -> crate::operation::list_plugins::builders::ListPluginsOutputBuilder {
    if builder.plugins.is_none() {
        builder.plugins = Some(Default::default())
    }
    builder
}

pub(crate) fn list_tasks_output_output_correct_errors(
    mut builder: crate::operation::list_tasks::builders::ListTasksOutputBuilder,
) -> crate::operation::list_tasks::builders::ListTasksOutputBuilder {
    if builder.tasks.is_none() {
        builder.tasks = Some(Default::default())
    }
    builder
}

pub(crate) fn reject_connector_output_output_correct_errors(
    mut builder: crate::operation::reject_connector::builders::RejectConnectorOutputBuilder,
) -> crate::operation::reject_connector::builders::RejectConnectorOutputBuilder {
    if builder.connector_id.is_none() {
        builder.connector_id = Some(Default::default())
    }
    if builder.connector_name.is_none() {
        builder.connector_name = Some(Default::default())
    }
    if builder.connector_type.is_none() {
        builder.connector_type = Some(Default::default())
    }
    if builder.account_connection.is_none() {
        builder.account_connection = Some(crate::types::AccountConnection::Unknown)
    }
    builder
}

pub(crate) fn send_message_output_output_correct_errors(
    mut builder: crate::operation::send_message::builders::SendMessageOutputBuilder,
) -> crate::operation::send_message::builders::SendMessageOutputBuilder {
    if builder.result.is_none() {
        builder.result = {
            let builder = crate::types::builders::NellyResultBuilder::default();
            crate::serde_util::nelly_result_correct_errors(builder).build().ok()
        }
    }
    if builder.metadata.is_none() {
        builder.metadata = {
            let builder = crate::types::builders::NellyResponseMetadataBuilder::default();
            crate::serde_util::nelly_response_metadata_correct_errors(builder)
                .build()
                .ok()
        }
    }
    if builder.result_code.is_none() {
        builder.result_code = "no value was set".parse::<crate::types::ResultCode>().ok()
    }
    builder
}

pub(crate) fn start_conversation_output_output_correct_errors(
    mut builder: crate::operation::start_conversation::builders::StartConversationOutputBuilder,
) -> crate::operation::start_conversation::builders::StartConversationOutputBuilder {
    if builder.conversation_id.is_none() {
        builder.conversation_id = Some(Default::default())
    }
    builder
}

pub(crate) fn start_troubleshooting_analysis_output_output_correct_errors(
    mut builder: crate::operation::start_troubleshooting_analysis::builders::StartTroubleshootingAnalysisOutputBuilder,
) -> crate::operation::start_troubleshooting_analysis::builders::StartTroubleshootingAnalysisOutputBuilder {
    if builder.session_id.is_none() {
        builder.session_id = Some(Default::default())
    }
    builder
}

pub(crate) fn use_plugin_output_output_correct_errors(
    mut builder: crate::operation::use_plugin::builders::UsePluginOutputBuilder,
) -> crate::operation::use_plugin::builders::UsePluginOutputBuilder {
    if builder.is_authorized.is_none() {
        builder.is_authorized = Some(Default::default())
    }
    builder
}

pub(crate) fn task_details_correct_errors(
    mut builder: crate::types::builders::TaskDetailsBuilder,
) -> crate::types::builders::TaskDetailsBuilder {
    if builder.overview.is_none() {
        builder.overview = {
            let builder = crate::types::builders::TaskOverviewBuilder::default();
            crate::serde_util::task_overview_correct_errors(builder).build().ok()
        }
    }
    if builder.content.is_none() {
        builder.content = Some(Default::default())
    }
    builder
}

pub(crate) fn nelly_result_correct_errors(
    mut builder: crate::types::builders::NellyResultBuilder,
) -> crate::types::builders::NellyResultBuilder {
    if builder.r#type.is_none() {
        builder.r#type = "no value was set".parse::<crate::types::ResultType>().ok()
    }
    if builder.format.is_none() {
        builder.format = "no value was set".parse::<crate::types::ResultFormat>().ok()
    }
    if builder.content.is_none() {
        builder.content = Some(crate::types::NellyContent::Unknown)
    }
    builder
}

pub(crate) fn nelly_response_metadata_correct_errors(
    mut builder: crate::types::builders::NellyResponseMetadataBuilder,
) -> crate::types::builders::NellyResponseMetadataBuilder {
    if builder.conversation_id.is_none() {
        builder.conversation_id = Some(Default::default())
    }
    if builder.utterance_id.is_none() {
        builder.utterance_id = Some(Default::default())
    }
    builder
}

pub(crate) fn task_overview_correct_errors(
    mut builder: crate::types::builders::TaskOverviewBuilder,
) -> crate::types::builders::TaskOverviewBuilder {
    if builder.label.is_none() {
        builder.label = Some(Default::default())
    }
    if builder.description.is_none() {
        builder.description = Some(Default::default())
    }
    builder
}

pub(crate) fn conversation_metadata_correct_errors(
    mut builder: crate::types::builders::ConversationMetadataBuilder,
) -> crate::types::builders::ConversationMetadataBuilder {
    if builder.conversation_id.is_none() {
        builder.conversation_id = Some(Default::default())
    }
    if builder.timestamp.is_none() {
        builder.timestamp = Some(::aws_smithy_types::DateTime::from_fractional_secs(0, 0_f64))
    }
    builder
}

pub(crate) fn dashboard_metric_correct_errors(
    mut builder: crate::types::builders::DashboardMetricBuilder,
) -> crate::types::builders::DashboardMetricBuilder {
    if builder.dimensions.is_none() {
        builder.dimensions = {
            let builder = crate::types::builders::DimensionsBuilder::default();
            Some(builder.build())
        }
    }
    builder
}

pub(crate) fn encrypted_tool_fas_creds_correct_errors(
    mut builder: crate::types::builders::EncryptedToolFasCredsBuilder,
) -> crate::types::builders::EncryptedToolFasCredsBuilder {
    if builder.tool_id.is_none() {
        builder.tool_id = "no value was set".parse::<crate::types::ToolId>().ok()
    }
    if builder.encrypted_tool_fas_creds.is_none() {
        builder.encrypted_tool_fas_creds = Some(Default::default())
    }
    builder
}

pub(crate) fn extension_correct_errors(
    mut builder: crate::types::builders::ExtensionBuilder,
) -> crate::types::builders::ExtensionBuilder {
    if builder.extension_provider.is_none() {
        builder.extension_provider = Some(Default::default())
    }
    if builder.extension_id.is_none() {
        builder.extension_id = Some(Default::default())
    }
    builder
}

pub(crate) fn extension_provider_metadata_correct_errors(
    mut builder: crate::types::builders::ExtensionProviderMetadataBuilder,
) -> crate::types::builders::ExtensionProviderMetadataBuilder {
    if builder.extension_provider.is_none() {
        builder.extension_provider = Some(Default::default())
    }
    builder
}

pub(crate) fn message_correct_errors(
    mut builder: crate::types::builders::MessageBuilder,
) -> crate::types::builders::MessageBuilder {
    if builder.utterance_id.is_none() {
        builder.utterance_id = Some(Default::default())
    }
    if builder.r#type.is_none() {
        builder.r#type = "no value was set".parse::<crate::types::ResultType>().ok()
    }
    if builder.format.is_none() {
        builder.format = "no value was set".parse::<crate::types::ResultFormat>().ok()
    }
    if builder.content.is_none() {
        builder.content = Some(crate::types::NellyContent::Unknown)
    }
    if builder.from.is_none() {
        builder.from = "no value was set".parse::<crate::types::MessageFromType>().ok()
    }
    if builder.timestamp.is_none() {
        builder.timestamp = Some(::aws_smithy_types::DateTime::from_fractional_secs(0, 0_f64))
    }
    builder
}

pub(crate) fn plugin_correct_errors(
    mut builder: crate::types::builders::PluginBuilder,
) -> crate::types::builders::PluginBuilder {
    if builder.plugin_provider.is_none() {
        builder.plugin_provider = Some(Default::default())
    }
    if builder.plugin_id.is_none() {
        builder.plugin_id = Some(Default::default())
    }
    builder
}

pub(crate) fn plugin_provider_metadata_correct_errors(
    mut builder: crate::types::builders::PluginProviderMetadataBuilder,
) -> crate::types::builders::PluginProviderMetadataBuilder {
    if builder.plugin_provider.is_none() {
        builder.plugin_provider = Some(Default::default())
    }
    builder
}

pub(crate) fn tag_correct_errors(
    mut builder: crate::types::builders::TagBuilder,
) -> crate::types::builders::TagBuilder {
    if builder.key.is_none() {
        builder.key = Some(Default::default())
    }
    if builder.value.is_none() {
        builder.value = Some(Default::default())
    }
    builder
}

pub(crate) fn task_summary_correct_errors(
    mut builder: crate::types::builders::TaskSummaryBuilder,
) -> crate::types::builders::TaskSummaryBuilder {
    if builder.task_id.is_none() {
        builder.task_id = Some(Default::default())
    }
    if builder.state.is_none() {
        builder.state = "no value was set".parse::<crate::types::TaskState>().ok()
    }
    builder
}

pub(crate) fn task_action_correct_errors(
    mut builder: crate::types::builders::TaskActionBuilder,
) -> crate::types::builders::TaskActionBuilder {
    if builder.label.is_none() {
        builder.label = Some(Default::default())
    }
    if builder.payload.is_none() {
        builder.payload = Some(Default::default())
    }
    builder
}

pub(crate) fn text_content_correct_errors(
    mut builder: crate::types::builders::TextContentBuilder,
) -> crate::types::builders::TextContentBuilder {
    if builder.body.is_none() {
        builder.body = Some(Default::default())
    }
    builder
}

pub(crate) fn alert_correct_errors(
    mut builder: crate::types::builders::AlertBuilder,
) -> crate::types::builders::AlertBuilder {
    if builder.r#type.is_none() {
        builder.r#type = "no value was set".parse::<crate::types::AlertType>().ok()
    }
    if builder.content.is_none() {
        builder.content = Some(Default::default())
    }
    builder
}

pub(crate) fn programming_language_correct_errors(
    mut builder: crate::types::builders::ProgrammingLanguageBuilder,
) -> crate::types::builders::ProgrammingLanguageBuilder {
    if builder.language_name.is_none() {
        builder.language_name = Some(Default::default())
    }
    builder
}

pub(crate) fn progress_correct_errors(
    mut builder: crate::types::builders::ProgressBuilder,
) -> crate::types::builders::ProgressBuilder {
    if builder.content.is_none() {
        builder.content = Some(Default::default())
    }
    builder
}

pub(crate) fn resource_correct_errors(
    mut builder: crate::types::builders::ResourceBuilder,
) -> crate::types::builders::ResourceBuilder {
    if builder.title.is_none() {
        builder.title = Some(Default::default())
    }
    if builder.link.is_none() {
        builder.link = Some(Default::default())
    }
    if builder.description.is_none() {
        builder.description = Some(Default::default())
    }
    if builder.r#type.is_none() {
        builder.r#type = Some(Default::default())
    }
    if builder.arn.is_none() {
        builder.arn = Some(Default::default())
    }
    if builder.resource_json_string.is_none() {
        builder.resource_json_string = Some(Default::default())
    }
    builder
}

pub(crate) fn resource_list_correct_errors(
    mut builder: crate::types::builders::ResourceListBuilder,
) -> crate::types::builders::ResourceListBuilder {
    if builder.items.is_none() {
        builder.items = Some(Default::default())
    }
    builder
}

pub(crate) fn section_correct_errors(
    mut builder: crate::types::builders::SectionBuilder,
) -> crate::types::builders::SectionBuilder {
    if builder.title.is_none() {
        builder.title = Some(Default::default())
    }
    if builder.content.is_none() {
        builder.content = Some(Default::default())
    }
    builder
}

pub(crate) fn step_correct_errors(
    mut builder: crate::types::builders::StepBuilder,
) -> crate::types::builders::StepBuilder {
    if builder.id.is_none() {
        builder.id = Some(Default::default())
    }
    if builder.state.is_none() {
        builder.state = "no value was set".parse::<crate::types::StepState>().ok()
    }
    if builder.label.is_none() {
        builder.label = Some(Default::default())
    }
    builder
}

pub(crate) fn suggestions_correct_errors(
    mut builder: crate::types::builders::SuggestionsBuilder,
) -> crate::types::builders::SuggestionsBuilder {
    if builder.items.is_none() {
        builder.items = Some(Default::default())
    }
    builder
}

pub(crate) fn task_action_note_correct_errors(
    mut builder: crate::types::builders::TaskActionNoteBuilder,
) -> crate::types::builders::TaskActionNoteBuilder {
    if builder.content.is_none() {
        builder.content = Some(Default::default())
    }
    builder
}

pub(crate) fn task_reference_correct_errors(
    mut builder: crate::types::builders::TaskReferenceBuilder,
) -> crate::types::builders::TaskReferenceBuilder {
    if builder.task_id.is_none() {
        builder.task_id = Some(Default::default())
    }
    builder
}

pub(crate) fn text_correct_errors(
    mut builder: crate::types::builders::TextBuilder,
) -> crate::types::builders::TextBuilder {
    if builder.content.is_none() {
        builder.content = Some(Default::default())
    }
    builder
}

pub(crate) fn infrastructure_update_transition_correct_errors(
    mut builder: crate::types::builders::InfrastructureUpdateTransitionBuilder,
) -> crate::types::builders::InfrastructureUpdateTransitionBuilder {
    if builder.current_state.is_none() {
        builder.current_state = Some(Default::default())
    }
    if builder.next_state.is_none() {
        builder.next_state = Some(Default::default())
    }
    builder
}

pub(crate) fn nelly_license_correct_errors(
    mut builder: crate::types::builders::NellyLicenseBuilder,
) -> crate::types::builders::NellyLicenseBuilder {
    if builder.id.is_none() {
        builder.id = Some(Default::default())
    }
    if builder.license_name.is_none() {
        builder.license_name = Some(Default::default())
    }
    builder
}

pub(crate) fn nelly_url_correct_errors(
    mut builder: crate::types::builders::NellyUrlBuilder,
) -> crate::types::builders::NellyUrlBuilder {
    if builder.id.is_none() {
        builder.id = Some(Default::default())
    }
    if builder.url.is_none() {
        builder.url = Some(Default::default())
    }
    builder
}

pub(crate) fn web_link_correct_errors(
    mut builder: crate::types::builders::WebLinkBuilder,
) -> crate::types::builders::WebLinkBuilder {
    if builder.label.is_none() {
        builder.label = Some(Default::default())
    }
    if builder.url.is_none() {
        builder.url = Some(Default::default())
    }
    builder
}

pub(crate) fn cloud_watch_troubleshooting_link_correct_errors(
    mut builder: crate::types::builders::CloudWatchTroubleshootingLinkBuilder,
) -> crate::types::builders::CloudWatchTroubleshootingLinkBuilder {
    if builder.label.is_none() {
        builder.label = Some(Default::default())
    }
    if builder.investigation_payload.is_none() {
        builder.investigation_payload = Some(Default::default())
    }
    builder
}

pub(crate) fn suggestion_correct_errors(
    mut builder: crate::types::builders::SuggestionBuilder,
) -> crate::types::builders::SuggestionBuilder {
    if builder.value.is_none() {
        builder.value = Some(Default::default())
    }
    builder
}
