//! Document language provider facade backed by orgize.

use std::env;
use std::process::Command;

const DOCUMENT_LANGUAGES: &[&str] = &["org", "md"];

pub(crate) fn is_document_language(language_id: &str) -> bool {
    DOCUMENT_LANGUAGES.contains(&language_id)
}

pub(crate) fn run_language_command(language_id: &str, args: &[String]) -> Result<(), String> {
    if is_help(args) {
        println!("{}", usage(language_id));
        return Ok(());
    }
    let Some(command) = args.first().map(String::as_str) else {
        return Err(usage(language_id));
    };
    if !matches!(command, "guide" | "search" | "query") {
        return Err(format!(
            "asp {language_id}: unsupported document command `{command}`; supported commands are guide, search, query"
        ));
    }

    let mut process = orgize_command();
    if language_id == "md" {
        process.arg("md");
    }
    process.args(args);
    let status = process.status().map_err(|error| {
        format!(
            "failed to run orgize for asp {language_id}: {error}; set ASP_ORGIZE_BIN or put orgize on PATH"
        )
    })?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

fn is_help(args: &[String]) -> bool {
    args.iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h" | "help"))
}

fn orgize_command() -> Command {
    if let Some(path) = env::var_os("ASP_ORGIZE_BIN") {
        return Command::new(path);
    }
    if let Ok(current_dir) = env::current_dir() {
        for dir in current_dir.ancestors() {
            let candidate = dir
                .join("languages/orgize/target/debug")
                .join(format!("orgize{}", env::consts::EXE_SUFFIX));
            if candidate.is_file() {
                return Command::new(candidate);
            }
        }
    }
    Command::new("orgize")
}

fn usage(language_id: &str) -> String {
    format!("usage: asp {language_id} <guide|search|query> ...")
}
