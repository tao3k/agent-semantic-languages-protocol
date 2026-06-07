mod config;
mod fzf;
mod ingest;
mod pipe_frontier;
mod reasoning;
mod suggest;

use serde_json::Value;

fn assert_graph_turbo_request_contract(payload: &Value) {
    assert_eq!(
        payload["schemaId"],
        "agent.semantic-protocols.semantic-graph-turbo-request"
    );
    assert_eq!(payload["schemaVersion"], "1");
    assert_eq!(
        payload["protocolId"],
        "agent.semantic-protocols.semantic-language"
    );
    assert_eq!(payload["protocolVersion"], "1");
    assert_eq!(payload["packetKind"], "graph-turbo-request");
    assert_eq!(payload["algorithm"], "typed-ppr-diverse");
    assert!(
        payload["profile"]
            .as_str()
            .is_some_and(|profile| !profile.is_empty()),
        "profile must be a non-empty string: {payload}"
    );
    assert!(
        payload["budget"].as_u64().is_some_and(|budget| budget > 0),
        "budget must be a positive integer: {payload}"
    );
    assert!(
        payload["seedIds"]
            .as_array()
            .is_some_and(|seed_ids| !seed_ids.is_empty()
                && seed_ids
                    .iter()
                    .all(|seed_id| seed_id.as_str().is_some_and(|seed_id| !seed_id.is_empty()))),
        "seedIds must be non-empty strings: {payload}"
    );

    let graph = payload["graph"].as_object().expect("graph object");
    assert!(
        graph
            .keys()
            .all(|key| matches!(key.as_str(), "nodes" | "edges")),
        "graph contains schema-unknown keys: {graph:?}"
    );
    let nodes = graph
        .get("nodes")
        .and_then(Value::as_array)
        .expect("graph.nodes array");
    let edges = graph
        .get("edges")
        .and_then(Value::as_array)
        .expect("graph.edges array");
    assert!(!nodes.is_empty(), "graph.nodes should not be empty");

    for node in nodes {
        let node = node.as_object().expect("node object");
        assert!(
            node.keys().all(|key| matches!(
                key.as_str(),
                "id" | "kind"
                    | "role"
                    | "value"
                    | "action"
                    | "weight"
                    | "locator"
                    | "location"
                    | "path"
                    | "owner"
                    | "ownerPath"
                    | "symbol"
                    | "name"
                    | "startLine"
                    | "endLine"
                    | "start"
                    | "end"
                    | "fields"
            )),
            "node contains schema-unknown keys: {node:?}"
        );
        for field in ["id", "kind", "role", "value"] {
            assert!(
                node.get(field)
                    .and_then(Value::as_str)
                    .is_some_and(|value| !value.is_empty()),
                "node.{field} must be a non-empty string: {node:?}"
            );
        }
    }

    for edge in edges {
        let edge = edge.as_object().expect("edge object");
        assert!(
            edge.keys().all(|key| matches!(
                key.as_str(),
                "source" | "target" | "relation" | "weight" | "fields"
            )),
            "edge contains schema-unknown keys: {edge:?}"
        );
        for field in ["source", "target", "relation"] {
            assert!(
                edge.get(field)
                    .and_then(Value::as_str)
                    .is_some_and(|value| !value.is_empty()),
                "edge.{field} must be a non-empty string: {edge:?}"
            );
        }
    }
}
