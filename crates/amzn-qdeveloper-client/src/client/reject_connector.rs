// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the
    /// [`RejectConnector`](crate::operation::reject_connector::builders::RejectConnectorFluentBuilder)
    /// operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`connector_id(impl Into<String>)`](crate::operation::reject_connector::builders::RejectConnectorFluentBuilder::connector_id) / [`set_connector_id(Option<String>)`](crate::operation::reject_connector::builders::RejectConnectorFluentBuilder::set_connector_id):<br>required: **true**<br>(undocumented)<br>
    ///   - [`client_token(impl Into<String>)`](crate::operation::reject_connector::builders::RejectConnectorFluentBuilder::client_token) / [`set_client_token(Option<String>)`](crate::operation::reject_connector::builders::RejectConnectorFluentBuilder::set_client_token):<br>required: **false**<br>(undocumented)<br>
    /// - On success, responds with
    ///   [`RejectConnectorOutput`](crate::operation::reject_connector::RejectConnectorOutput) with
    ///   field(s):
    ///   - [`connector_id(String)`](crate::operation::reject_connector::RejectConnectorOutput::connector_id): (undocumented)
    ///   - [`connector_name(String)`](crate::operation::reject_connector::RejectConnectorOutput::connector_name): Common non-blank String data type used for multiple parameters with a length restriction
    ///   - [`connector_type(String)`](crate::operation::reject_connector::RejectConnectorOutput::connector_type): Connector types like S3, CodeConnection etc
    ///   - [`account_connection(AccountConnection)`](crate::operation::reject_connector::RejectConnectorOutput::account_connection): Connector target account information
    /// - On failure, responds with
    ///   [`SdkError<RejectConnectorError>`](crate::operation::reject_connector::RejectConnectorError)
    pub fn reject_connector(&self) -> crate::operation::reject_connector::builders::RejectConnectorFluentBuilder {
        crate::operation::reject_connector::builders::RejectConnectorFluentBuilder::new(self.handle.clone())
    }
}
