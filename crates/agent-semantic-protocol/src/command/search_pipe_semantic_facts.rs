//! Bounded semantic facts for ASP-owned search pipe graph requests.

use std::{collections::HashSet, fs};

use super::search_pipe_render::Candidate;

const GRAPH_TURBO_CANDIDATE_NODE_LIMIT: usize = 64;
const COLLECTION_FIELD_OWNER_SCAN_LIMIT: usize = 16;
const COLLECTION_FIELD_FACT_LIMIT: usize = 24;

#[derive(Debug, Clone)]
pub(super) struct CollectionFieldFact {
    pub(super) owner_path: String,
    pub(super) field_name: String,
    pub(super) type_name: String,
    pub(super) type_value: String,
    pub(super) type_args: String,
    pub(super) collection_kind: String,
    pub(super) line: usize,
    pub(super) text: String,
}

pub(super) fn collect_collection_field_facts(
    language_id: &str,
    query: Option<&str>,
    candidates: &[Candidate],
) -> Vec<CollectionFieldFact> {
    if language_id != "rust" || !query_has_data_shape_intent(query) {
        return Vec::new();
    }
    let mut seen_facts = HashSet::new();
    let mut scanned_paths = HashSet::new();
    let mut facts = Vec::new();
    for candidate in candidates.iter().take(GRAPH_TURBO_CANDIDATE_NODE_LIMIT) {
        if facts.len() >= COLLECTION_FIELD_FACT_LIMIT {
            break;
        }
        if scanned_paths.len() >= COLLECTION_FIELD_OWNER_SCAN_LIMIT {
            break;
        }
        if !scanned_paths.insert(candidate.path.clone()) {
            continue;
        }
        push_collection_field_facts(&candidate.path, query, &mut seen_facts, &mut facts);
    }
    facts
}

pub(super) fn collection_field_matches_query(fact: &CollectionFieldFact, query: &str) -> bool {
    let semantic_text = format!(
        "{} {} {} {} {} field fields type types collection collections list lists map maps set sets",
        fact.field_name,
        fact.type_name,
        fact.type_value,
        fact.type_args,
        fact.collection_kind,
    )
    .to_ascii_lowercase();
    query_terms_for_matching(query)
        .into_iter()
        .any(|term| semantic_text.contains(&term) || collection_alias_matches(&term, fact))
}

fn push_collection_field_facts(
    owner_path: &str,
    query: Option<&str>,
    seen_facts: &mut HashSet<String>,
    facts: &mut Vec<CollectionFieldFact>,
) {
    let Ok(source) = fs::read_to_string(owner_path) else {
        return;
    };
    for (line_index, line) in source.lines().enumerate() {
        if facts.len() >= COLLECTION_FIELD_FACT_LIMIT {
            break;
        }
        let line_number = line_index + 1;
        let Some(fact) = rust_collection_field_fact(owner_path, line_number, line) else {
            continue;
        };
        if query.is_some_and(|query| !collection_field_matches_query(&fact, query)) {
            continue;
        }
        let key = format!("{}:{}:{}", fact.owner_path, fact.field_name, fact.line);
        if seen_facts.insert(key) {
            facts.push(fact);
        }
    }
}

fn rust_collection_field_fact(
    owner_path: &str,
    line_number: usize,
    line: &str,
) -> Option<CollectionFieldFact> {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with("impl ")
        || trimmed.starts_with("let ")
        || trimmed.contains("fn ")
        || trimmed.contains("->")
        || trimmed.contains('{')
        || trimmed.contains('}')
        || trimmed.contains("=>")
        || !trimmed.ends_with(',')
    {
        return None;
    }
    let colon = field_type_colon(trimmed)?;
    let field_part = trimmed[..colon].trim();
    if field_part.contains('(')
        || field_part.contains(')')
        || field_part.contains('=')
        || field_part.contains('<')
        || field_part.contains('>')
    {
        return None;
    }
    let field_name = last_identifier(field_part)?;
    if matches!(
        field_name.as_str(),
        "pub" | "crate" | "super" | "self" | "where"
    ) {
        return None;
    }
    let type_value = trim_rust_type_text(&trimmed[colon + 1..])?;
    if type_value.starts_with(':') || type_value.contains('{') || type_value.contains('}') {
        return None;
    }
    let collection_kind = rust_collection_kind(&type_value)?;
    let type_args = rust_type_args(&type_value).unwrap_or_default();
    Some(CollectionFieldFact {
        owner_path: owner_path.to_string(),
        field_name,
        type_name: collection_kind.clone(),
        type_value,
        type_args,
        collection_kind,
        line: line_number,
        text: trimmed.to_string(),
    })
}

fn field_type_colon(trimmed: &str) -> Option<usize> {
    let bytes = trimmed.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b':' {
            continue;
        }
        if index > 0 && bytes[index - 1] == b':' {
            continue;
        }
        if bytes.get(index + 1).is_some_and(|next| *next == b':') {
            continue;
        }
        return Some(index);
    }
    None
}

fn trim_rust_type_text(raw: &str) -> Option<String> {
    let without_comment = raw.split("//").next().unwrap_or(raw);
    let before_initializer = without_comment.split('=').next().unwrap_or(without_comment);
    let trimmed = before_initializer.trim().trim_end_matches(',').trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn rust_collection_kind(type_value: &str) -> Option<String> {
    const COLLECTION_TYPES: &[&str] = &[
        "VecDeque", "BTreeMap", "BTreeSet", "HashMap", "HashSet", "Vec",
    ];
    let compact: String = type_value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    COLLECTION_TYPES
        .iter()
        .find(|kind| {
            compact.starts_with(&format!("{kind}<"))
                || compact.contains(&format!("::{kind}<"))
                || compact.contains(&format!("{kind}<"))
        })
        .map(|kind| (*kind).to_string())
}

fn rust_type_args(type_value: &str) -> Option<String> {
    let start = type_value.find('<')?;
    let end = type_value.rfind('>')?;
    (end > start + 1).then(|| type_value[start + 1..end].trim().to_string())
}

fn last_identifier(value: &str) -> Option<String> {
    let trimmed = value.trim_end();
    let mut end = trimmed.len();
    while end > 0 {
        let character = trimmed[..end].chars().next_back()?;
        if character == '_' || character.is_ascii_alphanumeric() {
            break;
        }
        end -= character.len_utf8();
    }
    let mut start = end;
    while start > 0 {
        let character = trimmed[..start].chars().next_back()?;
        if !(character == '_' || character.is_ascii_alphanumeric()) {
            break;
        }
        start -= character.len_utf8();
    }
    let identifier = &trimmed[start..end];
    (!identifier.is_empty()).then(|| identifier.to_string())
}

fn query_has_data_shape_intent(query: Option<&str>) -> bool {
    let Some(query) = query else {
        return false;
    };
    query_terms_for_matching(query).into_iter().any(|term| {
        matches!(
            term.as_str(),
            "field"
                | "fields"
                | "type"
                | "types"
                | "collection"
                | "collections"
                | "list"
                | "lists"
                | "map"
                | "maps"
                | "set"
                | "sets"
                | "vec"
                | "vecdeque"
                | "hashmap"
                | "hashset"
                | "btreemap"
                | "btreeset"
        )
    })
}

fn collection_alias_matches(term: &str, fact: &CollectionFieldFact) -> bool {
    match term {
        "collection" | "collections" | "list" | "lists" => true,
        "map" | "maps" => fact.collection_kind.ends_with("Map"),
        "set" | "sets" => fact.collection_kind.ends_with("Set"),
        "field" | "fields" | "type" | "types" => true,
        _ => false,
    }
}

fn query_terms_for_matching(query: &str) -> Vec<String> {
    query
        .split(|character: char| !(character == '_' || character.is_ascii_alphanumeric()))
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}
