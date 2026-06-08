"""Profile relation matrix summaries for graph-turbo results."""

from __future__ import annotations

from .model import (
    Edge,
    GraphProfile,
    ProfileMatrixSummary,
    RelationChannelSummary,
    TypedGraph,
)
from .profiles import DEFAULT_PROFILES, allowed_oriented_edges


def profile_matrix_bank(
    graph: TypedGraph,
    selected_profile: GraphProfile,
    reachable_node_ids: frozenset[str],
) -> tuple[ProfileMatrixSummary, ...]:
    return tuple(
        _profile_matrix_summary(
            graph,
            profile,
            reachable_node_ids
            if profile.name == selected_profile.name
            else frozenset(),
        )
        for profile in DEFAULT_PROFILES.values()
    )


def _profile_matrix_summary(
    graph: TypedGraph, profile: GraphProfile, reachable_node_ids: frozenset[str]
) -> ProfileMatrixSummary:
    oriented_edges = allowed_oriented_edges(graph, profile)
    relation_channels = _relation_channels(
        profile.allowed_relations,
        oriented_edges,
        reachable_node_ids,
    )
    reachable_edge_count = sum(
        channel.reachable_edge_count for channel in relation_channels
    )
    supported_edge_count = sum(
        channel.supported_edge_count for channel in relation_channels
    )
    node_count = max(len(graph.nodes), 1)
    density = supported_edge_count / float(node_count * node_count)
    return ProfileMatrixSummary(
        profile=profile.name,
        relation_count=len(profile.allowed_relations),
        transition_count=len(profile.allowed_transitions),
        supported_edge_count=supported_edge_count,
        reachable_edge_count=reachable_edge_count,
        density=round(density, 6),
        relation_channels=relation_channels,
    )


def _relation_channels(
    allowed_relations: frozenset[str],
    oriented_edges: tuple[tuple[str, str, Edge], ...],
    reachable_node_ids: frozenset[str],
) -> tuple[RelationChannelSummary, ...]:
    supported_counts: dict[str, int] = {}
    reachable_counts: dict[str, int] = {}
    weight_mass: dict[str, float] = {}
    reachable_weight_mass: dict[str, float] = {}
    for source, target, edge in oriented_edges:
        relation = edge.relation
        supported_counts[relation] = supported_counts.get(relation, 0) + 1
        weight_mass[relation] = weight_mass.get(relation, 0.0) + edge.weight
        if source in reachable_node_ids and target in reachable_node_ids:
            reachable_counts[relation] = reachable_counts.get(relation, 0) + 1
            reachable_weight_mass[relation] = (
                reachable_weight_mass.get(relation, 0.0) + edge.weight
            )
    return tuple(
        RelationChannelSummary(
            relation=relation,
            supported_edge_count=supported_counts.get(relation, 0),
            reachable_edge_count=reachable_counts.get(relation, 0),
            weight_mass=round(weight_mass.get(relation, 0.0), 6),
            reachable_weight_mass=round(reachable_weight_mass.get(relation, 0.0), 6),
        )
        for relation in sorted(allowed_relations)
    )
