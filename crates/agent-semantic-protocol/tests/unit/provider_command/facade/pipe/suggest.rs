use crate::provider_command::support::{
    asp_command, prepend_path, provider, temp_project_root, write_activation, write_marker_provider,
};

#[test]
fn search_suggest_is_advisory_and_does_not_spawn_provider() {
    let root = temp_project_root("search-suggest-facade");
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
            "suggest",
            "HookDecision ClientReceipt",
            "--view",
            "commands",
            ".",
        ])
        .output()
        .expect("run asp rust search suggest");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.starts_with("[search-suggest]"), "{stdout}");
    assert!(
        stdout.contains("|contract executes=false provider=false planner=false"),
        "{stdout}"
    );
    assert!(stdout.contains("asp search history audit ."), "{stdout}");
    assert!(
        stdout.contains(
            "asp rust search pipe 'HookDecision ClientReceipt' --pipe items,tests --view seeds ."
        ),
        "{stdout}"
    );
    assert!(
        stdout.contains("asp rust search reasoning owner-query --owner <path>"),
        "{stdout}"
    );
    assert!(!marker.exists(), "search suggest should not spawn provider");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn unsupported_pipeline_alias_points_to_search_pipe_without_provider_spawn() {
    let root = temp_project_root("unsupported-pipeline-alias-facade");
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
            "compose",
            "--query",
            "HookDecision|ClientReceipt",
            "--owners",
            "src",
            "--pipe",
            "items,tests",
            "--view",
            "seeds",
            ".",
        ])
        .output()
        .expect("run asp rust search compose");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr");
    assert!(stderr.contains("unknown search view: compose"), "{stderr}");
    assert!(
        stderr.contains("use `search pipe` for ASP-owned candidate pipelines"),
        "{stderr}"
    );
    assert!(
        !marker.exists(),
        "search compose rejection should not spawn provider"
    );
    let _ = std::fs::remove_dir_all(root);
}
