use fig_proto::fig::aggregate_session_metric_action_request::{
    Action,
    Increment,
};
use fig_proto::fig::AggregateSessionMetricActionRequest;

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::figterm::FigtermState;

pub fn handle_aggregate_session_metric_action_request(
    request: AggregateSessionMetricActionRequest,
    state: &FigtermState,
) -> RequestResult {
    if let Some(result) = state.with_most_recent(|session| {
        if let Some(ref mut metrics) = session.current_session_metrics {
            if let Some(action) = request.action {
                match action {
                    Action::Increment(Increment { field, amount }) => {
                        if field == "num_popups" {
                            metrics.num_popups += amount.unwrap_or(1);
                        } else {
                            return Err(format!("Unknown field: {field}"));
                        }
                    },
                };
            }
        }
        Ok(())
    }) {
        result?;
    }

    RequestResult::success()
}
