"""Performance guards for Python harness query/search warm paths."""

from __future__ import annotations

import io
import sys
import time
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parents[2]
_PY_HARNESS_SRC = _REPO_ROOT / "languages/python-lang-project-harness/src"
if str(_PY_HARNESS_SRC) not in sys.path:
    sys.path.insert(0, str(_PY_HARNESS_SRC))

from python_lang_project_harness import run_cli  # noqa: E402

_WARM_CLI_BUDGET_MS = 250.0


def test_python_harness_search_and_query_warm_paths_are_millisecond_scale(tmp_path) -> None:
    project = _write_python_fixture(tmp_path)

    _run_cli(["search", "fzf", "compute_value", "owner", "tests", str(project)], project)
    search_ms, search_out = _timed_cli(
        ["search", "fzf", "compute_value", "owner", "tests", str(project)],
        project,
    )
    _run_cli(
        ["query", "src/example.py", "--term", "compute_value", "--names-only", str(project)],
        project,
    )
    query_ms, query_out = _timed_cli(
        ["query", "src/example.py", "--term", "compute_value", "--names-only", str(project)],
        project,
    )

    assert "compute_value" in search_out
    assert "compute_value" in query_out
    assert search_ms < _WARM_CLI_BUDGET_MS
    assert query_ms < _WARM_CLI_BUDGET_MS


def _write_python_fixture(tmp_path: Path) -> Path:
    project = tmp_path / "project"
    source_dir = project / "src"
    source_dir.mkdir(parents=True)
    (project / "pyproject.toml").write_text(
        "[project]\nname = \"perf-fixture\"\nversion = \"0.1.0\"\n",
        encoding="utf-8",
    )
    (source_dir / "example.py").write_text(
        "def compute_value(item: int) -> int:\n"
        "    return item + 1\n\n"
        "def unrelated() -> int:\n"
        "    return 0\n",
        encoding="utf-8",
    )
    return project


def _timed_cli(args: list[str], cwd: Path) -> tuple[float, str]:
    started = time.perf_counter()
    output = _run_cli(args, cwd)
    return (time.perf_counter() - started) * 1000.0, output


def _run_cli(args: list[str], cwd: Path) -> str:
    stdout = io.StringIO()
    stderr = io.StringIO()
    exit_code = run_cli(args, stdout=stdout, stderr=stderr, cwd=cwd)
    assert exit_code == 0, stderr.getvalue()
    return stdout.getvalue()
