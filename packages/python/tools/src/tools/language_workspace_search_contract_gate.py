"""Scenario runner for the cross-language workspace/search contract gate."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .language_workspace_search_contract_assertions import (
    assert_failure_contains,
    assert_ingest_result,
    assert_registry,
    assert_workspace_result,
    expect_success,
)
from .language_workspace_search_contract_cases import (
    CONTRACT_CASES,
    SearchContractCase,
)
from .language_workspace_search_contract_runner import _run_asp
from .language_workspace_search_contract_types import RunAsp
from .paths import repo_root as default_repo_root


@dataclass(frozen=True)
class ContractContext:
    root: Path
    asp_bin: str | None
    runner: RunAsp

    def run(self, args: list[str]) -> object:
        return self.runner(args, self.root, self.asp_bin)


def run_contract(
    *,
    repo_root: Path | None = None,
    asp_bin: str | None = None,
    run_asp: RunAsp | None = None,
) -> None:
    context = _contract_context(repo_root, asp_bin, run_asp)
    for case in CONTRACT_CASES:
        _validate_workspace(context, case)
        _validate_ingest(context, case)
    for case in CONTRACT_CASES:
        _validate_registry(context, case)


def _contract_context(
    repo_root: Path | None,
    asp_bin: str | None,
    run_asp: RunAsp | None,
) -> ContractContext:
    return ContractContext(
        root=(repo_root or default_repo_root()).resolve(),
        asp_bin=asp_bin,
        runner=_run_asp if run_asp is None else run_asp,
    )


def _validate_workspace(context: ContractContext, case: SearchContractCase) -> None:
    result = expect_success(
        context.run(_workspace_args(case)),
        f"{case.language} workspace",
    )
    assert_workspace_result(result, case)


def _validate_ingest(context: ContractContext, case: SearchContractCase) -> None:
    ingest = expect_success(context.run(_ingest_args(case)), f"{case.language} ingest")
    assert_ingest_result(ingest, case)
    assert_failure_contains(
        context.run([*_ingest_args(case), "."]),
        f"{case.language} ingest extra root",
        "expected at most one PROJECT_ROOT argument",
    )


def _validate_registry(context: ContractContext, case: SearchContractCase) -> None:
    result = expect_success(
        context.run(
            [case.language, "agent", "doctor", "--json", case.project_root],
        ),
        f"{case.language} registry",
    )
    assert_registry(result.stdout, case)


def _workspace_args(case: SearchContractCase) -> list[str]:
    return [
        case.language,
        "search",
        "workspace",
        "--view",
        "seeds",
        case.project_root,
    ]


def _ingest_args(case: SearchContractCase) -> list[str]:
    return [
        case.language,
        "search",
        "ingest",
        *case.ingest_pipes,
        "--view",
        "seeds",
        case.project_root,
    ]
