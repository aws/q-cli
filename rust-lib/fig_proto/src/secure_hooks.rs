use crate::fig::ShellContext;
use crate::proto::local::{
    EditBufferHook,
    InterceptedKeyHook,
    PreExecHook,
    PromptHook,
    TerminalCursorCoordinates,
};
use crate::proto::secure::{
    hostbound,
    Hostbound,
};

fn hook_enum_to_hook(hook: hostbound::hook::Hook) -> hostbound::Hook {
    hostbound::Hook { hook: Some(hook) }
}

pub fn hook_to_message(hook: hostbound::Hook) -> Hostbound {
    Hostbound {
        packet: Some(hostbound::Packet::Hook(hook)),
    }
}

/// Construct a edit buffer hook
pub fn new_edit_buffer_hook(
    context: impl Into<Option<ShellContext>>,
    text: impl Into<String>,
    cursor: i64,
    histno: i64,
    coords: impl Into<Option<TerminalCursorCoordinates>>,
) -> hostbound::Hook {
    hook_enum_to_hook(hostbound::hook::Hook::EditBuffer(EditBufferHook {
        context: context.into(),
        terminal_cursor_coordinates: coords.into(),
        text: text.into(),
        cursor,
        histno,
    }))
}

/// Construct a new prompt hook
pub fn new_prompt_hook(context: impl Into<Option<ShellContext>>) -> hostbound::Hook {
    hook_enum_to_hook(hostbound::hook::Hook::Prompt(PromptHook {
        context: context.into(),
    }))
}

pub fn new_preexec_hook(context: impl Into<Option<ShellContext>>) -> hostbound::Hook {
    hook_enum_to_hook(hostbound::hook::Hook::PreExec(PreExecHook {
        context: context.into(),
        command: None,
    }))
}

pub fn new_intercepted_key_hook(
    context: impl Into<Option<ShellContext>>,
    action: impl Into<String>,
    key: impl Into<String>,
) -> hostbound::Hook {
    hook_enum_to_hook(hostbound::hook::Hook::InterceptedKey(InterceptedKeyHook {
        context: context.into(),
        action: action.into(),
        key: key.into(),
    }))
}
