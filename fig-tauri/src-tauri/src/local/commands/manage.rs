use fig_proto::local::DebugModeCommand;

use crate::state::AppStateType;

use super::super::ResponseResult;

pub async fn debug(_state: &AppStateType, _command: DebugModeCommand) -> ResponseResult {
    todo!()
}
