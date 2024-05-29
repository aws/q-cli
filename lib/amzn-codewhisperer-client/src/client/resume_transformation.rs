// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the
    /// [`ResumeTransformation`](crate::operation::resume_transformation::builders::ResumeTransformationFluentBuilder)
    /// operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`transformation_job_id(impl Into<String>)`](crate::operation::resume_transformation::builders::ResumeTransformationFluentBuilder::transformation_job_id) / [`set_transformation_job_id(Option<String>)`](crate::operation::resume_transformation::builders::ResumeTransformationFluentBuilder::set_transformation_job_id):<br>required: **true**<br>Identifier for the Transformation Job<br>
    ///   - [`user_action_status(TransformationUserActionStatus)`](crate::operation::resume_transformation::builders::ResumeTransformationFluentBuilder::user_action_status) / [`set_user_action_status(Option<TransformationUserActionStatus>)`](crate::operation::resume_transformation::builders::ResumeTransformationFluentBuilder::set_user_action_status):<br>required: **false**<br>(undocumented)<br>
    /// - On success, responds with
    ///   [`ResumeTransformationOutput`](crate::operation::resume_transformation::ResumeTransformationOutput)
    ///   with field(s):
    ///   - [`transformation_status(TransformationStatus)`](crate::operation::resume_transformation::ResumeTransformationOutput::transformation_status): (undocumented)
    /// - On failure, responds with
    ///   [`SdkError<ResumeTransformationError>`](crate::operation::resume_transformation::ResumeTransformationError)
    pub fn resume_transformation(
        &self,
    ) -> crate::operation::resume_transformation::builders::ResumeTransformationFluentBuilder {
        crate::operation::resume_transformation::builders::ResumeTransformationFluentBuilder::new(self.handle.clone())
    }
}
