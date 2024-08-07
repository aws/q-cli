use amzn_codewhisperer_client::operation::generate_completions::GenerateCompletionsError;
use amzn_codewhisperer_client::operation::list_available_customizations::ListAvailableCustomizationsError;
use amzn_codewhisperer_streaming_client::operation::generate_assistant_response::GenerateAssistantResponseError;
// use amzn_codewhisperer_streaming_client::operation::send_message::SendMessageError as
// CodewhispererSendMessageError;
use amzn_codewhisperer_streaming_client::types::error::ChatResponseStreamError as CodewhispererChatResponseStreamError;
use amzn_consolas_client::operation::generate_recommendations::GenerateRecommendationsError;
use amzn_consolas_client::operation::list_customizations::ListCustomizationsError;
use amzn_qdeveloper_streaming_client::operation::send_message::SendMessageError as QDeveloperSendMessageError;
use amzn_qdeveloper_streaming_client::types::error::ChatResponseStreamError as QDeveloperChatResponseStreamError;
use aws_credential_types::provider::error::CredentialsError;
use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
use aws_smithy_runtime_api::client::result::SdkError;
use aws_smithy_types::event_stream::RawMessage;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to load credentials: {}", .0)]
    Credentials(CredentialsError),

    // Generate completions errors
    #[error(transparent)]
    GenerateCompletions(#[from] SdkError<GenerateCompletionsError, HttpResponse>),
    #[error(transparent)]
    GenerateRecommendations(#[from] SdkError<GenerateRecommendationsError, HttpResponse>),

    // List customizations error
    #[error(transparent)]
    ListAvailableCustomizations(#[from] SdkError<ListAvailableCustomizationsError, HttpResponse>),
    #[error(transparent)]
    ListAvailableServices(#[from] SdkError<ListCustomizationsError, HttpResponse>),

    // Send message errors
    #[error(transparent)]
    CodewhispererGenerateAssistantResponse(#[from] SdkError<GenerateAssistantResponseError, HttpResponse>),
    #[error(transparent)]
    QDeveloperSendMessage(#[from] SdkError<QDeveloperSendMessageError, HttpResponse>),

    // chat stream errors
    #[error(transparent)]
    CodewhispererChatResponseStream(#[from] SdkError<CodewhispererChatResponseStreamError, RawMessage>),
    #[error(transparent)]
    QDeveloperChatResponseStream(#[from] SdkError<QDeveloperChatResponseStreamError, RawMessage>),

    #[error("Unsupported action by consolas: {0}")]
    UnsupportedConsolas(&'static str),
    #[error("Unsupported action by codewhisperer: {0}")]
    UnsupportedCodewhisperer(&'static str),
}

impl Error {
    pub fn is_throttling_error(&self) -> bool {
        match self {
            Error::Credentials(_) => false,
            Error::GenerateCompletions(e) => e.as_service_error().map_or(false, |e| e.is_throttling_error()),
            Error::GenerateRecommendations(e) => e.as_service_error().map_or(false, |e| e.is_throttling_error()),
            Error::ListAvailableCustomizations(e) => e.as_service_error().map_or(false, |e| e.is_throttling_error()),
            Error::ListAvailableServices(e) => e.as_service_error().map_or(false, |e| e.is_throttling_error()),
            Error::CodewhispererGenerateAssistantResponse(e) => {
                e.as_service_error().map_or(false, |e| e.is_throttling_error())
            },
            Error::QDeveloperSendMessage(e) => e.as_service_error().map_or(false, |e| e.is_throttling_error()),
            Error::CodewhispererChatResponseStream(_)
            | Error::QDeveloperChatResponseStream(_)
            | Error::UnsupportedConsolas(_)
            | Error::UnsupportedCodewhisperer(_) => false,
        }
    }
}
