"""Ablation report CLI tests for graph turbo calibration."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

from asp_graph_turbo_cli_support import (
    sample_graph_turbo_request,
    validate_shared_schema,
)


def test_graph_turbo_ablation_report_cli_generates_schema_packet(tmp_path) -> None:
    packet_path = tmp_path / "graph-turbo-request.json"
    packet_path.write_text(json.dumps(sample_graph_turbo_request()), encoding="utf-8")

    completed = subprocess.run(
        [
            sys.executable,
            "-m",
            "asp_graph_turbo",
            "ablation-report",
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
        env=_subprocess_env(),
    )
    payload = json.loads(completed.stdout)
    validate_shared_schema(
        payload,
        "semantic-graph-turbo-ablation-report.v1.schema.json",
    )
    variants = {entry["variant"]: entry for entry in payload["variants"]}

    assert payload["packetKind"] == "graph-turbo-ablation-report"
    assert payload["summary"]["variantCount"] == 6
    assert payload["qualityGate"]["status"] == "pass"
    assert variants["full"]["comparison"]["rankOverlapRatio"] == 1.0
    assert variants["full"]["comparison"]["scoreDeltaL1"] == 0.0
    assert "transitionNonZeroDelta" in variants["no-provider-facts"]["comparison"]


def test_graph_turbo_ablation_report_cli_can_render_text(tmp_path) -> None:
    packet_path = tmp_path / "graph-turbo-request.json"
    packet_path.write_text(json.dumps(sample_graph_turbo_request()), encoding="utf-8")

    completed = subprocess.run(
        [
            sys.executable,
            "-m",
            "asp_graph_turbo",
            "ablation-report",
            str(packet_path),
            "--variant",
            "no-provider-facts",
            "--runs",
            "1",
            "--warmup-runs",
            "0",
            "--format",
            "text",
        ],
        check=True,
        text=True,
        capture_output=True,
        env=_subprocess_env(),
    )

    assert completed.stdout.startswith("[graph-ablation-report] ")
    assert "gate=pass" in completed.stdout
    assert "variant=no-provider-facts" in completed.stdout
    assert "readMemoryDelta=" in completed.stdout
    assert "receiptBoostDelta=" in completed.stdout
    assert "transitionNnzDelta=" in completed.stdout


def test_graph_turbo_ablation_report_exposes_receipt_read_memory_and_quality_deltas(
    tmp_path,
) -> None:
    packet_path = tmp_path / "graph-turbo-sensitive-request.json"
    packet_path.write_text(
        _sensitive_fixture_path().read_text(encoding="utf-8"),
        encoding="utf-8",
    )

    completed = subprocess.run(
        [
            sys.executable,
            "-m",
            "asp_graph_turbo",
            "ablation-report",
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
        env=_subprocess_env(),
    )
    payload = json.loads(completed.stdout)
    validate_shared_schema(
        payload,
        "semantic-graph-turbo-ablation-report.v1.schema.json",
    )
    variants = {entry["variant"]: entry for entry in payload["variants"]}

    assert variants["no-read-memory"]["comparison"]["readMemorySuppressedDelta"] < 0
    assert variants["no-receipt"]["comparison"]["receiptBoostDelta"] < 0
    assert variants["no-quality-fields"]["comparison"]["scoreDeltaL1"] > 0


def test_graph_turbo_ablation_report_can_fail_quality_gate(tmp_path) -> None:
    packet_path = tmp_path / "graph-turbo-sensitive-request.json"
    packet_path.write_text(
        _sensitive_fixture_path().read_text(encoding="utf-8"),
        encoding="utf-8",
    )

    completed = subprocess.run(
        [
            sys.executable,
            "-m",
            "asp_graph_turbo",
            "ablation-report",
            str(packet_path),
            "--runs",
            "1",
            "--warmup-runs",
            "0",
            "--cache-mode",
            "disabled",
            "--min-worst-rank-overlap-ratio",
            "0.95",
            "--fail-on-quality-gate",
            "--format",
            "json",
        ],
        text=True,
        capture_output=True,
        env=_subprocess_env(),
    )
    payload = json.loads(completed.stdout)

    assert completed.returncode == 1
    assert payload["qualityGate"]["status"] == "fail"
    assert payload["qualityGate"]["failures"][0]["field"] == (
        "summary.worstRankOverlapRatio"
    )


def _subprocess_env() -> dict[str, str]:
    repo_root = Path(__file__).resolve().parents[2]
    package_src = repo_root / "packages/python/asp_graph_turbo/src"
    unit_tests = repo_root / "tests/unit"
    env = os.environ.copy()
    env["PYTHONPATH"] = os.pathsep.join(
        [str(package_src), str(unit_tests), env.get("PYTHONPATH", "")]
    ).rstrip(os.pathsep)
    return env


def _sensitive_fixture_path() -> Path:
    return (
        Path(__file__).resolve().parents[2]
        / "sandtables/fixtures/asp/graph-turbo-sensitive-ablation.json"
    )
