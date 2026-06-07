use std::fs;
use std::path::{Path, PathBuf};

use super::{
    CLIENT_HOOK_CONFIG_SCHEMA_ID, default_hook_client_config_path,
    default_hook_client_config_template, load_hook_client_config_file,
};

#[test]
fn default_template_round_trips_through_config_parser() {
    let root = temp_root("hook-client-template");
    let config_path = default_hook_client_config_path(&root);
    fs::create_dir_all(config_path.parent().expect("config parent")).expect("config dir");
    fs::write(&config_path, default_hook_client_config_template()).expect("write config");

    let config = load_hook_client_config_file(&config_path).expect("load config");

    assert_eq!(
        config.schema_id.as_deref(),
        Some(CLIENT_HOOK_CONFIG_SCHEMA_ID)
    );
    assert_eq!(
        config
            .experimental
            .get("semanticAstPatch")
            .and_then(|feature| feature.get("enabled")),
        Some(&false)
    );
    assert!(config.rules.is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn missing_config_loads_empty_defaults() {
    let root = temp_root("hook-client-missing");
    let config = load_hook_client_config_file(&root.join("missing.toml")).expect("missing config");

    assert!(config.rules.is_empty());
    assert!(config.experimental.is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn invalid_route_kind_is_rejected_by_config_layer() {
    let root = temp_root("hook-client-invalid-route");
    let config_path = root.join("config.toml");
    fs::write(
        &config_path,
        r#"
schemaId = "agent.semantic-protocols.hook.client-config"
schemaVersion = "1"
protocolId = "agent.semantic-protocols.hook"
protocolVersion = "1"

[[rules]]
id = "deny-rust-read"
decision = "deny"

[[rules.routes]]
providerId = "rs-harness"
kind = "legacy-alias"
argv = ["asp", "rust"]
"#,
    )
    .expect("write config");

    let error = load_hook_client_config_file(&config_path).expect_err("invalid route kind");

    assert!(error.contains("legacy-alias"), "{error}");
    let _ = fs::remove_dir_all(root);
}

fn temp_root(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("agent-semantic-config-{label}-{nonce}"));
    fs::create_dir_all(&root).expect("create temp root");
    canonical(&root)
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}
