use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use semantic_agent_hook::{
    classify_hook, parse_profiles, DecisionKind, ProfileRegistry, ReasonKind,
};
use serde_json::json;

fn generated_rust_profile_path() -> &'static str {
    env!("SEMANTIC_AGENT_HOOK_RUST_PROFILE_REGISTRY")
}

fn temp_project_root(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("semantic-agent-hook-{name}-{unique}"));
    std::fs::create_dir_all(&root).expect("create temp project root");
    root
}

fn rust_harness_profile_registry() -> ProfileRegistry {
    let contents = std::fs::read_to_string(generated_rust_profile_path())
        .expect("generated rust profile registry");
    parse_profiles(&contents).expect("valid generated rust profile registry")
}

#[test]
fn build_script_uses_rust_harness_source_roots() {
    assert_eq!(env!("SEMANTIC_AGENT_HOOK_RUST_SOURCE_ROOTS"), "src");
}

#[test]
fn generated_rust_harness_profile_uses_provider_identity() {
    let registry = rust_harness_profile_registry();
    assert_eq!(registry.profiles.len(), 1);
    let profile = &registry.profiles[0];
    assert_eq!(profile.language_id, "rust");
    assert_eq!(profile.provider_id, "rs-harness");
    assert_eq!(profile.binary, "rs-harness");
    assert!(profile.source_roots.iter().any(|root| root == "src"));
    assert!(profile
        .source_extensions
        .iter()
        .any(|extension| extension == ".rs"));
}

#[test]
fn rust_harness_profile_routes_direct_reads_to_owner_search() {
    let decision = classify_hook(
        &rust_harness_profile_registry(),
        "codex",
        "pre-tool",
        &json!({
            "tool_name": "Read",
            "tool_input": {"path": "src/lib.rs"}
        }),
    );

    assert_eq!(decision.decision, DecisionKind::Deny);
    assert_eq!(decision.reason_kind, ReasonKind::DirectSourceRead);
    assert_eq!(
        decision.routes[0].argv,
        [
            "rs-harness",
            "search",
            "owner",
            "src/lib.rs",
            "items",
            "--view",
            "seeds",
            "."
        ]
    );
}

#[test]
fn rust_harness_profile_routes_raw_root_search_to_ingest() {
    let decision = classify_hook(
        &rust_harness_profile_registry(),
        "codex",
        "pre-tool",
        &json!({
            "tool_name": "functions.exec_command",
            "tool_input": {"cmd": "rg -n \"HookDecision\" ."}
        }),
    );

    assert_eq!(decision.decision, DecisionKind::Deny);
    assert_eq!(decision.reason_kind, ReasonKind::RawBroadSearch);
    assert_eq!(decision.routes[0].kind, "ingest");
    assert_eq!(
        decision.routes[0].stdin_mode.as_deref(),
        Some("pipe-candidates")
    );
}

#[test]
fn cli_doctor_accepts_generated_rust_profile_registry() {
    let output = Command::new(env!("CARGO_BIN_EXE_semantic-agent-hook"))
        .args(["doctor", "--profiles", generated_rust_profile_path()])
        .output()
        .expect("run semantic-agent-hook doctor");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("doctor stdout");
    assert!(stdout.contains("semantic-agent-hook profiles=1 projectRoot=."));
}

#[test]
fn cli_hook_emits_decision_for_generated_rust_profile_registry() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_semantic-agent-hook"))
        .args([
            "hook",
            "--client",
            "codex",
            "pre-tool",
            "--profiles",
            generated_rust_profile_path(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("run semantic-agent-hook hook");
    child
        .stdin
        .as_mut()
        .expect("hook stdin")
        .write_all(br#"{"tool_name":"Read","tool_input":{"path":"src/lib.rs"}}"#)
        .expect("write hook payload");

    let output = child.wait_with_output().expect("wait for hook output");

    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("hook JSON");
    assert_eq!(value["hookSpecificOutput"]["permissionDecision"], "deny");
    assert_eq!(value["agentHookDecision"]["decision"], "deny");
    assert_eq!(
        value["agentHookDecision"]["reasonKind"],
        "direct-source-read"
    );
    assert_eq!(
        value["agentHookDecision"]["routes"][0]["binary"],
        "rs-harness"
    );
    assert_eq!(
        value["agentHookDecision"]["routes"][0]["argv"][3],
        "src/lib.rs"
    );
}

#[test]
fn cli_hook_can_emit_raw_decision_for_schema_tests() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_semantic-agent-hook"))
        .args([
            "hook",
            "--client",
            "codex",
            "pre-tool",
            "--profiles",
            generated_rust_profile_path(),
            "--emit",
            "decision",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("run semantic-agent-hook hook");
    child
        .stdin
        .as_mut()
        .expect("hook stdin")
        .write_all(br#"{"tool_name":"Read","tool_input":{"path":"src/lib.rs"}}"#)
        .expect("write hook payload");

    let output = child.wait_with_output().expect("wait for hook output");

    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("hook JSON");
    assert_eq!(value["decision"], "deny");
    assert_eq!(value["reasonKind"], "direct-source-read");
}

#[test]
fn cli_install_writes_root_owned_codex_hook_config() {
    let root = temp_project_root("install");
    let output = Command::new(env!("CARGO_BIN_EXE_semantic-agent-hook"))
        .args([
            "install",
            "--client",
            "codex",
            "--profiles",
            ".codex/agent-hook-profiles.json",
            root.to_str().expect("utf8 temp root"),
        ])
        .output()
        .expect("run semantic-agent-hook install");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("install stdout");
    assert!(stdout.contains("[agent-install] client=codex"));
    assert!(stdout.contains("profiles=.codex/agent-hook-profiles.json"));
    let config =
        std::fs::read_to_string(root.join(".codex/config.toml")).expect("installed config");
    assert!(config.contains("# BEGIN semantic-agent-hook"));
    assert!(config.contains("semantic-agent-hook hook --client codex pre-tool"));
    assert!(config.contains("--profiles \"$repo_root/.codex/agent-hook-profiles.json\""));
    assert!(!config.contains("ts-harness agent hook --client codex"));
    assert!(!config.contains("rs-harness agent hook --client codex"));
    std::fs::remove_dir_all(root).expect("cleanup temp project root");
}
