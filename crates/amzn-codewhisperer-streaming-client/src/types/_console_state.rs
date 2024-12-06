// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Information about the state of the AWS management console page from which the user is calling
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq)]
pub struct ConsoleState {
    #[allow(missing_docs)] // documentation missing in model
    pub region: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub console_url: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub service_id: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub service_console_page: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub service_subconsole_page: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub task_name: ::std::option::Option<::std::string::String>,
}
impl ConsoleState {
    #[allow(missing_docs)] // documentation missing in model
    pub fn region(&self) -> ::std::option::Option<&str> {
        self.region.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn console_url(&self) -> ::std::option::Option<&str> {
        self.console_url.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn service_id(&self) -> ::std::option::Option<&str> {
        self.service_id.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn service_console_page(&self) -> ::std::option::Option<&str> {
        self.service_console_page.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn service_subconsole_page(&self) -> ::std::option::Option<&str> {
        self.service_subconsole_page.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn task_name(&self) -> ::std::option::Option<&str> {
        self.task_name.as_deref()
    }
}
impl ::std::fmt::Debug for ConsoleState {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut formatter = f.debug_struct("ConsoleState");
        formatter.field("region", &self.region);
        formatter.field("console_url", &"*** Sensitive Data Redacted ***");
        formatter.field("service_id", &self.service_id);
        formatter.field("service_console_page", &self.service_console_page);
        formatter.field("service_subconsole_page", &self.service_subconsole_page);
        formatter.field("task_name", &"*** Sensitive Data Redacted ***");
        formatter.finish()
    }
}
impl ConsoleState {
    /// Creates a new builder-style object to manufacture
    /// [`ConsoleState`](crate::types::ConsoleState).
    pub fn builder() -> crate::types::builders::ConsoleStateBuilder {
        crate::types::builders::ConsoleStateBuilder::default()
    }
}

/// A builder for [`ConsoleState`](crate::types::ConsoleState).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default)]
#[non_exhaustive]
pub struct ConsoleStateBuilder {
    pub(crate) region: ::std::option::Option<::std::string::String>,
    pub(crate) console_url: ::std::option::Option<::std::string::String>,
    pub(crate) service_id: ::std::option::Option<::std::string::String>,
    pub(crate) service_console_page: ::std::option::Option<::std::string::String>,
    pub(crate) service_subconsole_page: ::std::option::Option<::std::string::String>,
    pub(crate) task_name: ::std::option::Option<::std::string::String>,
}
impl ConsoleStateBuilder {
    #[allow(missing_docs)] // documentation missing in model
    pub fn region(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.region = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_region(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.region = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_region(&self) -> &::std::option::Option<::std::string::String> {
        &self.region
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn console_url(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.console_url = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_console_url(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.console_url = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_console_url(&self) -> &::std::option::Option<::std::string::String> {
        &self.console_url
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn service_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.service_id = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_service_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.service_id = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_service_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.service_id
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn service_console_page(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.service_console_page = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_service_console_page(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.service_console_page = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_service_console_page(&self) -> &::std::option::Option<::std::string::String> {
        &self.service_console_page
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn service_subconsole_page(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.service_subconsole_page = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_service_subconsole_page(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.service_subconsole_page = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_service_subconsole_page(&self) -> &::std::option::Option<::std::string::String> {
        &self.service_subconsole_page
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn task_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.task_name = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_task_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.task_name = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_task_name(&self) -> &::std::option::Option<::std::string::String> {
        &self.task_name
    }

    /// Consumes the builder and constructs a [`ConsoleState`](crate::types::ConsoleState).
    pub fn build(self) -> crate::types::ConsoleState {
        crate::types::ConsoleState {
            region: self.region,
            console_url: self.console_url,
            service_id: self.service_id,
            service_console_page: self.service_console_page,
            service_subconsole_page: self.service_subconsole_page,
            task_name: self.task_name,
        }
    }
}
impl ::std::fmt::Debug for ConsoleStateBuilder {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut formatter = f.debug_struct("ConsoleStateBuilder");
        formatter.field("region", &self.region);
        formatter.field("console_url", &"*** Sensitive Data Redacted ***");
        formatter.field("service_id", &self.service_id);
        formatter.field("service_console_page", &self.service_console_page);
        formatter.field("service_subconsole_page", &self.service_subconsole_page);
        formatter.field("task_name", &"*** Sensitive Data Redacted ***");
        formatter.finish()
    }
}
