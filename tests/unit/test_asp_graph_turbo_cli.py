"""CLI entrypoint tests for the packaged ASP graph turbo command."""

from __future__ import annotations

import json
import subprocess
import sys

from asp_graph_turbo.graph_turbo_cli import main


def test_graph_turbo_dispatcher_help_lists_subcommands(capsys) -> None:
    assert main(["help"]) == 0

    captured = capsys.readouterr()

    assert "usage: asp-graph-turbo <command> [args]" in captured.out
    assert "rank" in captured.out
    assert "artifacts" in captured.out
    assert "timeline" in captured.out
    assert "metrics" in captured.out


def test_graph_turbo_dispatcher_routes_metrics_command(capsys) -> None:
    assert (
        main(
            [
                "metrics",
                "--scenario",
                "rust-fzf-request-rank",
                "--measured-at",
                "2026-06-07T00:50:24Z",
                "--profile",
                "owner-query",
                "--command",
                "asp rust search fzf --view graph-turbo-request",
                "--command",
                "graph-turbo rank --format compact",
                "--command-count",
                "2",
                "--packet-bytes",
                "22359",
                "--result-bytes",
                "1181",
                "--latency-ms",
                "811",
                "--repeated-trigger-patterns",
                "0",
                "--missing-facts",
                "0",
                "--confusing-next-actions",
                "0",
            ]
        )
        == 0
    )

    captured = capsys.readouterr()

    assert captured.out.startswith("[graph-turbo-real-trigger]")
    assert "packetBytes=22359" in captured.out


def test_graph_turbo_module_entrypoint_dispatches_timeline_json(tmp_path) -> None:
    completed = subprocess.run(
        [
            sys.executable,
            "-m",
            "asp_graph_turbo",
            "timeline",
            str(tmp_path),
            "--format",
            "json",
        ],
        check=True,
        text=True,
        capture_output=True,
    )
    payload = json.loads(completed.stdout)

    assert payload["schemaId"] == (
        "agent.semantic-protocols.graph-turbo-artifact-timeline"
    )
    assert payload["eventCount"] == 0
    assert payload["actionSummary"]["actionCount"] == 0
    assert payload["efficiencyEstimate"]["observedActions"] == 0
