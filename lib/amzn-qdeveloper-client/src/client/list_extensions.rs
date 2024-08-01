// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the
    /// [`ListExtensions`](crate::operation::list_extensions::builders::ListExtensionsFluentBuilder)
    /// operation. This operation supports pagination; See
    /// [`into_paginator()`](crate::operation::list_extensions::builders::ListExtensionsFluentBuilder::into_paginator).
    ///
    ///
    /// - The fluent builder is configurable:
    ///   - [`max_results(i32)`](crate::operation::list_extensions::builders::ListExtensionsFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::list_extensions::builders::ListExtensionsFluentBuilder::set_max_results):<br>required: **false**<br>(undocumented)<br>
    ///   - [`next_token(impl Into<String>)`](crate::operation::list_extensions::builders::ListExtensionsFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::list_extensions::builders::ListExtensionsFluentBuilder::set_next_token):<br>required: **false**<br>(undocumented)<br>
    /// - On success, responds with
    ///   [`ListExtensionsOutput`](crate::operation::list_extensions::ListExtensionsOutput) with
    ///   field(s):
    ///   - [`extensions(Vec::<Extension>)`](crate::operation::list_extensions::ListExtensionsOutput::extensions): (undocumented)
    ///   - [`next_token(Option<String>)`](crate::operation::list_extensions::ListExtensionsOutput::next_token): (undocumented)
    /// - On failure, responds with
    ///   [`SdkError<ListExtensionsError>`](crate::operation::list_extensions::ListExtensionsError)
    pub fn list_extensions(&self) -> crate::operation::list_extensions::builders::ListExtensionsFluentBuilder {
        crate::operation::list_extensions::builders::ListExtensionsFluentBuilder::new(self.handle.clone())
    }
}
