//! Replay-time graph rendering through the shared provider transport.

use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};

use agent_semantic_provider_transport::{
    OutputMode, ProviderProcessLimits, ProviderProcessSpec, StdinMode,
    run_provider_process as run_transport_process,
};
use bytes::Bytes;

const SEMANTIC_AGENT_PROTOCOL_BIN_ENV: &str = "SEMANTIC_AGENT_PROTOCOL_BIN";

pub(crate) fn run_graph_render_packet(packet_path: &Path, max_stdout_bytes: u64) -> Option<Bytes> {
    run_graph_render_process(
        packet_path.display().to_string(),
        StdinMode::Closed,
        max_stdout_bytes,
    )
}

pub(crate) fn run_graph_render_packet_bytes(
    packet_bytes: impl Into<Bytes>,
    max_stdout_bytes: u64,
) -> Option<Bytes> {
    run_graph_render_process(
        "-".to_string(),
        StdinMode::bytes(packet_bytes.into()),
        max_stdout_bytes,
    )
}

fn run_graph_render_process(
    packet_arg: String,
    stdin: StdinMode,
    max_stdout_bytes: u64,
) -> Option<Bytes> {
    let output = run_transport_process(ProviderProcessSpec {
        program: protocol_graph_renderer_binary().display().to_string(),
        args: vec![
            "graph".to_string(),
            "render".to_string(),
            "--packet".to_string(),
            packet_arg,
            "--view".to_string(),
            "seeds".to_string(),
        ],
        cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        env: BTreeMap::new(),
        stdin,
        stdout: OutputMode::Capture,
        stderr: OutputMode::Capture,
        limits: ProviderProcessLimits {
            max_stdout_bytes: Some(max_stdout_bytes as usize + 1),
            max_stderr_bytes: Some(64 * 1024),
            timeout: None,
        },
    })
    .ok()?;
    if !output.status.success()
        || output.stdout.is_empty()
        || output.receipt.stdout_bytes as u64 > max_stdout_bytes
    {
        return None;
    }
    Some(output.stdout)
}

fn protocol_graph_renderer_binary() -> PathBuf {
    env::var_os(SEMANTIC_AGENT_PROTOCOL_BIN_ENV)
        .map(PathBuf::from)
        .or_else(|| env::current_exe().ok())
        .unwrap_or_else(|| PathBuf::from("asp"))
}
