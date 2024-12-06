// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the
    /// [`UsePlugin`](crate::operation::use_plugin::builders::UsePluginFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`plugin_arn(impl
    ///     Into<String>)`](crate::operation::use_plugin::builders::UsePluginFluentBuilder::plugin_arn)
    ///     / [`set_plugin_arn(Option<String>)`](crate::operation::use_plugin::builders::UsePluginFluentBuilder::set_plugin_arn):
    ///     <br>required: **true**<br>(undocumented)<br>
    /// - On success, responds with
    ///   [`UsePluginOutput`](crate::operation::use_plugin::UsePluginOutput) with field(s):
    ///   - [`is_authorized(bool)`](crate::operation::use_plugin::UsePluginOutput::is_authorized):
    ///     (undocumented)
    /// - On failure, responds with
    ///   [`SdkError<UsePluginError>`](crate::operation::use_plugin::UsePluginError)
    pub fn use_plugin(&self) -> crate::operation::use_plugin::builders::UsePluginFluentBuilder {
        crate::operation::use_plugin::builders::UsePluginFluentBuilder::new(self.handle.clone())
    }
}
