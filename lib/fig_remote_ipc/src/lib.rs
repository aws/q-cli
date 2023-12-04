use std::sync::Arc;

use fig_proto::local::{
    EditBufferHook,
    InterceptedKeyHook,
    PostExecHook,
    PreExecHook,
    PromptHook,
    ShellContext,
};
use fig_proto::remote::clientbound;
use fig_proto::remote::hostbound::ConfirmExchangeCredentialsRequest;
use figterm::{
    FigtermSessionId,
    FigtermState,
};
use tokio::time::Instant;

pub mod figterm;
pub mod remote;

pub type AuthCode = Option<(u32, Instant)>;

#[async_trait::async_trait]
pub trait RemoteHookHandler {
    async fn edit_buffer(
        &mut self,
        edit_buffer_hook: &EditBufferHook,
        session_id: &FigtermSessionId,
        figterm_state: &Arc<FigtermState>,
    ) -> anyhow::Result<Option<clientbound::response::Response>>;

    async fn prompt(
        &mut self,
        prompt_hook: &PromptHook,
        session_id: &FigtermSessionId,
        figterm_state: &Arc<FigtermState>,
    ) -> anyhow::Result<Option<clientbound::response::Response>>;

    async fn pre_exec(
        &mut self,
        pre_exec_hook: &PreExecHook,
        session_id: &FigtermSessionId,
        figterm_state: &Arc<FigtermState>,
    ) -> anyhow::Result<Option<clientbound::response::Response>>;

    async fn post_exec(
        &mut self,
        post_exec_hook: &PostExecHook,
        session_id: &FigtermSessionId,
        figterm_state: &Arc<FigtermState>,
    ) -> anyhow::Result<Option<clientbound::response::Response>>;

    async fn intercepted_key(
        &mut self,
        intercepted_key: InterceptedKeyHook,
    ) -> anyhow::Result<Option<clientbound::response::Response>>;

    async fn account_info(&mut self) -> anyhow::Result<Option<clientbound::response::Response>>;

    async fn start_exchange_credentials(
        &mut self,
        last_auth_code: &mut AuthCode,
    ) -> anyhow::Result<Option<clientbound::response::Response>>;

    async fn confirm_exchange_credentials(
        &mut self,
        request: ConfirmExchangeCredentialsRequest,
        last_auth_code: &mut AuthCode,
    ) -> anyhow::Result<Option<clientbound::response::Response>>;

    /// This is not technically a hook, it is triggers by many other hooks and does not allow for a
    /// response, mostly used for diagnostics and testing
    async fn shell_context(&mut self, _context: &ShellContext, _session_id: &FigtermSessionId) {}
}
