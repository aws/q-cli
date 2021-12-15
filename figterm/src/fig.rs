use std::env;

pub struct FigInfo {
    pub term_session_id: Option<String>,
    pub fig_integration_version: Option<String>,
    pub pt_name: Option<String>,
}

impl FigInfo {
    pub fn new() -> FigInfo {
        let term_session_id = env::var("TERM_SESSION_ID").ok();
        let fig_integration_version = env::var("FIG_INTEGRATION_VERSION").ok();

        FigInfo {
            term_session_id,
            fig_integration_version,
            pt_name: None,
        }
    }
}
