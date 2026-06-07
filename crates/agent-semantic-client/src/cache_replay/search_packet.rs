//! Search packet artifact replay through the local graph renderer.

use std::fs;
use std::path::Path;

use bytes::Bytes;

use super::graph_render::{run_graph_render_packet, run_graph_render_packet_bytes};
use super::limits::MAX_CACHE_REPLAY_ARTIFACT_BYTES;

pub(crate) fn search_output_artifact_replay_safe(stdout: &[u8]) -> bool {
    let Ok(stdout) = std::str::from_utf8(stdout) else {
        return false;
    };
    stdout.contains("[search-")
        && stdout.contains("legend: ID=kind:role(value)!next;")
        && stdout.contains("frontier ID.next")
        && stdout.contains("aliases: graph:{")
        && !stdout.contains('\0')
}

pub(crate) fn render_search_packet_bytes(packet_bytes: Bytes) -> Option<Bytes> {
    if packet_bytes.is_empty() || packet_bytes.len() as u64 > MAX_CACHE_REPLAY_ARTIFACT_BYTES {
        return None;
    }
    let output = run_graph_render_packet_bytes(packet_bytes, MAX_CACHE_REPLAY_ARTIFACT_BYTES)?;
    if !search_output_artifact_replay_safe(&output) {
        return None;
    }
    Some(output)
}

pub(crate) fn render_search_packet_artifact_stdout(artifact_path: &Path) -> Option<Bytes> {
    let metadata = fs::metadata(artifact_path).ok()?;
    if !metadata.is_file() || metadata.len() > MAX_CACHE_REPLAY_ARTIFACT_BYTES {
        return None;
    }
    run_graph_render_packet(artifact_path, MAX_CACHE_REPLAY_ARTIFACT_BYTES)
}
