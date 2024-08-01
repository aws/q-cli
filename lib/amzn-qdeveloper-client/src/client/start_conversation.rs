// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the
    /// [`StartConversation`](crate::operation::start_conversation::builders::StartConversationFluentBuilder)
    /// operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`origin(impl Into<String>)`](crate::operation::start_conversation::builders::StartConversationFluentBuilder::origin) / [`set_origin(Option<String>)`](crate::operation::start_conversation::builders::StartConversationFluentBuilder::set_origin):<br>required: **false**<br>(undocumented)<br>
    ///   - [`source(Origin)`](crate::operation::start_conversation::builders::StartConversationFluentBuilder::source) / [`set_source(Option<Origin>)`](crate::operation::start_conversation::builders::StartConversationFluentBuilder::set_source):<br>required: **false**<br>Enum to represent the origin application conversing with Sidekick.<br>
    ///   - [`dry_run(bool)`](crate::operation::start_conversation::builders::StartConversationFluentBuilder::dry_run) / [`set_dry_run(Option<bool>)`](crate::operation::start_conversation::builders::StartConversationFluentBuilder::set_dry_run):<br>required: **false**<br>(undocumented)<br>
    /// - On success, responds with
    ///   [`StartConversationOutput`](crate::operation::start_conversation::StartConversationOutput)
    ///   with field(s):
    ///   - [`conversation_id(String)`](crate::operation::start_conversation::StartConversationOutput::conversation_id): (undocumented)
    ///   - [`conversation_token(Option<String>)`](crate::operation::start_conversation::StartConversationOutput::conversation_token): (undocumented)
    ///   - [`expiration_time(Option<String>)`](crate::operation::start_conversation::StartConversationOutput::expiration_time): (undocumented)
    /// - On failure, responds with
    ///   [`SdkError<StartConversationError>`](crate::operation::start_conversation::StartConversationError)
    pub fn start_conversation(&self) -> crate::operation::start_conversation::builders::StartConversationFluentBuilder {
        crate::operation::start_conversation::builders::StartConversationFluentBuilder::new(self.handle.clone())
    }
}
