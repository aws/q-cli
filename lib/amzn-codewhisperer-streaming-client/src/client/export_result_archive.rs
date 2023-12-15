// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`ExportResultArchive`](crate::operation::export_result_archive::builders::ExportResultArchiveFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`export_id(impl Into<String>)`](crate::operation::export_result_archive::builders::ExportResultArchiveFluentBuilder::export_id) / [`set_export_id(Option<String>)`](crate::operation::export_result_archive::builders::ExportResultArchiveFluentBuilder::set_export_id):<br>required: **true**<br>(undocumented)<br>
    ///   - [`export_intent(ExportIntent)`](crate::operation::export_result_archive::builders::ExportResultArchiveFluentBuilder::export_intent) / [`set_export_intent(Option<ExportIntent>)`](crate::operation::export_result_archive::builders::ExportResultArchiveFluentBuilder::set_export_intent):<br>required: **true**<br>Export Intent<br>
    /// - On success, responds with [`ExportResultArchiveOutput`](crate::operation::export_result_archive::ExportResultArchiveOutput) with field(s):
    ///   - [`body(EventReceiver<ResultArchiveStream, ResultArchiveStreamError>)`](crate::operation::export_result_archive::ExportResultArchiveOutput::body): Response Stream
    /// - On failure, responds with [`SdkError<ExportResultArchiveError>`](crate::operation::export_result_archive::ExportResultArchiveError)
    pub fn export_result_archive(&self) -> crate::operation::export_result_archive::builders::ExportResultArchiveFluentBuilder {
        crate::operation::export_result_archive::builders::ExportResultArchiveFluentBuilder::new(self.handle.clone())
    }
}
