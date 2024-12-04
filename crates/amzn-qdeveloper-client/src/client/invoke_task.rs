// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the
    /// [`InvokeTask`](crate::operation::invoke_task::builders::InvokeTaskFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`task_id(impl
    ///     Into<String>)`](crate::operation::invoke_task::builders::InvokeTaskFluentBuilder::task_id)
    ///     / [`set_task_id(Option<String>)`](crate::operation::invoke_task::builders::InvokeTaskFluentBuilder::set_task_id):
    ///     <br>required: **true**<br>(undocumented)<br>
    ///   - [`payload(impl Into<String>, impl
    ///     Into<String>)`](crate::operation::invoke_task::builders::InvokeTaskFluentBuilder::payload)
    ///     / [`set_payload(Option<HashMap::<String,
    ///     String>>)`](crate::operation::invoke_task::builders::InvokeTaskFluentBuilder::set_payload):
    ///     <br>required: **true**<br>Map representing key-value pairs for the payload of a task
    ///     action.<br>
    /// - On success, responds with
    ///   [`InvokeTaskOutput`](crate::operation::invoke_task::InvokeTaskOutput) with field(s):
    ///   - [`task_id(String)`](crate::operation::invoke_task::InvokeTaskOutput::task_id):
    ///     (undocumented)
    /// - On failure, responds with
    ///   [`SdkError<InvokeTaskError>`](crate::operation::invoke_task::InvokeTaskError)
    pub fn invoke_task(&self) -> crate::operation::invoke_task::builders::InvokeTaskFluentBuilder {
        crate::operation::invoke_task::builders::InvokeTaskFluentBuilder::new(self.handle.clone())
    }
}
