//! Protocal buffer definitions

pub mod figterm;
pub mod hooks;
pub mod local;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_fig_pbuf() {
        let hook = hooks::new_edit_buffer_hook(None, "test".into(), 0, 0);
        let message = hooks::hook_to_message(hook);
        assert!(message.to_fig_pbuf().unwrap().starts_with(b"\x1b@fig-pbuf"))
    }
}
