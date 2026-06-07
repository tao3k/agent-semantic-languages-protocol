use crate::provider_command::support::{
    asp_command, prepend_path, provider, temp_project_root, write_activation, write_marker_provider,
};
use serde_json::Value;

use super::assert_graph_turbo_request_contract;

#[test]
fn fzf_seeds_is_asp_owned_for_cheap_discovery() {
    let root = temp_project_root("search-fzf-fast-facade");
    let bin_dir = root.join(".bin");
    let marker = root.join("provider-called");
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(
        root.join("src/lib.rs"),
        "pub fn cache_root() {}\npub fn unrelated() {}\n",
    )
    .expect("write source");
    write_marker_provider(&bin_dir, "rs-harness", &marker);
    write_activation(&root, &[provider("rust", Vec::new())]);

    let output = asp_command(&root)
        .env("PATH", prepend_path(&bin_dir))
        .env("PRJ_CACHE_HOME", root.join(".cache"))
        .args([
            "rust",
            "search",
            "fzf",
            "cache_root",
            "owner",
            "tests",
            "--view",
            "seeds",
            ".",
        ])
        .output()
        .expect("run asp rust search fzf");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.starts_with("[search-ingest]"), "{stdout}");
    assert!(
        stdout.contains("S=symbol:symbol(cache_root)@src/lib.rs:1:1!symbol"),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            "entries=owner-items(O=>candidate-items+symbols),owner-tests(O=>covering-tests+test-entrypoints+fixtures)"
        ),
        "{stdout}"
    );
    assert!(
        !marker.exists(),
        "search fzf seeds should not spawn provider"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn fzf_scoped_root_outputs_workspace_relative_replayable_locators() {
    let root = temp_project_root("search-fzf-scoped-root-locators");
    let bin_dir = root.join(".bin");
    let marker = root.join("provider-called");
    std::fs::create_dir_all(root.join("crates/demo/src")).expect("create scoped src");
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/demo\"]\n",
    )
    .expect("write workspace manifest");
    std::fs::write(
        root.join("crates/demo/Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write demo manifest");
    std::fs::write(
        root.join("crates/demo/src/lib.rs"),
        "pub fn cache_root() {}\n",
    )
    .expect("write scoped source");
    write_marker_provider(&bin_dir, "rs-harness", &marker);
    write_activation(&root, &[provider("rust", Vec::new())]);

    let output = asp_command(&root)
        .env("PATH", prepend_path(&bin_dir))
        .env("PRJ_CACHE_HOME", root.join(".cache"))
        .args([
            "rust",
            "search",
            "fzf",
            "cache_root",
            "owner",
            "tests",
            "--view",
            "seeds",
            "crates/demo",
        ])
        .output()
        .expect("run scoped asp rust search fzf");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(
        stdout.contains("O=owner:path(crates/demo/src/lib.rs)!owner"),
        "{stdout}"
    );
    assert!(
        stdout.contains("S=symbol:symbol(cache_root)@crates/demo/src/lib.rs:1:1!symbol"),
        "{stdout}"
    );

    assert!(
        !marker.exists(),
        "scoped fast path should not spawn provider"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn fzf_can_emit_graph_turbo_request_for_live_candidate_frontier() {
    let root = temp_project_root("search-fzf-graph-turbo-request");
    let bin_dir = root.join(".bin");
    let marker = root.join("provider-called");
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(
        root.join("src/lib.rs"),
        "pub fn cache_root() {}\npub fn unrelated() {}\n",
    )
    .expect("write source");
    write_marker_provider(&bin_dir, "rs-harness", &marker);
    write_activation(&root, &[provider("rust", Vec::new())]);

    let output = asp_command(&root)
        .env("PATH", prepend_path(&bin_dir))
        .env("PRJ_CACHE_HOME", root.join(".cache"))
        .args([
            "rust",
            "search",
            "fzf",
            "cache_root",
            "owner",
            "tests",
            "--view",
            "graph-turbo-request",
            ".",
        ])
        .output()
        .expect("run asp rust search fzf graph turbo request");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: Value = serde_json::from_slice(&output.stdout).expect("graph turbo request JSON");
    assert_graph_turbo_request_contract(&payload);
    assert_eq!(payload["profile"], "owner-query");
    assert_eq!(payload["seedIds"][0], "query:cache_root");
    assert!(
        payload["graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .any(|node| node["kind"] == "item" && node["value"] == "cache_root")
    );
    assert!(
        !marker.exists(),
        "search fzf graph-turbo-request should not spawn provider"
    );
    let _ = std::fs::remove_dir_all(root);
}
