"""Validate sandtable step expectation behavior."""

from __future__ import annotations

from pathlib import Path

from tools.semantic_sandtable.expectations import validate_step
from tools.semantic_sandtable.models import StepResult


def test_allow_non_zero_exit_suppresses_exit_code_error() -> None:
    result = StepResult(
        scenario_id="rust.live",
        step_id="claude",
        command=["claude"],
        status="pass",
        exit_code=1,
        elapsed_ms=10,
        stdout_lines=1,
        stderr_lines=0,
        stdout_bytes=4,
        stderr_bytes=0,
    )

    validate_step(
        {"expect": {"allowNonZeroExit": True}},
        result,
        "done",
        "",
        Path("."),
    )

    assert result.errors == []


def test_pipe_flow_expectation_reports_budget_and_stage_drift() -> None:
    result = StepResult(
        scenario_id="rust.live",
        step_id="claude",
        command=["claude"],
        status="pass",
        exit_code=1,
        elapsed_ms=10,
        stdout_lines=1,
        stderr_lines=0,
        stdout_bytes=4,
        stderr_bytes=0,
        observations={
            "pipeFlow": {
                "aspCommands": 9,
                "searchCommands": 5,
                "queryCommands": 1,
                "repeatedCommands": 1,
                "searchPipeCommands": 2,
                "searchPrimeCommands": 2,
                "searchFzfCommands": 0,
                "searchReasoningCommands": 1,
                "querySelectorCommands": 0,
                "complexPipeFlow": False,
                "missingComplexPipeStages": ["query-selector"],
            }
        },
    )

    validate_step(
        {
            "expect": {
                "allowNonZeroExit": True,
                "pipeFlow": {
                    "maxAspCommands": 8,
                    "maxSearchCommands": 4,
                    "maxRepeatedCommands": 0,
                    "maxSearchPipeCommands": 1,
                    "maxSearchPrimeCommands": 1,
                    "minQuerySelectorCommands": 1,
                    "requireComplexPipeFlow": True,
                    "requireTokenCost": True,
                    "requiredStages": ["search-pipe", "query-selector"],
                    "forbiddenStages": ["repeated-prime", "repeated-commands"],
                },
            }
        },
        result,
        "done",
        "",
        Path("."),
    )

    assert "pipeFlow aspCommands=9 exceeds maxAspCommands=8" in result.errors
    assert "pipeFlow searchCommands=5 exceeds maxSearchCommands=4" in result.errors
    assert "pipeFlow repeatedCommands=1 exceeds maxRepeatedCommands=0" in result.errors
    assert (
        "pipeFlow searchPipeCommands=2 exceeds maxSearchPipeCommands=1"
        in result.errors
    )
    assert (
        "pipeFlow searchPrimeCommands=2 exceeds maxSearchPrimeCommands=1"
        in result.errors
    )
    assert (
        "pipeFlow querySelectorCommands=0 below minQuerySelectorCommands=1"
        in result.errors
    )
    assert "pipeFlow complex=false missing=['query-selector']" in result.errors
    assert "tokenCost missing from agent observations" in result.errors
    assert "pipeFlow missing required stage 'query-selector'" in result.errors
    assert "pipeFlow contains forbidden stage 'repeated-prime'" in result.errors
    assert "pipeFlow contains forbidden stage 'repeated-commands'" in result.errors
