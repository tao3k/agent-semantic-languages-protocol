use crate::provider_command::support::{
    asp_command, make_executable, prepend_path, provider, temp_project_root, write_activation,
    write_marker_provider,
};
use serde_json::Value;

use super::assert_graph_turbo_request_contract;

#[test]
fn fzf_seeds_is_asp_owned_for_cheap_discovery() {
    let root = temp_project_root("search-fzf-fast-facade");
    let bin_dir = root.join(".bin");
    let marker = root.join("provider-called");
    let args_path = root.join("graph-turbo-args");
    let stdin_path = root.join("graph-turbo-stdin.json");
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(
        root.join("src/lib.rs"),
        "pub fn cache_root() {}\npub fn unrelated() {}\n",
    )
    .expect("write source");
    write_marker_provider(&bin_dir, "rs-harness", &marker);
    write_graph_turbo_ranker(&bin_dir);
    write_activation(&root, &[provider("rust", Vec::new())]);

    let output = asp_command(&root)
        .env("PATH", prepend_path(&bin_dir))
        .env("PRJ_CACHE_HOME", root.join(".cache"))
        .env("ASP_GRAPH_TURBO_ARGS_OUT", &args_path)
        .env("ASP_GRAPH_TURBO_STDIN_OUT", &stdin_path)
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
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout"),
        "[graph-frontier] external=fzf\n"
    );
    assert_eq!(
        std::fs::read_to_string(&args_path).expect("read args"),
        "rank\n-\n--format\ncompact\n"
    );
    let payload: Value = serde_json::from_slice(&std::fs::read(&stdin_path).expect("read stdin"))
        .expect("graph turbo stdin JSON");
    assert_graph_turbo_request_contract(&payload);
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
        "search fzf seeds should not spawn provider"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn fzf_scoped_root_outputs_workspace_relative_replayable_locators() {
    let root = temp_project_root("search-fzf-scoped-root-locators");
    let bin_dir = root.join(".bin");
    let marker = root.join("provider-called");
    let args_path = root.join("graph-turbo-args");
    let stdin_path = root.join("graph-turbo-stdin.json");
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
    write_graph_turbo_ranker(&bin_dir);
    write_activation(&root, &[provider("rust", Vec::new())]);

    let output = asp_command(&root)
        .env("PATH", prepend_path(&bin_dir))
        .env("PRJ_CACHE_HOME", root.join(".cache"))
        .env("ASP_GRAPH_TURBO_ARGS_OUT", &args_path)
        .env("ASP_GRAPH_TURBO_STDIN_OUT", &stdin_path)
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
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout"),
        "[graph-frontier] external=fzf\n"
    );
    let payload: Value = serde_json::from_slice(&std::fs::read(&stdin_path).expect("read stdin"))
        .expect("graph turbo stdin JSON");
    assert!(
        payload["graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .any(|node| node["kind"] == "owner" && node["value"] == "crates/demo/src/lib.rs")
    );
    assert!(
        payload["graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .any(|node| node["kind"] == "item" && node["locator"] == "crates/demo/src/lib.rs:1:1")
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

#[test]
fn typescript_fzf_can_emit_typed_hot_request_for_live_candidate_frontier() {
    let root = temp_project_root("typescript-search-fzf-graph-turbo-request");
    let bin_dir = root.join(".bin");
    let marker = root.join("provider-called");
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(
        root.join("src/index.ts"),
        "export function cacheRoot() {}\nexport function unrelated() {}\n",
    )
    .expect("write source");
    write_marker_provider(&bin_dir, "ts-harness", &marker);
    write_activation(&root, &[provider("typescript", Vec::new())]);

    let output = asp_command(&root)
        .env("PATH", prepend_path(&bin_dir))
        .env("PRJ_CACHE_HOME", root.join(".cache"))
        .args([
            "typescript",
            "search",
            "fzf",
            "cacheRoot",
            "owner",
            "tests",
            "--view",
            "graph-turbo-request",
            ".",
        ])
        .output()
        .expect("run asp typescript search fzf graph turbo request");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: Value = serde_json::from_slice(&output.stdout).expect("graph turbo request JSON");
    assert_graph_turbo_request_contract(&payload);
    assert_eq!(payload["profile"], "owner-query");
    assert_eq!(payload["seedIds"][0], "query:cacheroot");
    assert_graph_has_hot_code_path(&payload, "cacheroot");
    assert!(
        !marker.exists(),
        "typescript graph-turbo-request should not spawn provider"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn fzf_default_view_uses_builtin_ranker_for_live_candidate_frontier() {
    let root = temp_project_root("search-fzf-default-graph-turbo");
    let bin_dir = root.join(".bin");
    let marker = root.join("provider-called");
    let args_path = root.join("graph-turbo-args");
    let stdin_path = root.join("graph-turbo-stdin.json");
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(
        root.join("src/lib.rs"),
        "pub fn cache_root() {}\npub fn unrelated() {}\n",
    )
    .expect("write source");
    write_marker_provider(&bin_dir, "rs-harness", &marker);
    write_graph_turbo_ranker(&bin_dir);
    write_activation(&root, &[provider("rust", Vec::new())]);

    let output = asp_command(&root)
        .env("PATH", prepend_path(&bin_dir))
        .env("PRJ_CACHE_HOME", root.join(".cache"))
        .env("ASP_GRAPH_TURBO_ARGS_OUT", &args_path)
        .env("ASP_GRAPH_TURBO_STDIN_OUT", &stdin_path)
        .args(["rust", "search", "fzf", "cache_root", "owner", "tests", "."])
        .output()
        .expect("run asp rust search fzf default view");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout"),
        "[graph-frontier] external=fzf\n"
    );
    assert_eq!(
        std::fs::read_to_string(&args_path).expect("read args"),
        "rank\n-\n--format\ncompact\n"
    );
    let payload: Value = serde_json::from_slice(&std::fs::read(&stdin_path).expect("read stdin"))
        .expect("graph turbo stdin JSON");
    assert_graph_turbo_request_contract(&payload);
    assert_eq!(payload["profile"], "owner-query");
    assert_eq!(payload["seedIds"][0], "query:cache_root");
    assert_graph_has_hot_code_path(&payload, "cache_root");
    assert!(
        !marker.exists(),
        "search fzf default view should not spawn provider"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn typescript_fzf_default_view_uses_shared_graph_turbo_ranker() {
    let root = temp_project_root("typescript-search-fzf-default-graph-turbo");
    let bin_dir = root.join(".bin");
    let marker = root.join("provider-called");
    let args_path = root.join("graph-turbo-args");
    let stdin_path = root.join("graph-turbo-stdin.json");
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(
        root.join("src/index.ts"),
        "export function cacheRoot() {}\nexport function unrelated() {}\n",
    )
    .expect("write source");
    write_marker_provider(&bin_dir, "ts-harness", &marker);
    write_graph_turbo_ranker(&bin_dir);
    write_activation(&root, &[provider("typescript", Vec::new())]);

    let output = asp_command(&root)
        .env("PATH", prepend_path(&bin_dir))
        .env("PRJ_CACHE_HOME", root.join(".cache"))
        .env("ASP_GRAPH_TURBO_ARGS_OUT", &args_path)
        .env("ASP_GRAPH_TURBO_STDIN_OUT", &stdin_path)
        .args([
            "typescript",
            "search",
            "fzf",
            "cacheRoot",
            "owner",
            "tests",
            ".",
        ])
        .output()
        .expect("run asp typescript search fzf default view");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout"),
        "[graph-frontier] external=fzf\n"
    );
    assert_eq!(
        std::fs::read_to_string(&args_path).expect("read args"),
        "rank\n-\n--format\ncompact\n"
    );
    let payload: Value = serde_json::from_slice(&std::fs::read(&stdin_path).expect("read stdin"))
        .expect("graph turbo stdin JSON");
    assert_graph_turbo_request_contract(&payload);
    assert_eq!(payload["seedIds"][0], "query:cacheroot");
    assert_graph_has_hot_code_path(&payload, "cacheroot");
    assert!(
        payload["graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .any(|node| node["kind"] == "item" && node["value"] == "cacheroot")
    );
    assert!(
        !marker.exists(),
        "typescript search fzf default view should not spawn provider"
    );
    let _ = std::fs::remove_dir_all(root);
}

fn write_graph_turbo_ranker(bin_dir: &std::path::Path) {
    std::fs::create_dir_all(bin_dir).expect("create fake graph turbo bin dir");
    let graph_turbo = bin_dir.join("asp-graph-turbo");
    std::fs::write(
        &graph_turbo,
        "#!/bin/sh\n\
         printf '%s\n' \"$@\" > \"$ASP_GRAPH_TURBO_ARGS_OUT\"\n\
         cat > \"$ASP_GRAPH_TURBO_STDIN_OUT\"\n\
         printf '[graph-frontier] external=fzf\\n'\n",
    )
    .expect("write fake asp-graph-turbo");
    make_executable(&graph_turbo);
}

fn assert_graph_has_hot_code_path(payload: &Value, symbol: &str) {
    let nodes = payload["graph"]["nodes"].as_array().expect("nodes");
    let edges = payload["graph"]["edges"].as_array().expect("edges");
    let hot_id = nodes
        .iter()
        .find(|node| node["kind"] == "hot" && node["symbol"] == symbol)
        .and_then(|node| node["id"].as_str())
        .expect("hot node for symbol");
    let item_id = nodes
        .iter()
        .find(|node| node["kind"] == "item" && node["value"] == symbol)
        .and_then(|node| node["id"].as_str())
        .expect("item node for symbol");

    assert!(
        edges.iter().any(|edge| {
            edge["source"] == item_id && edge["target"] == hot_id && edge["relation"] == "contains"
        }),
        "expected item -> hot contains edge for {symbol}"
    );
}
