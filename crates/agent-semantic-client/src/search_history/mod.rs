//! Search command history audit via the graph-turbo artifact timeline.

mod artifact_events;
mod history_audit;
#[path = "../search_history_paths.rs"]
mod search_history_paths;

pub(crate) use history_audit::run_search_history;
