use crate::provider_command::support::{
    asp_command, prepend_path, provider, temp_project_root, write_activation, write_marker_provider,
};

#[test]
fn search_pipe_is_asp_owned_and_renders_generated_candidates_without_provider_spawn() {
    let root = temp_project_root("search-pipe-facade");
    let bin_dir = root.join(".bin");
    let marker = root.join("provider-called");
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(
        root.join("src/lib.rs"),
        "pub struct HookDecision;\npub struct ClientReceipt;\nfn unrelated() {}\n",
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
            "pipe",
            "HookDecision ClientReceipt",
            "--pipe",
            "items,tests",
            "--view",
            "seeds",
            ".",
        ])
        .output()
        .expect("run asp rust search pipe");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.starts_with("[search-ingest]"), "{stdout}");
    assert!(
        stdout.contains("O=owner:path(src/lib.rs)!owner"),
        "{stdout}"
    );
    assert!(
        stdout.contains("S=symbol:symbol(hookdecision)@src/lib.rs:1:1!symbol"),
        "{stdout}"
    );
    assert!(
        stdout.contains("S2=symbol:symbol(clientreceipt)@src/lib.rs:2:2!symbol"),
        "{stdout}"
    );
    assert!(!marker.exists(), "search pipe should not spawn provider");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn search_pipe_commands_view_points_to_search_suggest() {
    let root = temp_project_root("search-pipe-commands-facade");
    let bin_dir = root.join(".bin");
    let marker = root.join("provider-called");
    write_marker_provider(&bin_dir, "rs-harness", &marker);
    write_activation(&root, &[provider("rust", Vec::new())]);

    let output = asp_command(&root)
        .env("PATH", prepend_path(&bin_dir))
        .env("PRJ_CACHE_HOME", root.join(".cache"))
        .args([
            "rust",
            "search",
            "pipe",
            "HookDecision",
            "--pipe",
            "items,tests",
            "--view",
            "commands",
            ".",
        ])
        .output()
        .expect("run asp rust search pipe commands");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr");
    assert!(
        stderr.contains("search pipe --view commands moved to search suggest --view commands"),
        "{stderr}"
    );
    assert!(
        !marker.exists(),
        "commands migration error should not spawn provider"
    );
    let _ = std::fs::remove_dir_all(root);
}
