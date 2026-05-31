from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator


REPO_ROOT = Path(__file__).resolve().parents[1]


def minimal_packet() -> dict[str, object]:
    return {
        "schemaId": "agent.semantic-protocols.semantic-search-packet",
        "schemaVersion": "1",
        "protocolId": "agent.semantic-protocols.semantic-language",
        "protocolVersion": "1",
        "languageId": "typescript",
        "providerId": "ts-harness",
        "binary": "ts-harness",
        "namespace": "agent.semantic-protocols.languages.typescript.ts-harness",
        "method": "search/owner",
        "projectRoot": ".",
        "view": "owner",
        "renderMode": "graph",
        "header": {"kind": "search-owner", "fields": {}},
        "nodes": [
            {
                "id": "O:src/components/WorkflowExecution.tsx",
                "kind": "owner",
                "path": "src/components/WorkflowExecution.tsx",
                "fields": {},
            }
        ],
        "edges": [],
        "owners": [
            {
                "path": "src/components/WorkflowExecution.tsx",
                "role": "source",
                "public": False,
                "fields": {},
            }
        ],
        "hits": [
            {
                "kind": "text",
                "ownerPath": "src/components/WorkflowExecution.tsx",
                "location": {
                    "path": "src/components/WorkflowExecution.tsx",
                    "line": 42,
                    "column": 17,
                },
                "score": 1.0,
                "reason": "parser-visible-source",
            }
        ],
        "findings": [],
        "nextActions": [
            {
                "kind": "owner",
                "target": "src/data/workflows.ts",
                "ownerPath": "src/components/WorkflowExecution.tsx",
            }
        ],
        "notes": [],
    }


class SemanticSearchPacketSchemaTests(unittest.TestCase):
    def setUp(self) -> None:
        schema_path = REPO_ROOT / "schemas" / "semantic-search-packet.v1.schema.json"
        with schema_path.open("r", encoding="utf-8") as handle:
            self.validator = Draft202012Validator(json.load(handle))

    def validation_errors(self, packet: dict[str, object]) -> list[str]:
        return [error.message for error in self.validator.iter_errors(packet)]

    def test_project_root_relative_paths_are_valid(self) -> None:
        self.assertEqual([], self.validation_errors(minimal_packet()))

    def test_workspace_root_path_is_valid(self) -> None:
        packet = minimal_packet()
        packet["nodes"] = [{"id": "O:.", "kind": "owner", "path": ".", "fields": {}}]
        packet["owners"] = [
            {"path": ".", "role": "root", "public": True, "fields": {}}
        ]
        packet["hits"] = copy.deepcopy(packet["hits"])
        packet["hits"][0]["ownerPath"] = "."
        packet["hits"][0]["location"]["path"] = "."

        self.assertEqual([], self.validation_errors(packet))

    def test_rank_prefixed_owner_paths_are_rejected(self) -> None:
        packet = minimal_packet()
        packet["owners"] = [
            {"path": "0:src/components/WorkflowExecution.tsx", "role": "source", "public": False, "fields": {}}
        ]

        errors = self.validation_errors(packet)

        self.assertTrue(any("does not match" in message for message in errors))

    def test_relative_escape_location_paths_are_rejected(self) -> None:
        packet = minimal_packet()
        packet["hits"] = copy.deepcopy(packet["hits"])
        packet["hits"][0]["location"]["path"] = "../src/components/WorkflowExecution.tsx"

        errors = self.validation_errors(packet)

        self.assertTrue(any("does not match" in message for message in errors))


if __name__ == "__main__":
    unittest.main()
