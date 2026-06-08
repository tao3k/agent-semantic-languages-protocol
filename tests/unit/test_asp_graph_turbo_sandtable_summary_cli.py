"""Sandtable summary command tests for graph turbo."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from unit.asp_graph_turbo_cli_support import (
    sample_graph_turbo_request,
    validate_shared_schema,
)


def test_graph_turbo_sandtable_summary_combines_benchmark_and_receipt(tmp_path) -> None:
    packet_path = tmp_path / "graph-turbo-request.json"
    benchmark_path = tmp_path / "graph-turbo-benchmark.json"
    packet_path.write_text(json.dumps(sample_graph_turbo_request()), encoding="utf-8")
    benchmark_path.write_text(
        _benchmark_stdout(packet_path),
        encoding="utf-8",
    )

    completed = subprocess.run(
        [
            sys.executable,
            "-m",
            "asp_graph_turbo",
            "sandtable-summary",
            "--benchmark",
            str(benchmark_path),
            "--receipt",
            str(_receipt_fixtures_path()),
            "--scenario",
            "unit-summary",
            "--format",
            "json",
        ],
        check=True,
        text=True,
        capture_output=True,
    )
    payload = json.loads(completed.stdout)
    validate_shared_schema(
        payload,
        "semantic-graph-turbo-sandtable-summary.v1.schema.json",
    )

    assert payload["scenario"] == "unit-summary"
    assert payload["benchmark"]["pathCandidateCount"] >= 1
    assert payload["receipt"]["frontierReturnedCount"] >= 1
    assert payload["receipt"]["commandsToValidation"] >= 1


def test_graph_turbo_sandtable_summary_text_is_single_table_row(tmp_path) -> None:
    packet_path = tmp_path / "graph-turbo-request.json"
    benchmark_path = tmp_path / "graph-turbo-benchmark.json"
    packet_path.write_text(json.dumps(sample_graph_turbo_request()), encoding="utf-8")
    benchmark_path.write_text(_benchmark_stdout(packet_path), encoding="utf-8")

    completed = subprocess.run(
        [
            sys.executable,
            "-m",
            "asp_graph_turbo",
            "sandtable-summary",
            "--benchmark",
            str(benchmark_path),
            "--receipt",
            str(_receipt_fixtures_path()),
            "--format",
            "text",
        ],
        check=True,
        text=True,
        capture_output=True,
    )

    assert completed.stdout.startswith("[graph-sandtable-summary]")
    assert "pathCandidates=" in completed.stdout
    assert "commandsToValidation=" in completed.stdout


def test_graph_turbo_sandtable_summary_can_benchmark_packet_inline(tmp_path) -> None:
    packet_path = tmp_path / "graph-turbo-request.json"
    packet_path.write_text(json.dumps(sample_graph_turbo_request()), encoding="utf-8")

    completed = subprocess.run(
        [
            sys.executable,
            "-m",
            "asp_graph_turbo",
            "sandtable-summary",
            "--benchmark-packet",
            str(packet_path),
            "--benchmark-runs",
            "2",
            "--benchmark-warmup-runs",
            "0",
            "--benchmark-cache-mode",
            "disabled",
            "--receipt",
            str(_receipt_fixtures_path()),
            "--scenario",
            "inline-benchmark",
            "--format",
            "json",
        ],
        check=True,
        text=True,
        capture_output=True,
    )
    payload = json.loads(completed.stdout)
    validate_shared_schema(
        payload,
        "semantic-graph-turbo-sandtable-summary.v1.schema.json",
    )

    assert payload["scenario"] == "inline-benchmark"
    assert payload["benchmark"]["cacheMode"] == "disabled"
    assert payload["benchmark"]["pathCandidateCount"] >= 1


def test_tools_route_exposes_graph_turbo_benchmark(tmp_path) -> None:
    packet_path = tmp_path / "graph-turbo-request.json"
    packet_path.write_text(json.dumps(sample_graph_turbo_request()), encoding="utf-8")

    completed = subprocess.run(
        [
            sys.executable,
            "-m",
            "tools",
            "graph",
            "turbo",
            "benchmark",
            str(packet_path),
            "--runs",
            "1",
            "--warmup-runs",
            "0",
            "--cache-mode",
            "disabled",
            "--format",
            "json",
        ],
        check=True,
        text=True,
        capture_output=True,
    )
    payload = json.loads(completed.stdout)

    assert payload["packetKind"] == "graph-turbo-benchmark"
    assert payload["runs"] == 1


def _benchmark_stdout(packet_path: Path) -> str:
    completed = subprocess.run(
        [
            sys.executable,
            "-m",
            "asp_graph_turbo",
            "benchmark",
            str(packet_path),
            "--runs",
            "2",
            "--warmup-runs",
            "0",
            "--cache-mode",
            "disabled",
            "--format",
            "json",
        ],
        check=True,
        text=True,
        capture_output=True,
    )
    return completed.stdout


def _receipt_fixtures_path() -> Path:
    return (
        Path(__file__).resolve().parents[2]
        / "schemas"
        / ("semantic-fact-frontier-receipt.fixtures.v1.json")
    )
