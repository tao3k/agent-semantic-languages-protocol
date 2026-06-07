#![deny(dead_code)]

//! Runtime state materialization for ASP project-local storage.

mod state;

pub use state::{
    ProjectRuntimeState, ensure_project_artifacts_dir, ensure_project_client_cache_dir,
    ensure_project_hook_cache_dir, ensure_project_hook_state_dir, ensure_project_runtime_home,
    project_runtime_state,
};
