use crate::provider_command::support::{
    asp_command, prepend_path, provider, temp_project_root, write_activation, write_marker_provider,
};

#[test]
fn reasoning_owner_query_is_asp_owned_and_does_not_spawn_provider() {
    let root = temp_project_root("search-reasoning-owner-query-facade");
    let bin_dir = root.join(".bin");
    let marker = root.join("provider-called");
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(
        root.join("src/lib.rs"),
        "fn unrelated() {}\nfn render_fast_prime_search() {}\n",
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
            "reasoning",
            "owner-query",
            "--owner",
            "src/lib.rs",
            "--query",
            "render_fast_prime_search",
            "--view",
            "seeds",
            ".",
        ])
        .output()
        .expect("run asp rust search reasoning owner-query");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.starts_with("[search-reasoning]"), "{stdout}");
    assert!(
        stdout.contains("I=item:symbol(render_fast_prime_search)@src/lib.rs:2:3!code"),
        "{stdout}"
    );
    assert!(
        stdout.contains("entries=owner-query(O,Q=>items+tests+dependency-usage)\n"),
        "{stdout}"
    );
    assert!(
        !stdout.contains("owner-tests("),
        "owner-query reasoning should not infer owner-tests entry: {stdout}"
    );
    assert!(
        !marker.exists(),
        "owner-query fast path should not spawn provider"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn scoped_owner_query_code_locator_replays_from_workspace_root() {
    let root = temp_project_root("search-reasoning-scoped-root-code-replay");
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
        "fn cache_root() {\n    let value = 1;\n    let _ = value;\n}\n",
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
            "reasoning",
            "owner-query",
            "--owner",
            "src/lib.rs",
            "--query",
            "cache_root",
            "--view",
            "seeds",
            "crates/demo",
        ])
        .output()
        .expect("run scoped asp rust search reasoning owner-query");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(
        stdout.contains("I=item:symbol(cache_root)@crates/demo/src/lib.rs:1:4!code"),
        "{stdout}"
    );
    assert!(
        !marker.exists(),
        "scoped owner-query fast path should not spawn provider"
    );

    let replay = asp_command(&root)
        .env("PATH", prepend_path(&bin_dir))
        .env("PRJ_CACHE_HOME", root.join(".cache"))
        .args([
            "rust",
            "query",
            "--selector",
            "crates/demo/src/lib.rs:1:4",
            "--code",
            ".",
        ])
        .output()
        .expect("replay scoped code locator");

    assert!(
        replay.status.success(),
        "status={:?} stdout={} stderr={}",
        replay.status.code(),
        String::from_utf8_lossy(&replay.stdout),
        String::from_utf8_lossy(&replay.stderr)
    );
    let replay_stdout = String::from_utf8(replay.stdout).expect("replay stdout");
    assert!(
        replay_stdout.contains("fn cache_root() {"),
        "{replay_stdout}"
    );
    assert!(replay_stdout.contains("let value = 1;"), "{replay_stdout}");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn owner_tests_and_owner_items_query_are_asp_owned() {
    let root = temp_project_root("search-owner-fast-facade");
    let bin_dir = root.join(".bin");
    let marker = root.join("provider-called");
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(
        root.join("src/lib.rs"),
        "fn unrelated() {}\nfn render_fast_prime_search() {}\n",
    )
    .expect("write source");
    write_marker_provider(&bin_dir, "rs-harness", &marker);
    write_activation(&root, &[provider("rust", Vec::new())]);

    let owner_tests = asp_command(&root)
        .env("PATH", prepend_path(&bin_dir))
        .env("PRJ_CACHE_HOME", root.join(".cache"))
        .args([
            "rust",
            "search",
            "reasoning",
            "owner-tests",
            "--owner",
            "src/lib.rs",
            "--view",
            "seeds",
            ".",
        ])
        .output()
        .expect("run asp rust search reasoning owner-tests");
    assert!(
        owner_tests.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&owner_tests.stderr)
    );
    let owner_tests_stdout = String::from_utf8(owner_tests.stdout).expect("stdout");
    assert!(
        owner_tests_stdout.contains("T=test:path(src/lib.rs)!tests"),
        "{owner_tests_stdout}"
    );

    let owner_items = asp_command(&root)
        .env("PATH", prepend_path(&bin_dir))
        .env("PRJ_CACHE_HOME", root.join(".cache"))
        .args([
            "rust",
            "search",
            "owner",
            "src/lib.rs",
            "items",
            "--query",
            "render_fast_prime_search",
            "--view",
            "seeds",
            ".",
        ])
        .output()
        .expect("run asp rust search owner items");
    assert!(
        owner_items.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&owner_items.stderr)
    );
    let owner_items_stdout = String::from_utf8(owner_items.stdout).expect("stdout");
    assert!(
        owner_items_stdout.contains("I=item:symbol(render_fast_prime_search)@src/lib.rs:2:3!code"),
        "{owner_items_stdout}"
    );
    assert!(
        !marker.exists(),
        "owner fast paths should not spawn provider"
    );
    let _ = std::fs::remove_dir_all(root);
}
