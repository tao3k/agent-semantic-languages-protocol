use crate::provider_command::support::{
    asp_command, make_executable, prepend_path, temp_project_root,
};

#[test]
fn md_facade_uses_native_orgize_dependency() {
    let root = temp_project_root("md-document-facade");
    let bin_dir = root.join(".bin");
    std::fs::create_dir_all(&bin_dir).expect("create bin dir");
    let orgize = bin_dir.join("orgize");
    std::fs::write(&orgize, "#!/bin/sh\nexit 42\n").expect("write orgize");
    make_executable(&orgize);

    let output = asp_command(&root)
        .env("PATH", prepend_path(&bin_dir))
        .args(["md", "search", "prime", "--view", "seeds", "."])
        .output()
        .expect("run asp md search");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("[search-prime] lang=md"), "stdout={stdout}");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn md_facade_search_fzf_toc_returns_toc_for_keyword_matched_documents() {
    let root = temp_project_root("md-document-fzf-toc");
    std::fs::write(
        root.join("guide.md"),
        "# Guide\n\nTree facts live here.\n\n## Syntax\n\nSitter details live in this section.\n",
    )
    .expect("write guide markdown");
    std::fs::write(
        root.join("other.md"),
        "# Other\n\nThis document should not match both query terms.\n",
    )
    .expect("write other markdown");

    let output = asp_command(&root)
        .args([
            "md", "search", "fzf", "Tree", "Sitter", "--view", "toc", ".",
        ])
        .output()
        .expect("run asp md fzf toc");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("[search-fzf-toc] lang=md"), "{stdout}");
    assert!(stdout.contains("q=Tree Sitter"), "{stdout}");
    assert!(
        stdout.contains("|doc path=\"./guide.md\" heading=2"),
        "{stdout}"
    );
    assert!(stdout.contains("level=1 title=\"Guide\""), "{stdout}");
    assert!(stdout.contains("level=2 title=\"Syntax\""), "{stdout}");
    assert!(!stdout.contains("./other.md"), "{stdout}");

    let _ = std::fs::remove_dir_all(root);
}
