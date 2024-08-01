// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
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

pub(crate) fn internal_server_error_correct_errors(
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

pub(crate) fn service_quota_exceeded_exception_correct_errors(
    mut builder: crate::types::error::builders::ServiceQuotaExceededErrorBuilder,
) -> crate::types::error::builders::ServiceQuotaExceededErrorBuilder {
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

pub(crate) fn resource_not_found_exception_correct_errors(
    mut builder: crate::types::error::builders::ResourceNotFoundErrorBuilder,
) -> crate::types::error::builders::ResourceNotFoundErrorBuilder {
    if builder.message.is_none() {
        builder.message = Some(Default::default())
    }
    builder
}

pub(crate) fn assistant_response_event_correct_errors(
    mut builder: crate::types::builders::AssistantResponseEventBuilder,
) -> crate::types::builders::AssistantResponseEventBuilder {
    if builder.content.is_none() {
        builder.content = Some(Default::default())
    }
    builder
}

pub(crate) fn code_event_correct_errors(
    mut builder: crate::types::builders::CodeEventBuilder,
) -> crate::types::builders::CodeEventBuilder {
    if builder.content.is_none() {
        builder.content = Some(Default::default())
    }
    builder
}

pub(crate) fn invalid_state_event_correct_errors(
    mut builder: crate::types::builders::InvalidStateEventBuilder,
) -> crate::types::builders::InvalidStateEventBuilder {
    if builder.reason.is_none() {
        builder.reason = "no value was set".parse::<crate::types::InvalidStateReason>().ok()
    }
    if builder.message.is_none() {
        builder.message = Some(Default::default())
    }
    builder
}

pub(crate) fn followup_prompt_correct_errors(
    mut builder: crate::types::builders::FollowupPromptBuilder,
) -> crate::types::builders::FollowupPromptBuilder {
    if builder.content.is_none() {
        builder.content = Some(Default::default())
    }
    builder
}

pub(crate) fn supplementary_web_link_correct_errors(
    mut builder: crate::types::builders::SupplementaryWebLinkBuilder,
) -> crate::types::builders::SupplementaryWebLinkBuilder {
    if builder.url.is_none() {
        builder.url = Some(Default::default())
    }
    if builder.title.is_none() {
        builder.title = Some(Default::default())
    }
    builder
}
