// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the
    /// [`ListCustomizationVersions`](crate::operation::list_customization_versions::builders::ListCustomizationVersionsFluentBuilder)
    /// operation. This operation supports pagination; See
    /// [`into_paginator()`](crate::operation::list_customization_versions::builders::ListCustomizationVersionsFluentBuilder::into_paginator).
    ///
    ///
    /// - The fluent builder is configurable:
    ///   - [`identifier(impl Into<String>)`](crate::operation::list_customization_versions::builders::ListCustomizationVersionsFluentBuilder::identifier) / [`set_identifier(Option<String>)`](crate::operation::list_customization_versions::builders::ListCustomizationVersionsFluentBuilder::set_identifier):<br>required: **true**<br>(undocumented)<br>
    ///   - [`max_results(i32)`](crate::operation::list_customization_versions::builders::ListCustomizationVersionsFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::list_customization_versions::builders::ListCustomizationVersionsFluentBuilder::set_max_results):<br>required: **false**<br>(undocumented)<br>
    ///   - [`next_token(impl Into<String>)`](crate::operation::list_customization_versions::builders::ListCustomizationVersionsFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::list_customization_versions::builders::ListCustomizationVersionsFluentBuilder::set_next_token):<br>required: **false**<br>(undocumented)<br>
    /// - On success, responds with
    ///   [`ListCustomizationVersionsOutput`](crate::operation::list_customization_versions::ListCustomizationVersionsOutput)
    ///   with field(s):
    ///   - [`versions(Vec::<CustomizationVersionSummary>)`](crate::operation::list_customization_versions::ListCustomizationVersionsOutput::versions): (undocumented)
    ///   - [`next_token(Option<String>)`](crate::operation::list_customization_versions::ListCustomizationVersionsOutput::next_token): (undocumented)
    /// - On failure, responds with [`SdkError<ListCustomizationVersionsError>`](crate::operation::list_customization_versions::ListCustomizationVersionsError)
    pub fn list_customization_versions(
        &self,
    ) -> crate::operation::list_customization_versions::builders::ListCustomizationVersionsFluentBuilder {
        crate::operation::list_customization_versions::builders::ListCustomizationVersionsFluentBuilder::new(
            self.handle.clone(),
        )
    }
}
