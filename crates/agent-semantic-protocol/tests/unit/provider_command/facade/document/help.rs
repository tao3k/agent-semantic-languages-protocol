use crate::provider_command::support::{asp_command, temp_project_root};

#[test]
fn top_level_usage_lists_document_facades() {
    let root = temp_project_root("document-facade-top-level-help");

    let output = asp_command(&root)
        .arg("--help")
        .output()
        .expect("run top-level help");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("stderr");
    assert!(
        stderr.contains("rust|typescript|python|julia|org|md"),
        "stderr={stderr}"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn document_facade_help_does_not_spawn_orgize() {
    let root = temp_project_root("document-facade-help");

    for language in ["org", "md"] {
        let output = asp_command(&root)
            .env("PATH", "")
            .args([language, "--help"])
            .output()
            .expect("run document help");

        assert!(
            output.status.success(),
            "language={language} stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).expect("stdout");
        assert!(
            stdout.contains(&format!("usage: asp {language} <guide|search|query> ...")),
            "stdout={stdout}"
        );
    }
    let _ = std::fs::remove_dir_all(root);
}
