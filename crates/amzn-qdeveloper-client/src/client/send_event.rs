// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the
    /// [`SendEvent`](crate::operation::send_event::builders::SendEventFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`client_token(impl Into<String>)`](crate::operation::send_event::builders::SendEventFluentBuilder::client_token) / [`set_client_token(Option<String>)`](crate::operation::send_event::builders::SendEventFluentBuilder::set_client_token):<br>required: **true**<br>(undocumented)<br>
    ///   - [`provider_id(SupportedProviderId)`](crate::operation::send_event::builders::SendEventFluentBuilder::provider_id) / [`set_provider_id(Option<SupportedProviderId>)`](crate::operation::send_event::builders::SendEventFluentBuilder::set_provider_id):<br>required: **true**<br>Currently supported providers for receiving events.<br>
    ///   - [`event_id(impl
    ///     Into<String>)`](crate::operation::send_event::builders::SendEventFluentBuilder::event_id)
    ///     / [`set_event_id(Option<String>)`](crate::operation::send_event::builders::SendEventFluentBuilder::set_event_id):
    ///     <br>required: **true**<br>(undocumented)<br>
    ///   - [`event_version(impl Into<String>)`](crate::operation::send_event::builders::SendEventFluentBuilder::event_version) / [`set_event_version(Option<String>)`](crate::operation::send_event::builders::SendEventFluentBuilder::set_event_version):<br>required: **true**<br>(undocumented)<br>
    ///   - [`event(Blob)`](crate::operation::send_event::builders::SendEventFluentBuilder::event) /
    ///     [`set_event(Option<Blob>)`](crate::operation::send_event::builders::SendEventFluentBuilder::set_event):
    ///     <br>required: **true**<br>(undocumented)<br>
    /// - On success, responds with
    ///   [`SendEventOutput`](crate::operation::send_event::SendEventOutput) with field(s):
    ///   - [`client_token(Option<String>)`](crate::operation::send_event::SendEventOutput::client_token): (undocumented)
    ///   - [`provider_id(Option<SupportedProviderId>)`](crate::operation::send_event::SendEventOutput::provider_id): Currently supported providers for receiving events.
    ///   - [`event_id(Option<String>)`](crate::operation::send_event::SendEventOutput::event_id):
    ///     (undocumented)
    ///   - [`event_version(Option<String>)`](crate::operation::send_event::SendEventOutput::event_version): (undocumented)
    /// - On failure, responds with
    ///   [`SdkError<SendEventError>`](crate::operation::send_event::SendEventError)
    pub fn send_event(&self) -> crate::operation::send_event::builders::SendEventFluentBuilder {
        crate::operation::send_event::builders::SendEventFluentBuilder::new(self.handle.clone())
    }
}
