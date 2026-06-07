use std::path::{Path, PathBuf};

use crate::provider_command::support::asp_command;

pub(super) fn write_org_elements_fixture(root: &Path) -> PathBuf {
    let path = root.join("plan.org");
    std::fs::write(
        &path,
        "* TODO [#A] Task :work:\nSCHEDULED: <2026-06-06 Sat>\n:PROPERTIES:\n:CUSTOM_ID: task-1\n:END:\n\nProvider activation carries execution mode.\nDocument providers stay embedded inside ASP.\n\n** Repository Map\n*** Docs\n| Key | Value |\n| Foo | Bar |\n\n- [X] ship element map\n- plain list item\n[[https://example.com][site]]\n[[file:diagram.png]]\n\n#+begin_src rust\nfn main() {}\n#+end_src\n\n#+begin_export html\n<div>exported</div>\n#+end_export\n",
    )
    .expect("write org elements fixture");
    path
}

pub(super) fn asp_org_query(root: &Path, args: &[&str]) -> String {
    let output = asp_command(root)
        .arg("org")
        .args(args)
        .output()
        .expect("run asp org query");
    assert!(
        output.status.success(),
        "args={args:?} stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout")
}
