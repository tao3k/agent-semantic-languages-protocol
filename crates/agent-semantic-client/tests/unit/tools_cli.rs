use std::path::{Path, PathBuf};

use crate::tools_cli::{run_tools, run_wrap_with_path};

#[test]
fn tools_cli_rejects_unknown_subcommand() {
    let error = run_tools(Path::new("."), &["status".to_string()])
        .expect_err("unknown tools subcommand should fail");

    assert_eq!(
        error,
        "usage: asp tools <doctor [PROJECT_ROOT]|wrap asp-graph-turbo [--] [ARGS...]>"
    );
}

#[test]
fn wrap_asp_graph_turbo_uses_native_allowlisted_command() {
    let root = temp_dir("wrap-graph-turbo");
    let bin_dir = root.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("create bin dir");
    let wrapped_args = root.join("wrapped-args.txt");
    let graph_turbo = bin_dir.join("asp-graph-turbo");
    std::fs::write(
        &graph_turbo,
        "#!/bin/sh\n\
         printf '%s\n' \"$@\" > \"$1\"\n",
    )
    .expect("write fake asp-graph-turbo");
    make_executable(&graph_turbo);
    let path = std::env::join_paths([bin_dir]).expect("join PATH");

    run_wrap_with_path(
        &[
            "asp-graph-turbo".to_string(),
            "--".to_string(),
            wrapped_args.display().to_string(),
            "rank".to_string(),
        ],
        Some(path),
    )
    .expect("wrap graph turbo");

    let observed = std::fs::read_to_string(&wrapped_args).expect("read wrapped args");
    assert_eq!(observed, format!("{}\nrank\n", wrapped_args.display()));
    let _ = std::fs::remove_dir_all(root);
}

fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("asp-tools-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("create temp dir");
    path
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .expect("script metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("chmod script");
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}
