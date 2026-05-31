use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;

use semantic_agent_hook::{classify_hook, parse_payload, parse_profiles, render_platform_response};

fn main() {
    if let Err(message) = run() {
        eprintln!("{message}");
        process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("hook") => run_hook(&args[1..]),
        Some("doctor") => run_doctor(&args[1..]),
        Some("install") => run_install(&args[1..]),
        _ => Err("usage: semantic-agent-hook hook --client <client> <event> --profiles <path> [--emit platform|decision]".to_string()),
    }
}

fn run_hook(args: &[String]) -> Result<(), String> {
    let client = flag_value(args, "--client")
        .ok_or_else(|| "missing required --client <client>".to_string())?;
    let profiles_path = flag_value(args, "--profiles")
        .ok_or_else(|| "missing required --profiles <path>".to_string())?;
    let emit = flag_value(args, "--emit").unwrap_or("platform");
    let event = first_positional(args).ok_or_else(|| "missing hook event".to_string())?;
    let registry = load_profiles(profiles_path)?;
    let mut stdin = String::new();
    io::stdin()
        .read_to_string(&mut stdin)
        .map_err(|error| format!("failed to read hook payload from stdin: {error}"))?;
    let payload =
        parse_payload(&stdin).map_err(|error| format!("invalid hook payload JSON: {error:?}"))?;
    let decision = classify_hook(&registry, client, event, &payload);
    let output_value = match emit {
        "decision" => serde_json::to_value(&decision)
            .map_err(|error| format!("failed to serialize hook decision: {error}"))?,
        "platform" => render_platform_response(&decision)
            .map_err(|error| format!("failed to render hook response: {error:?}"))?,
        other => {
            return Err(format!(
                "unsupported --emit value: {other}; expected platform or decision"
            ));
        }
    };
    let output = serde_json::to_string_pretty(&output_value)
        .map_err(|error| format!("failed to serialize hook response: {error}"))?;
    println!("{output}");
    Ok(())
}

fn run_doctor(args: &[String]) -> Result<(), String> {
    let profiles_path = flag_value(args, "--profiles")
        .ok_or_else(|| "missing required --profiles <path>".to_string())?;
    let registry = load_profiles(profiles_path)?;
    println!(
        "semantic-agent-hook profiles={} projectRoot={}",
        registry.profiles.len(),
        registry.project_root
    );
    Ok(())
}

fn run_install(args: &[String]) -> Result<(), String> {
    let client = flag_value(args, "--client")
        .ok_or_else(|| "missing required --client <client>".to_string())?;
    if client != "codex" {
        return Err(format!("unsupported install client: {client}"));
    }
    let profiles_path = flag_value(args, "--profiles").unwrap_or(".codex/agent-hook-profiles.json");
    let project_root = first_positional(args).unwrap_or(".");
    let project_root = PathBuf::from(project_root);
    let codex_dir = project_root.join(".codex");
    fs::create_dir_all(&codex_dir)
        .map_err(|error| format!("failed to create {}: {error}", codex_dir.display()))?;
    let config_path = codex_dir.join("config.toml");
    let existing = fs::read_to_string(&config_path).unwrap_or_default();
    let block = codex_hook_config(profiles_path);
    let content = merge_managed_block(
        &existing,
        &block,
        "# BEGIN semantic-agent-hook",
        "# END semantic-agent-hook",
    );
    let mode = if existing.is_empty() {
        "created"
    } else if existing == content {
        "present"
    } else {
        "updated"
    };
    fs::write(&config_path, content)
        .map_err(|error| format!("failed to write {}: {error}", config_path.display()))?;
    println!(
        "[agent-install] client=codex path={} profiles={} mode={mode}",
        relative_display(&project_root, &config_path),
        profiles_path
    );
    Ok(())
}

fn load_profiles(path: &str) -> Result<semantic_agent_hook::ProfileRegistry, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("failed to read profile registry {path}: {error}"))?;
    parse_profiles(&contents).map_err(|error| format!("invalid profile registry JSON: {error:?}"))
}

fn merge_managed_block(existing: &str, block: &str, begin: &str, end: &str) -> String {
    let block = block.trim_end();
    if existing.trim().is_empty() {
        return format!("{block}\n");
    }
    if let Some(start) = existing.find(begin) {
        if let Some(relative_end) = existing[start..].find(end) {
            let end_index = start + relative_end + end.len();
            return ensure_trailing_newline(format!(
                "{}{}{}",
                existing[..start].trim_end(),
                if existing[..start].trim().is_empty() {
                    ""
                } else {
                    "\n\n"
                },
                [
                    block,
                    existing[end_index..].trim_start_matches('\n').trim_end()
                ]
                .into_iter()
                .filter(|section| !section.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n")
            ));
        }
    }
    ensure_trailing_newline(format!("{}\n\n{block}", existing.trim_end()))
}

fn ensure_trailing_newline(content: String) -> String {
    if content.ends_with('\n') {
        content
    } else {
        format!("{content}\n")
    }
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn codex_hook_config(profiles_path: &str) -> String {
    format!(
        r#"# BEGIN semantic-agent-hook
# Generated by `semantic-agent-hook install --client codex`.
#
# Language providers own `.codex/agent-hook-profiles.json`; this root hook
# owns Codex payload parsing, shell classification, and deny rendering.

[[hooks.SessionStart]]
matcher = "startup|resume|clear|compact"

[[hooks.SessionStart.hooks]]
type = "command"
timeout = 5
statusMessage = "Loading semantic agent hook profiles"
command = '''
repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$repo_root"
exec semantic-agent-hook hook --client codex session-start --profiles "$repo_root/{profiles_path}"
'''

[[hooks.UserPromptSubmit]]

[[hooks.UserPromptSubmit.hooks]]
type = "command"
timeout = 5
statusMessage = "Planning semantic search flow"
command = '''
repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$repo_root"
exec semantic-agent-hook hook --client codex user-prompt --profiles "$repo_root/{profiles_path}"
'''

[[hooks.PreToolUse]]
matcher = ".*(Read|readFile|readDirectory|read_file|FsReadFile|FsReadDirectory|fs\\.read|fs\\.readDirectory|fs/readFile|fs/readDirectory|fs\\.readbin|writeFile|FsWriteFile|fs\\.write|fs/write|fs\\.writeFile|fs/writeFile|FsRemove|fs\\.remove|fs/remove|FsCopy|fs\\.copy|fs/copy|fs\\.rename|fs/rename|mcp__.*__read.*|Bash|exec_command|command_execution|apply_patch|Edit|Write).*"

[[hooks.PreToolUse.hooks]]
type = "command"
timeout = 5
statusMessage = "Checking semantic search flow"
command = '''
repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$repo_root"
exec semantic-agent-hook hook --client codex pre-tool --profiles "$repo_root/{profiles_path}"
'''

[[hooks.PermissionRequest]]
matcher = ".*(Read|readFile|readDirectory|read_file|FsReadFile|FsReadDirectory|fs\\.read|fs\\.readDirectory|fs/readFile|fs/readDirectory|fs\\.readbin|writeFile|FsWriteFile|fs\\.write|fs/write|fs\\.writeFile|fs/writeFile|FsRemove|fs\\.remove|fs/remove|FsCopy|fs\\.copy|fs/copy|fs\\.rename|fs/rename|mcp__.*__read.*|Bash|exec_command|command_execution|apply_patch|Edit|Write).*"

[[hooks.PermissionRequest.hooks]]
type = "command"
timeout = 5
statusMessage = "Checking semantic approval flow"
command = '''
repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$repo_root"
exec semantic-agent-hook hook --client codex permission-request --profiles "$repo_root/{profiles_path}"
'''

[[hooks.PostToolUse]]
matcher = ".*(Read|readFile|readDirectory|read_file|FsReadFile|FsReadDirectory|fs\\.read|fs\\.readDirectory|fs/readFile|fs/readDirectory|fs\\.readbin|writeFile|FsWriteFile|fs\\.write|fs/write|fs\\.writeFile|fs/writeFile|FsRemove|fs\\.remove|fs/remove|FsCopy|fs\\.copy|fs/copy|fs\\.rename|fs/rename|mcp__.*__read.*|Bash|exec_command|command_execution|apply_patch|Edit|Write).*"

[[hooks.PostToolUse.hooks]]
type = "command"
timeout = 5
statusMessage = "Updating semantic hook state"
command = '''
repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$repo_root"
exec semantic-agent-hook hook --client codex post-tool --profiles "$repo_root/{profiles_path}"
'''

[[hooks.SubagentStart]]
matcher = ".*"

[[hooks.SubagentStart.hooks]]
type = "command"
timeout = 5
statusMessage = "Preparing semantic subagent context"
command = '''
repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$repo_root"
exec semantic-agent-hook hook --client codex subagent-start --profiles "$repo_root/{profiles_path}"
'''

[[hooks.SubagentStop]]
matcher = ".*"

[[hooks.SubagentStop.hooks]]
type = "command"
timeout = 5
statusMessage = "Checking semantic subagent evidence"
command = '''
repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$repo_root"
exec semantic-agent-hook hook --client codex subagent-stop --profiles "$repo_root/{profiles_path}"
'''

[[hooks.Stop]]

[[hooks.Stop.hooks]]
type = "command"
timeout = 5
statusMessage = "Checking semantic changed files"
command = '''
repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$repo_root"
exec semantic-agent-hook hook --client codex stop --profiles "$repo_root/{profiles_path}"
'''
# END semantic-agent-hook
"#
    )
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == flag)
        .map(|window| window[1].as_str())
}

fn first_positional(args: &[String]) -> Option<&str> {
    let mut skip_next = false;
    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if matches!(arg.as_str(), "--client" | "--profiles" | "--emit") {
            skip_next = true;
            continue;
        }
        if !arg.starts_with('-') {
            return Some(arg.as_str());
        }
    }
    None
}
