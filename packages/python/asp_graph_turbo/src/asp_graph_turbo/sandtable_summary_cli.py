"""Summarize graph turbo benchmark and receipt packets for sandtables."""

from __future__ import annotations

import argparse
import json
import sys
from collections.abc import Mapping, Sequence
from pathlib import Path

from .benchmark_cli import benchmark_packet
from .cli import _load_packet


def main(argv: Sequence[str] | None = None) -> int:
    args = _parse_args(argv)
    benchmark = _load_benchmark(args)
    receipt = _load_receipt(args.receipt, args.receipt_fixture_id)
    summary = _summary_packet(benchmark, receipt, args.scenario)
    if args.format == "json":
        sys.stdout.write(json.dumps(summary, sort_keys=True) + "\n")
    else:
        sys.stdout.write(_render_text(summary) + "\n")
    return 0


def _load_benchmark(args: argparse.Namespace) -> Mapping[str, object]:
    if args.benchmark is not None:
        return _load_json(args.benchmark)
    packet = _load_packet(args.benchmark_packet)
    return benchmark_packet(
        packet,
        runs=args.benchmark_runs,
        warmup_runs=args.benchmark_warmup_runs,
        cache_mode=args.benchmark_cache_mode,
        profile=args.profile,
        seed=args.seed,
        limit=args.limit,
    )


def _summary_packet(
    benchmark: Mapping[str, object], receipt: Mapping[str, object], scenario: str | None
) -> dict[str, object]:
    benchmark_metrics = _mapping(benchmark.get("lastAlgorithmMetrics"))
    receipt_metrics = _mapping(receipt.get("metrics"))
    duration = _mapping(benchmark.get("durationMs"))
    scenario_name = scenario or str(receipt.get("taskFingerprint") or "graph-turbo")
    return {
        "schemaId": "agent.semantic-protocols.semantic-graph-turbo-sandtable-summary",
        "schemaVersion": "1",
        "protocolId": "agent.semantic-protocols.semantic-language",
        "protocolVersion": "1",
        "packetKind": "graph-turbo-sandtable-summary",
        "scenario": scenario_name,
        "profile": benchmark.get("profile"),
        "benchmark": {
            "runs": benchmark.get("runs"),
            "warmupRuns": benchmark.get("warmupRuns"),
            "cacheMode": benchmark.get("cacheMode"),
            "medianMs": duration.get("median"),
            "p95Ms": duration.get("p95"),
            "pathBackend": benchmark_metrics.get("pathBackend"),
            "pathPairCount": benchmark_metrics.get("pathPairCount"),
            "pathCandidateCount": benchmark_metrics.get("pathCandidateCount"),
            "pathFallbackCount": benchmark_metrics.get("pathFallbackCount"),
            "pathCount": benchmark_metrics.get("pathCount"),
            "relationChannelCount": benchmark_metrics.get("relationChannelCount"),
            "pprIterations": benchmark_metrics.get("pprIterations"),
            "cacheStatus": benchmark_metrics.get("cacheStatus"),
        },
        "receipt": {
            "receiptId": receipt.get("receiptId"),
            "frontierReturnedCount": receipt_metrics.get("frontierReturnedCount"),
            "frontierFollowedCount": receipt_metrics.get("frontierFollowedCount"),
            "frontierFollowRate": receipt_metrics.get("frontierFollowRate"),
            "codeActuallyReadCount": receipt_metrics.get("codeActuallyReadCount"),
            "rawReadFallbackCount": receipt_metrics.get("rawReadFallbackCount"),
            "duplicateSelectorCount": receipt_metrics.get("duplicateSelectorCount"),
            "sameOwnerScanCount": receipt_metrics.get("sameOwnerScanCount"),
            "commandsToFirstUsefulLocator": receipt_metrics.get(
                "commandsToFirstUsefulLocator"
            ),
            "commandsToValidation": receipt_metrics.get("commandsToValidation"),
        },
    }


def _render_text(packet: Mapping[str, object]) -> str:
    benchmark = _mapping(packet.get("benchmark"))
    receipt = _mapping(packet.get("receipt"))
    return (
        "[graph-sandtable-summary] "
        f"scenario={packet.get('scenario')} profile={packet.get('profile')} "
        f"medianMs={benchmark.get('medianMs')} p95Ms={benchmark.get('p95Ms')}\n"
        "benchmark="
        f"pathBackend={benchmark.get('pathBackend')},"
        f"pathPairs={benchmark.get('pathPairCount')},"
        f"pathCandidates={benchmark.get('pathCandidateCount')},"
        f"pathFallbacks={benchmark.get('pathFallbackCount')},"
        f"pprIterations={benchmark.get('pprIterations')},"
        f"cache={benchmark.get('cacheStatus')}\n"
        "receipt="
        f"followRate={receipt.get('frontierFollowRate')},"
        f"rawReadFallbacks={receipt.get('rawReadFallbackCount')},"
        f"duplicateSelectors={receipt.get('duplicateSelectorCount')},"
        f"sameOwnerScans={receipt.get('sameOwnerScanCount')},"
        f"commandsToValidation={receipt.get('commandsToValidation')}"
    )


def _load_receipt(path: str, fixture_id: str | None) -> Mapping[str, object]:
    packet = _load_json(path)
    fixtures = packet.get("fixtures")
    if not isinstance(fixtures, list):
        return packet
    if fixture_id is None:
        fixture = fixtures[0]
    else:
        fixture = next(
            (
                item
                for item in fixtures
                if isinstance(item, Mapping) and item.get("fixtureId") == fixture_id
            ),
            None,
        )
        if fixture is None:
            raise SystemExit(f"receipt fixture not found: {fixture_id}")
    if not isinstance(fixture, Mapping) or not isinstance(
        fixture.get("receipt"), Mapping
    ):
        raise SystemExit("receipt fixture entry must contain a receipt object")
    return fixture["receipt"]


def _load_json(path: str) -> Mapping[str, object]:
    packet = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(packet, Mapping):
        raise SystemExit(f"{path} must contain a JSON object")
    return packet


def _mapping(value: object) -> Mapping[str, object]:
    return value if isinstance(value, Mapping) else {}


def _parse_args(argv: Sequence[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    benchmark_source = parser.add_mutually_exclusive_group(required=True)
    benchmark_source.add_argument("--benchmark")
    benchmark_source.add_argument("--benchmark-packet")
    parser.add_argument("--benchmark-runs", type=_positive_int, default=30)
    parser.add_argument("--benchmark-warmup-runs", type=_non_negative_int, default=3)
    parser.add_argument(
        "--benchmark-cache-mode",
        choices=["packet", "enabled", "disabled"],
        default="packet",
    )
    parser.add_argument("--profile", default=None)
    parser.add_argument("--seed", action="append", default=[])
    parser.add_argument("--limit", type=int, default=None)
    parser.add_argument("--receipt", required=True)
    parser.add_argument("--receipt-fixture-id")
    parser.add_argument("--scenario")
    parser.add_argument("--format", choices=["json", "text"], default="json")
    return parser.parse_args(argv)


def _positive_int(value: str) -> int:
    parsed = int(value)
    if parsed < 1:
        raise argparse.ArgumentTypeError("value must be positive")
    return parsed


def _non_negative_int(value: str) -> int:
    parsed = int(value)
    if parsed < 0:
        raise argparse.ArgumentTypeError("value must be non-negative")
    return parsed


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
