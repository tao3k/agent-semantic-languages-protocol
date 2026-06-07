"""Validate large-library deep question sandtable coverage."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator


REPO_ROOT = Path(__file__).resolve().parents[3]
SCHEMA_PATH = REPO_ROOT / "schemas" / "semantic-sandtable-scenario.v1.schema.json"
RUST_MATRIX_PATHS = [
    REPO_ROOT / "sandtables" / "rust" / "tokio-intent-matrix.json",
    REPO_ROOT / "sandtables" / "rust" / "bytes-intent-matrix.json",
    REPO_ROOT / "sandtables" / "rust" / "ignore-intent-matrix.json",
]
LIVE_TOKIO_PATH = (
    REPO_ROOT / "sandtables" / "rust" / "tokio-claude-deep-question-flow.json"
)


def _load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text())


class DeepQuestionCaseTests(unittest.TestCase):
    def setUp(self) -> None:
        self.validator = Draft202012Validator(_load_json(SCHEMA_PATH))

    def assert_valid_scenario(self, scenario: dict[str, Any]) -> None:
        errors = [error.message for error in self.validator.iter_errors(scenario)]
        self.assertEqual([], errors)

    def test_rust_large_library_matrices_define_ten_deep_questions(self) -> None:
        total_questions = 0
        for path in RUST_MATRIX_PATHS:
            scenario = _load_json(path)
            self.assert_valid_scenario(scenario)
            evidence = scenario["evidence"]
            step_ids = {step["id"] for step in scenario["steps"]}
            deep_questions = evidence["deepQuestionCases"]
            self.assertGreaterEqual(len(deep_questions), 3)
            total_questions += len(deep_questions)

            for question in deep_questions:
                self.assertTrue(question["question"].strip())
                self.assertGreaterEqual(len(question["queryTerms"]), 3)
                self.assertTrue(set(question["stepIds"]).issubset(step_ids))
                audit = question["audit"]
                self.assertLessEqual(audit["maxSearchCommands"], audit["maxAspCommands"])
                self.assertLessEqual(audit["maxQueryCommands"], audit["maxAspCommands"])
                self.assertEqual(0, audit["maxRepeatedCommands"])
                self.assertTrue(audit["requiresGraphSignals"])
                self.assertTrue(audit["requiresQuerySet"])

        self.assertEqual(10, total_questions)

    def test_live_tokio_claude_deep_question_is_env_gated(self) -> None:
        scenario = _load_json(LIVE_TOKIO_PATH)
        self.assert_valid_scenario(scenario)
        self.assertEqual(
            [
                "ASP_LIVE_CLAUDE_CLI",
                "ANTHROPIC_AUTH_TOKEN",
                "SANDTABLE_RUST_TOKIO_060_ROOT",
            ],
            scenario["skipUnlessEnv"],
        )
        step = scenario["steps"][0]
        self.assertEqual("agent-cli", step["kind"])
        self.assertEqual("claude", step["agentCli"]["client"])
        self.assertTrue(step["agentCli"]["includeHookEvents"])
        deep_question = scenario["evidence"]["deepQuestionCases"][0]
        self.assertIn("tokio 0.6.0", deep_question["question"])
        self.assertEqual(["claude-tokio-vec-scalar"], deep_question["stepIds"])


if __name__ == "__main__":
    unittest.main()
