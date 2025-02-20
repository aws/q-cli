// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the
    /// [`ListWorkspaceMetadata`](crate::operation::list_workspace_metadata::builders::ListWorkspaceMetadataFluentBuilder)
    /// operation. This operation supports pagination; See
    /// [`into_paginator()`](crate::operation::list_workspace_metadata::builders::ListWorkspaceMetadataFluentBuilder::into_paginator).
    ///
    ///
    /// - The fluent builder is configurable:
    ///   - [`workspace_root(impl Into<String>)`](crate::operation::list_workspace_metadata::builders::ListWorkspaceMetadataFluentBuilder::workspace_root) / [`set_workspace_root(Option<String>)`](crate::operation::list_workspace_metadata::builders::ListWorkspaceMetadataFluentBuilder::set_workspace_root):<br>required: **true**<br>(undocumented)<br>
    ///   - [`next_token(impl Into<String>)`](crate::operation::list_workspace_metadata::builders::ListWorkspaceMetadataFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::list_workspace_metadata::builders::ListWorkspaceMetadataFluentBuilder::set_next_token):<br>required: **false**<br>(undocumented)<br>
    ///   - [`max_results(i32)`](crate::operation::list_workspace_metadata::builders::ListWorkspaceMetadataFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::list_workspace_metadata::builders::ListWorkspaceMetadataFluentBuilder::set_max_results):<br>required: **false**<br>(undocumented)<br>
    /// - On success, responds with
    ///   [`ListWorkspaceMetadataOutput`](crate::operation::list_workspace_metadata::ListWorkspaceMetadataOutput)
    ///   with field(s):
    ///   - [`workspaces(Vec::<WorkspaceMetadata>)`](crate::operation::list_workspace_metadata::ListWorkspaceMetadataOutput::workspaces): (undocumented)
    ///   - [`next_token(Option<String>)`](crate::operation::list_workspace_metadata::ListWorkspaceMetadataOutput::next_token): (undocumented)
    /// - On failure, responds with
    ///   [`SdkError<ListWorkspaceMetadataError>`](crate::operation::list_workspace_metadata::ListWorkspaceMetadataError)
    pub fn list_workspace_metadata(
        &self,
    ) -> crate::operation::list_workspace_metadata::builders::ListWorkspaceMetadataFluentBuilder {
        crate::operation::list_workspace_metadata::builders::ListWorkspaceMetadataFluentBuilder::new(
            self.handle.clone(),
        )
    }
}
