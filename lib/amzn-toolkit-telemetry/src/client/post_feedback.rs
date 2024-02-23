// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the
    /// [`PostFeedback`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder)
    /// operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`aws_product(AwsProduct)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::aws_product) / [`set_aws_product(Option<AwsProduct>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::set_aws_product):<br>required: **true**<br>(undocumented)<br>
    ///   - [`aws_product_version(impl Into<String>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::aws_product_version) / [`set_aws_product_version(Option<String>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::set_aws_product_version):<br>required: **true**<br>(undocumented)<br>
    ///   - [`os(impl Into<String>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::os) / [`set_os(Option<String>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::set_os):<br>required: **false**<br>(undocumented)<br>
    ///   - [`os_version(impl Into<String>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::os_version) / [`set_os_version(Option<String>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::set_os_version):<br>required: **false**<br>(undocumented)<br>
    ///   - [`parent_product(impl Into<String>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::parent_product) / [`set_parent_product(Option<String>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::set_parent_product):<br>required: **true**<br>(undocumented)<br>
    ///   - [`parent_product_version(impl Into<String>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::parent_product_version) / [`set_parent_product_version(Option<String>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::set_parent_product_version):<br>required: **true**<br>(undocumented)<br>
    ///   - [`metadata(MetadataEntry)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::metadata) / [`set_metadata(Option<Vec::<MetadataEntry>>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::set_metadata):<br>required: **false**<br>(undocumented)<br>
    ///   - [`sentiment(Sentiment)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::sentiment) / [`set_sentiment(Option<Sentiment>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::set_sentiment):<br>required: **true**<br>(undocumented)<br>
    ///   - [`comment(impl
    ///     Into<String>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::comment)
    ///     / [`set_comment(Option<String>)`](crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::set_comment):
    ///     <br>required: **true**<br>(undocumented)<br>
    /// - On success, responds with
    ///   [`PostFeedbackOutput`](crate::operation::post_feedback::PostFeedbackOutput)
    /// - On failure, responds with
    ///   [`SdkError<PostFeedbackError>`](crate::operation::post_feedback::PostFeedbackError)
    pub fn post_feedback(&self) -> crate::operation::post_feedback::builders::PostFeedbackFluentBuilder {
        crate::operation::post_feedback::builders::PostFeedbackFluentBuilder::new(self.handle.clone())
    }
}
