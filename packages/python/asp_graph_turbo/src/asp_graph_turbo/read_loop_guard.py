"""Read-loop guard projection for graph turbo frontier results."""

from __future__ import annotations

from collections import Counter
from dataclasses import dataclass

from .model import FrontierEntry, GraphProfile, Node, ReadLoopGuard
from .profiles import frontier_action
from .selector import (
    GraphTurboSelectorRange,
    graph_turbo_node_range,
    graph_turbo_owner_path_for_node,
    graph_turbo_parse_selector,
    graph_turbo_selector_for_node,
)


@dataclass(frozen=True)
class _ReadCandidate:
    selector: str
    owner_path: str
    node_range: GraphTurboSelectorRange | None


@dataclass(frozen=True)
class GraphTurboReadLoopSecondPass:
    candidate_count: int = 0
    duplicate_selector_suppressed_count: int = 0
    same_owner_suppressed_count: int = 0

    @property
    def suppressed_count(self) -> int:
        return (
            self.duplicate_selector_suppressed_count + self.same_owner_suppressed_count
        )


EMPTY_READ_LOOP_SECOND_PASS = GraphTurboReadLoopSecondPass()


def graph_turbo_apply_read_loop_second_pass(
    profile: GraphProfile,
    ranked: tuple[Node, ...],
    *,
    limit: int,
) -> tuple[tuple[Node, ...], GraphTurboReadLoopSecondPass]:
    kept: list[Node] = []
    owner_deferred: list[Node] = []
    seen_selectors: set[str] = set()
    owner_code_counts: Counter[str] = Counter()
    duplicate_selector_suppressed_count = 0
    for node in ranked:
        if len(kept) >= limit:
            break
        if frontier_action(profile, node) != "code":
            kept.append(node)
            continue
        selector = graph_turbo_selector_for_node(node)
        if selector is not None and selector in seen_selectors:
            duplicate_selector_suppressed_count += 1
            continue
        owner_path = graph_turbo_owner_path_for_node(node)
        if owner_path is not None and owner_code_counts[owner_path] >= 2:
            owner_deferred.append(node)
            continue
        _append_code_candidate(
            kept,
            node,
            seen_selectors=seen_selectors,
            owner_code_counts=owner_code_counts,
            selector=selector,
            owner_path=owner_path,
        )
    owner_restored_count = 0
    for node in owner_deferred:
        if len(kept) >= limit:
            break
        owner_restored_count += 1
        kept.append(node)
    return (
        tuple(kept[:limit]),
        GraphTurboReadLoopSecondPass(
            candidate_count=len(ranked),
            duplicate_selector_suppressed_count=duplicate_selector_suppressed_count,
            same_owner_suppressed_count=len(owner_deferred) - owner_restored_count,
        ),
    )


def evaluate_read_loop_guard(
    frontier: tuple[FrontierEntry, ...], *, max_gap_lines: int
) -> ReadLoopGuard:
    candidates = tuple(
        candidate
        for entry in frontier
        if entry.action == "code"
        if (candidate := _candidate_from_node(entry.node)) is not None
    )
    selector_counts = Counter(candidate.selector for candidate in candidates)
    duplicate_selector_count = sum(
        count - 1 for count in selector_counts.values() if count > 1
    )
    unique_candidates = tuple(
        {candidate.selector: candidate for candidate in candidates}.values()
    )
    adjacent_range_window_count = _adjacent_range_window_count(
        unique_candidates, max_gap_lines=max_gap_lines
    )
    owner_counts = Counter(candidate.owner_path for candidate in unique_candidates)
    same_owner_scan_count = sum(
        count - 1 for count in owner_counts.values() if count >= 3
    )
    avoid: list[str] = []
    if duplicate_selector_count:
        avoid.append("duplicate-read")
    if adjacent_range_window_count:
        avoid.append("manual-window-scan")
    if same_owner_scan_count:
        avoid.append("repeat-owner")
    return ReadLoopGuard(
        direct_code_action_count=len(candidates),
        duplicate_selector_count=duplicate_selector_count,
        adjacent_range_window_count=adjacent_range_window_count,
        same_owner_scan_count=same_owner_scan_count,
        avoid=tuple(avoid),
    )


def _candidate_from_node(node: Node) -> _ReadCandidate | None:
    selector = graph_turbo_selector_for_node(node)
    if selector is None:
        return None
    locator = graph_turbo_parse_selector(selector) or graph_turbo_node_range(node)
    owner_path = graph_turbo_owner_path_for_node(node) or (
        locator.path if locator is not None else ""
    )
    if not owner_path:
        owner_path = selector
    return _ReadCandidate(
        selector=selector,
        owner_path=owner_path,
        node_range=locator,
    )


def _adjacent_range_window_count(
    candidates: tuple[_ReadCandidate, ...], *, max_gap_lines: int
) -> int:
    ranges_by_path: dict[str, list[GraphTurboSelectorRange]] = {}
    for candidate in candidates:
        if candidate.node_range is None:
            continue
        ranges_by_path.setdefault(candidate.node_range.path, []).append(
            candidate.node_range
        )
    adjacent_count = 0
    for ranges in ranges_by_path.values():
        ranges.sort(key=lambda item: (item.start_line, item.end_line))
        previous_end: int | None = None
        for node_range in ranges:
            if (
                previous_end is not None
                and node_range.start_line <= previous_end + max_gap_lines
            ):
                adjacent_count += 1
            previous_end = max(previous_end or node_range.end_line, node_range.end_line)
    return adjacent_count


def _append_code_candidate(
    kept: list[Node],
    node: Node,
    *,
    seen_selectors: set[str],
    owner_code_counts: Counter[str],
    selector: str | None,
    owner_path: str | None,
) -> None:
    kept.append(node)
    if selector is not None:
        seen_selectors.add(selector)
    if owner_path is not None:
        owner_code_counts[owner_path] += 1
