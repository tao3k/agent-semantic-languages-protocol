from __future__ import annotations

import json
import sys
from copy import deepcopy
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator

_TESTS_ROOT = Path(__file__).resolve().parent
if str(_TESTS_ROOT) not in sys.path:
    sys.path.insert(0, str(_TESTS_ROOT))

from schema_validation import schema_validator_for  # noqa: E402

_REPO_ROOT = Path(__file__).resolve().parents[2]
_SCHEMA_PATH = (
    _REPO_ROOT / "schemas" / "semantic-fact-frontier-benchmark-report.v1.schema.json"
)
_FIXTURES_PATH = (
    _REPO_ROOT / "schemas" / "semantic-fact-frontier-benchmark-report.fixtures.v1.json"
)
_RECEIPT_FIXTURES_PATH = (
    _REPO_ROOT / "schemas" / "semantic-fact-frontier-receipt.fixtures.v1.json"
)


def _load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _validator() -> Draft202012Validator:
    return schema_validator_for(_SCHEMA_PATH)


def _reports() -> list[dict[str, Any]]:
    return _load_json(_FIXTURES_PATH)["reports"]


def _receipt_fixtures_by_id() -> dict[str, dict[str, Any]]:
    catalog = _load_json(_RECEIPT_FIXTURES_PATH)
    return {fixture["fixtureId"]: fixture for fixture in catalog["fixtures"]}


def test_frontier_benchmark_report_fixtures_match_schema() -> None:
    validator = _validator()

    for report in _reports():
        errors = sorted(validator.iter_errors(report), key=lambda error: error.path)

        assert not errors, [error.message for error in errors]


def test_frontier_benchmark_report_scenarios_match_receipt_fixtures() -> None:
    receipt_fixtures = _receipt_fixtures_by_id()

    for report in _reports():
        assert (
            report["sourceReceiptFixtureCatalog"]
            == "schemas/semantic-fact-frontier-receipt.fixtures.v1.json"
        )
        assert (
            report["sourceBenchmarkFixture"]
            == "sandtables/fixtures/asp/graph-turbo-owner-query.json"
        )

        for scenario in report["scenarios"]:
            receipt_fixture = receipt_fixtures[scenario["receiptFixtureId"]]
            receipt = receipt_fixture["receipt"]

            assert scenario["receiptId"] == receipt["receiptId"]
            assert scenario["receiptMetrics"] == receipt["metrics"]


def test_frontier_benchmark_report_summary_is_derived_from_scenarios() -> None:
    receipt_fixtures = _receipt_fixtures_by_id()

    for report in _reports():
        scenarios = report["scenarios"]
        summary = report["summary"]

        assert summary["scenarioCount"] == len(scenarios)
        assert summary["receiptFixtureCount"] == len(receipt_fixtures)
        assert summary["minFrontierFollowRate"] == min(
            scenario["receiptMetrics"]["frontierFollowRate"] for scenario in scenarios
        )
        assert summary["maxRawReadFallbackCount"] == max(
            scenario["receiptMetrics"]["rawReadFallbackCount"] for scenario in scenarios
        )
        assert summary["allRelationChannelsVisible"] is all(
            scenario["receiptMetrics"]["relationChannelCount"] > 0
            for scenario in scenarios
        )
        assert summary["runtimeCaptureScenarioCount"] == sum(
            scenario["benchmarkReadiness"]["hasRuntimeCapture"]
            for scenario in scenarios
        )
        assert summary["calibrationReadyScenarioCount"] == sum(
            scenario["benchmarkReadiness"]["readyForWeightCalibration"]
            for scenario in scenarios
        )


def test_frontier_benchmark_report_keeps_calibration_blocked_until_runtime_use() -> None:
    report = _reports()[0]
    scenarios = {scenario["scenarioId"]: scenario for scenario in report["scenarios"]}

    asp_runtime = scenarios["asp-runtime-frontier-only"]

    assert asp_runtime["benchmarkReadiness"]["hasRuntimeCapture"] is True
    assert asp_runtime["benchmarkReadiness"]["hasFollowedFrontier"] is False
    assert asp_runtime["benchmarkReadiness"]["hasValidationCommand"] is False
    assert report["summary"]["calibrationReadyScenarioCount"] == 0


def test_frontier_benchmark_report_rejects_unknown_capture_kind() -> None:
    report = deepcopy(_reports()[0])
    report["scenarios"][0]["captureKind"] = "obsolete-frontier-kind"

    errors = sorted(_validator().iter_errors(report), key=lambda error: error.path)

    assert any("is not one of" in error.message for error in errors)


def test_frontier_benchmark_report_requires_benchmark_metrics() -> None:
    report = deepcopy(_reports()[0])
    report["scenarios"][0].pop("benchmarkMetrics")

    errors = sorted(_validator().iter_errors(report), key=lambda error: error.path)

    assert any("'benchmarkMetrics' is a required property" in error.message for error in errors)
