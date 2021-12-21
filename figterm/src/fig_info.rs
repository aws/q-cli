//! 🦀

use std::env;

#[derive(Debug, Clone)]
pub struct FigInfo {
    pub term_session_id: Option<String>,
    pub fig_integration_version: Option<i32>,
    pub pt_name: Option<String>,
}

impl FigInfo {
    pub fn new() -> FigInfo {
        let term_session_id = env::var("TERM_SESSION_ID").ok();
        let fig_integration_version = env::var("FIG_INTEGRATION_VERSION")
            .ok()
            .and_then(|f| f.parse().ok());

        FigInfo {
            term_session_id,
            fig_integration_version,
            pt_name: None,
        }
    }
}

impl Default for FigInfo {
    fn default() -> Self {
        Self::new()
    }
}
