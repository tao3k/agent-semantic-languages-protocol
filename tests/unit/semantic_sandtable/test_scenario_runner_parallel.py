"""Validate scenario-level parallel sandtable execution."""

from __future__ import annotations

import time
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.semantic_sandtable.models import ScenarioResult, StepResult
from tools.semantic_sandtable.scenario_runner import _run_scenario_steps


class ScenarioRunnerParallelTests(unittest.TestCase):
    def test_parallel_execution_preserves_step_order(self) -> None:
        result = ScenarioResult(
            scenario_id="rust.parallel",
            language="rust",
            path=Path("scenario.json"),
            status="pass",
            workdir=Path("."),
        )
        steps = [{"id": "one"}, {"id": "two"}, {"id": "three"}]

        def fake_run_step(**kwargs: object) -> StepResult:
            step = kwargs["step"]
            assert isinstance(step, dict)
            time.sleep(0.15)
            return StepResult(
                scenario_id="rust.parallel",
                step_id=str(step["id"]),
                command=["fake"],
                status="pass",
                exit_code=0,
                elapsed_ms=150,
                stdout_lines=0,
                stderr_lines=0,
                stdout_bytes=0,
                stderr_bytes=0,
            )

        started = time.perf_counter()
        with patch(
            "tools.semantic_sandtable.scenario_runner.run_step",
            side_effect=fake_run_step,
        ):
            totals = _run_scenario_steps(
                repo_root=Path("."),
                workdir=Path("."),
                scenario_id="rust.parallel",
                steps=steps,
                env={},
                captures={},
                result=result,
                execution={"mode": "parallel", "maxConcurrentSteps": 3},
            )
        elapsed = time.perf_counter() - started

        self.assertLess(elapsed, 0.35)
        self.assertEqual(
            ["one", "two", "three"], [step.step_id for step in result.steps]
        )
        self.assertEqual(450, totals["elapsedMs"])


if __name__ == "__main__":
    unittest.main()
