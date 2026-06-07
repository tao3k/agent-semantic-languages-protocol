"""ASP subprocess runner for the language workspace/search contract gate."""

from __future__ import annotations

import os
import shutil
import subprocess
from collections.abc import Sequence
from pathlib import Path

from .language_workspace_search_contract_types import AspResult, ContractFailure


def _run_asp(args: Sequence[str], root: Path, asp_bin: str | None) -> AspResult:
    command = [*_asp_command(root, asp_bin), *args]
    completed = subprocess.run(
        command,
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    return AspResult(
        args=tuple(args),
        returncode=completed.returncode,
        stdout=completed.stdout,
        stderr=completed.stderr,
    )


def _asp_command(root: Path, asp_bin: str | None) -> list[str]:
    if asp_bin:
        return [asp_bin]
    env_bin = os.environ.get("SEMANTIC_AGENT_PROTOCOL_BIN")
    if env_bin:
        return [env_bin]
    if shutil.which("cargo"):
        return [
            "cargo",
            "run",
            "-q",
            "-p",
            "agent-semantic-protocol",
            "--bin",
            "asp",
            "--",
        ]
    debug_asp = root / "target/debug/asp"
    if debug_asp.is_file():
        return [str(debug_asp)]
    raise ContractFailure("missing cargo and SEMANTIC_AGENT_PROTOCOL_BIN for asp contract gate")
