// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the
    /// [`CreateResolution`](crate::operation::create_resolution::builders::CreateResolutionFluentBuilder)
    /// operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`session_id(impl Into<String>)`](crate::operation::create_resolution::builders::CreateResolutionFluentBuilder::session_id) / [`set_session_id(Option<String>)`](crate::operation::create_resolution::builders::CreateResolutionFluentBuilder::set_session_id):<br>required: **true**<br>(undocumented)<br>
    /// - On success, responds with
    ///   [`CreateResolutionOutput`](crate::operation::create_resolution::CreateResolutionOutput)
    /// - On failure, responds with
    ///   [`SdkError<CreateResolutionError>`](crate::operation::create_resolution::CreateResolutionError)
    pub fn create_resolution(&self) -> crate::operation::create_resolution::builders::CreateResolutionFluentBuilder {
        crate::operation::create_resolution::builders::CreateResolutionFluentBuilder::new(self.handle.clone())
    }
}
