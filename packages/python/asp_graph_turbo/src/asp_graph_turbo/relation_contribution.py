"""Relation contribution explanations for graph turbo ranking."""

from __future__ import annotations

from collections import defaultdict
from collections.abc import Iterable

from .model import Node, OrientedEdge


def graph_turbo_relation_reasons_by_node(
    edges: Iterable[OrientedEdge],
    ranked: Iterable[Node],
    *,
    max_relations: int = 3,
) -> dict[str, tuple[str, ...]]:
    ranked_ids = {node.id for node in ranked}
    relation_mass_by_node: dict[str, dict[str, float]] = defaultdict(dict)
    for edge in edges:
        if edge.target not in ranked_ids or edge.weight <= 0.0:
            continue
        relation_mass = relation_mass_by_node[edge.target]
        relation_mass[edge.relation] = (
            relation_mass.get(edge.relation, 0.0) + edge.weight
        )
    return {
        node_id: tuple(
            f"relation:{relation}:{weight:+.2f}"
            for relation, weight in _top_relations(relation_mass, max_relations)
        )
        for node_id, relation_mass in relation_mass_by_node.items()
    }


def _top_relations(
    relation_mass: dict[str, float], max_relations: int
) -> tuple[tuple[str, float], ...]:
    return tuple(
        sorted(relation_mass.items(), key=lambda item: (-item[1], item[0]))[
            :max_relations
        ]
    )
