//! Graph-turbo request packets for ASP-owned fast search candidates.

use serde_json::{Value, json};

use super::search_pipe_render::Candidate;

const GRAPH_TURBO_REQUEST_SCHEMA_ID: &str = "agent.semantic-protocols.semantic-graph-turbo-request";

pub(super) fn render_graph_turbo_request(
    query: Option<&str>,
    candidates: &[Candidate],
    pipes: &[String],
) -> Result<String, String> {
    let packet = graph_turbo_request(query, candidates, pipes);
    serde_json::to_string_pretty(&packet)
        .map(|mut text| {
            text.push('\n');
            text
        })
        .map_err(|error| format!("failed to serialize graph turbo request: {error}"))
}

fn graph_turbo_request(query: Option<&str>, candidates: &[Candidate], pipes: &[String]) -> Value {
    let profile = profile_for_pipes(pipes);
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seed_ids = Vec::new();
    if let Some(query) = query.filter(|query| !query.trim().is_empty()) {
        let query_id = stable_node_id("query", query);
        seed_ids.push(query_id.clone());
        nodes.push(json!({
            "id": query_id,
            "kind": "query",
            "role": "term",
            "value": query,
            "action": "fzf"
        }));
    }

    let owners = unique_candidate_paths(candidates);
    if seed_ids.is_empty() {
        seed_ids.extend(
            owners
                .iter()
                .take(2)
                .map(|owner| stable_node_id("owner", owner)),
        );
    }
    append_owner_nodes(&mut nodes, &owners);
    append_candidate_nodes(&mut nodes, candidates);
    append_test_nodes(&mut nodes, &owners, pipes);
    append_graph_edges(&mut edges, query, candidates, &owners, pipes);

    json!({
        "schemaId": GRAPH_TURBO_REQUEST_SCHEMA_ID,
        "schemaVersion": "1",
        "protocolId": "agent.semantic-protocols.semantic-language",
        "protocolVersion": "1",
        "packetKind": "graph-turbo-request",
        "profile": profile,
        "algorithm": "typed-ppr-diverse",
        "seedIds": seed_ids,
        "budget": 10,
        "kindBudgets": {"owner": 4, "dependency": 2, "test": 3, "item": 6, "hot": 2},
        "windowMerge": {"enabled": true, "maxGapLines": 8},
        "pathBudget": 5,
        "pathMaxHops": 4,
        "cache": {"enabled": true},
        "graph": {
            "nodes": nodes,
            "edges": edges,
        },
    })
}

fn append_owner_nodes(nodes: &mut Vec<Value>, owners: &[String]) {
    for owner in owners {
        nodes.push(json!({
            "id": stable_node_id("owner", owner),
            "kind": "owner",
            "role": "path",
            "value": owner,
            "action": "owner",
            "path": owner
        }));
    }
}

fn append_candidate_nodes(nodes: &mut Vec<Value>, candidates: &[Candidate]) {
    for candidate in candidates.iter().take(12) {
        nodes.push(json!({
            "id": candidate_node_id(candidate),
            "kind": "item",
            "role": "symbol",
            "value": candidate.symbol,
            "action": "code",
            "path": candidate.path,
            "ownerPath": candidate.path,
            "symbol": candidate.symbol,
            "startLine": candidate.line,
            "endLine": candidate.line,
            "locator": format!("{}:{}:{}", candidate.path, candidate.line, candidate.line),
        }));
    }
}

fn append_test_nodes(nodes: &mut Vec<Value>, owners: &[String], pipes: &[String]) {
    if !include_tests(pipes) {
        return;
    }
    for owner in owners {
        nodes.push(json!({
            "id": stable_node_id("test", owner),
            "kind": "test",
            "role": "path",
            "value": owner,
            "action": "tests",
            "path": owner
        }));
    }
}

fn append_graph_edges(
    edges: &mut Vec<Value>,
    query: Option<&str>,
    candidates: &[Candidate],
    owners: &[String],
    pipes: &[String],
) {
    if let Some(query) = query.filter(|query| !query.trim().is_empty()) {
        append_query_match_edges(edges, query, candidates, owners);
    }
    append_owner_candidate_edges(edges, candidates);
    append_test_cover_edges(edges, owners, pipes);
}

fn append_query_match_edges(
    edges: &mut Vec<Value>,
    query: &str,
    candidates: &[Candidate],
    owners: &[String],
) {
    let query_id = stable_node_id("query", query);
    for owner in owners {
        edges.push(edge(&query_id, &stable_node_id("owner", owner), "matches"));
    }
    for candidate in candidates.iter().take(12) {
        edges.push(edge(&query_id, &candidate_node_id(candidate), "matches"));
    }
}

fn append_owner_candidate_edges(edges: &mut Vec<Value>, candidates: &[Candidate]) {
    for candidate in candidates.iter().take(12) {
        edges.push(edge(
            &stable_node_id("owner", &candidate.path),
            &candidate_node_id(candidate),
            "contains",
        ));
    }
}

fn append_test_cover_edges(edges: &mut Vec<Value>, owners: &[String], pipes: &[String]) {
    if include_tests(pipes) {
        for owner in owners {
            edges.push(edge(
                &stable_node_id("owner", owner),
                &stable_node_id("test", owner),
                "covers",
            ));
        }
    }
}

fn edge(source: &str, target: &str, relation: &str) -> Value {
    json!({
        "source": source,
        "target": target,
        "relation": relation,
    })
}

fn unique_candidate_paths(candidates: &[Candidate]) -> Vec<String> {
    candidates.iter().fold(Vec::new(), |mut paths, candidate| {
        if !paths.contains(&candidate.path) {
            paths.push(candidate.path.clone());
        }
        paths
    })
}

fn include_tests(pipes: &[String]) -> bool {
    pipes.is_empty() || pipes.iter().any(|pipe| pipe == "tests")
}

fn profile_for_pipes(pipes: &[String]) -> &'static str {
    if pipes
        .iter()
        .any(|pipe| matches!(pipe.as_str(), "deps" | "dependencies"))
    {
        "query-deps"
    } else if pipes.iter().any(|pipe| pipe == "tests")
        && !pipes
            .iter()
            .any(|pipe| matches!(pipe.as_str(), "items" | "owner"))
    {
        "owner-tests"
    } else {
        "owner-query"
    }
}

fn candidate_node_id(candidate: &Candidate) -> String {
    stable_node_id(
        "item",
        &format!("{}:{}:{}", candidate.path, candidate.symbol, candidate.line),
    )
}

fn stable_node_id(kind: &str, value: &str) -> String {
    let mut rendered = String::with_capacity(kind.len() + value.len() + 1);
    rendered.push_str(kind);
    rendered.push(':');
    for character in value.chars() {
        if character == '_' || character == '-' || character == '/' || character == '.' {
            rendered.push(character);
        } else if character.is_ascii_alphanumeric() {
            rendered.push(character.to_ascii_lowercase());
        } else {
            rendered.push('-');
        }
    }
    while rendered.ends_with('-') {
        rendered.pop();
    }
    if rendered.len() == kind.len() + 1 {
        rendered.push_str("node");
    }
    rendered
}
