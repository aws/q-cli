// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the
    /// [`UpdateProfile`](crate::operation::update_profile::builders::UpdateProfileFluentBuilder)
    /// operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`profile_arn(impl Into<String>)`](crate::operation::update_profile::builders::UpdateProfileFluentBuilder::profile_arn) / [`set_profile_arn(Option<String>)`](crate::operation::update_profile::builders::UpdateProfileFluentBuilder::set_profile_arn):<br>required: **true**<br>(undocumented)<br>
    ///   - [`profile_name(impl Into<String>)`](crate::operation::update_profile::builders::UpdateProfileFluentBuilder::profile_name) / [`set_profile_name(Option<String>)`](crate::operation::update_profile::builders::UpdateProfileFluentBuilder::set_profile_name):<br>required: **false**<br>(undocumented)<br>
    ///   - [`reference_tracker_configuration(ReferenceTrackerConfiguration)`](crate::operation::update_profile::builders::UpdateProfileFluentBuilder::reference_tracker_configuration) / [`set_reference_tracker_configuration(Option<ReferenceTrackerConfiguration>)`](crate::operation::update_profile::builders::UpdateProfileFluentBuilder::set_reference_tracker_configuration):<br>required: **false**<br>(undocumented)<br>
    ///   - [`active_functionalities(FunctionalityName)`](crate::operation::update_profile::builders::UpdateProfileFluentBuilder::active_functionalities) / [`set_active_functionalities(Option<Vec::<FunctionalityName>>)`](crate::operation::update_profile::builders::UpdateProfileFluentBuilder::set_active_functionalities):<br>required: **false**<br>(undocumented)<br>
    ///   - [`kms_key_arn(impl Into<String>)`](crate::operation::update_profile::builders::UpdateProfileFluentBuilder::kms_key_arn) / [`set_kms_key_arn(Option<String>)`](crate::operation::update_profile::builders::UpdateProfileFluentBuilder::set_kms_key_arn):<br>required: **false**<br>(undocumented)<br>
    /// - On success, responds with
    ///   [`UpdateProfileOutput`](crate::operation::update_profile::UpdateProfileOutput) with
    ///   field(s):
    ///   - [`profile_arn(String)`](crate::operation::update_profile::UpdateProfileOutput::profile_arn): (undocumented)
    /// - On failure, responds with
    ///   [`SdkError<UpdateProfileError>`](crate::operation::update_profile::UpdateProfileError)
    pub fn update_profile(&self) -> crate::operation::update_profile::builders::UpdateProfileFluentBuilder {
        crate::operation::update_profile::builders::UpdateProfileFluentBuilder::new(self.handle.clone())
    }
}
