//! Optional client-side hook rules loaded on each hook invocation.

use agent_semantic_config::{
    HookClientConfigDecision, HookClientConfigFile, HookClientConfigReasonKind,
    HookClientConfigRouteKind, HookClientConfigStdinMode, HookClientRuleConfig,
    HookClientRuleMatchConfig, HookClientRuleRouteConfig, default_hook_client_config_path,
    default_hook_client_config_template, load_hook_client_config_file,
};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};

use crate::command::semantic_shell_tokens;
use crate::protocol::{
    DecisionKind, DecisionRoute, DecisionRouteKind, HOOK_DECISION_SCHEMA_ID,
    HOOK_DECISION_SCHEMA_VERSION, HOOK_PROTOCOL_ID, HOOK_PROTOCOL_VERSION, HookDecision,
    ReasonKind, StdinMode,
};
use crate::protocol_activation::HookRuntime;
use crate::source_selector::collect_source_selector_matches;
use crate::tool_action::{ToolAction, subject_for_action};

#[derive(Debug, Default)]
/// Compiled project-local hook rules loaded from `.codex/agent-semantic-protocol`.
pub struct ClientHookConfig {
    rules: Vec<CompiledHookRule>,
    semantic_ast_patch_disabled: bool,
}

#[derive(Debug)]
struct CompiledHookRule {
    id: String,
    priority: i64,
    decision: HookClientConfigDecision,
    reason_kind: ReasonKind,
    message: Option<String>,
    language_ids: Vec<String>,
    event: Option<String>,
    platform: Option<String>,
    match_config: RuleMatch,
    routes: Vec<RuleRoute>,
}

#[derive(Debug)]
struct RuleMatch {
    tool_any: Vec<String>,
    command_any: Vec<String>,
    command_contains_any: Vec<String>,
    path_any: Vec<String>,
    path_glob_any: Option<GlobSet>,
}

#[derive(Debug)]
struct RuleRoute {
    provider_id: String,
    language_id: Option<String>,
    binary: Option<String>,
    kind: DecisionRouteKind,
    argv: Vec<String>,
    stdin_mode: Option<StdinMode>,
}

/// Return the versioned project-local hook config path.
pub fn default_client_config_path(project_root: &str) -> PathBuf {
    default_hook_client_config_path(project_root)
}

/// Render the seed project-local hook config file.
pub fn default_client_config_template() -> String {
    default_hook_client_config_template()
}

/// Load and compile project-local hook config rules.
pub fn load_client_config(path: &Path) -> Result<ClientHookConfig, String> {
    let parsed = load_hook_client_config_file(path)?;
    compile_config(parsed)
}

impl ClientHookConfig {
    pub(crate) fn semantic_ast_patch_enabled(&self) -> bool {
        !self.semantic_ast_patch_disabled
    }

    pub(crate) fn classify(
        &self,
        runtime: &HookRuntime,
        platform: &str,
        event: &str,
        action: &ToolAction,
    ) -> Option<HookDecision> {
        self.rules
            .iter()
            .find(|rule| rule.matches(runtime, platform, event, action))
            .map(|rule| rule.decision(runtime, platform, event, action))
    }
}

impl CompiledHookRule {
    fn matches(
        &self,
        runtime: &HookRuntime,
        platform: &str,
        event: &str,
        action: &ToolAction,
    ) -> bool {
        self.platform
            .as_deref()
            .is_none_or(|expected| expected.eq_ignore_ascii_case(platform))
            && self
                .event
                .as_deref()
                .is_none_or(|expected| canonical_event(expected) == canonical_event(event))
            && self.matches_language(runtime, action)
            && self.match_config.matches(action)
    }

    fn matches_language(&self, runtime: &HookRuntime, action: &ToolAction) -> bool {
        if self.language_ids.is_empty() {
            return true;
        }
        if action.paths.is_empty() {
            return false;
        }
        !collect_source_selector_matches(
            runtime,
            action.paths.iter().map(String::as_str),
            |provider| {
                self.language_ids
                    .iter()
                    .any(|language_id| language_id.eq_ignore_ascii_case(&provider.language_id))
            },
        )
        .is_empty()
    }

    fn decision(
        &self,
        runtime: &HookRuntime,
        platform: &str,
        event: &str,
        action: &ToolAction,
    ) -> HookDecision {
        let decision = match self.decision {
            HookClientConfigDecision::Block => DecisionKind::Block,
            HookClientConfigDecision::Deny => DecisionKind::Deny,
        };
        let routes = self
            .routes
            .iter()
            .map(|route| route.decision_route(runtime))
            .collect::<Vec<_>>();
        let message = self.message.clone().unwrap_or_else(|| {
            format!(
                "client hook config rule `{}` matched this tool use",
                self.id
            )
        });
        HookDecision {
            schema_id: HOOK_DECISION_SCHEMA_ID,
            schema_version: HOOK_DECISION_SCHEMA_VERSION,
            protocol_id: HOOK_PROTOCOL_ID,
            protocol_version: HOOK_PROTOCOL_VERSION,
            platform: platform.to_string(),
            event: event.to_string(),
            decision,
            reason_kind: self.reason_kind,
            language_ids: self.language_ids.clone(),
            subject: subject_for_action(action),
            routes,
            message,
            fields: std::collections::BTreeMap::from([(
                "configRuleId".to_string(),
                serde_json::Value::String(self.id.clone()),
            )]),
        }
    }
}

impl RuleMatch {
    fn matches(&self, action: &ToolAction) -> bool {
        self.matches_tool(action) && self.matches_command(action) && self.matches_path(action)
    }

    fn matches_tool(&self, action: &ToolAction) -> bool {
        self.tool_any.is_empty()
            || self
                .tool_any
                .iter()
                .any(|tool| tool.eq_ignore_ascii_case(&action.tool_name))
    }

    fn matches_command(&self, action: &ToolAction) -> bool {
        let Some(command) = action.command.as_deref() else {
            return self.command_any.is_empty() && self.command_contains_any.is_empty();
        };
        let tokens = semantic_shell_tokens(command);
        let token_match = self.command_any.is_empty()
            || self.command_any.iter().any(|expected| {
                tokens
                    .iter()
                    .any(|token| token.eq_ignore_ascii_case(expected))
            });
        let contains_match = self.command_contains_any.is_empty()
            || self
                .command_contains_any
                .iter()
                .any(|expected| command.contains(expected));
        token_match && contains_match
    }

    fn matches_path(&self, action: &ToolAction) -> bool {
        if self.path_any.is_empty() && self.path_glob_any.is_none() {
            return true;
        }
        let exact_match = !self.path_any.is_empty()
            && action.paths.iter().any(|path| {
                self.path_any
                    .iter()
                    .any(|expected| path == expected || path.ends_with(expected))
            });
        let glob_match = self.path_glob_any.as_ref().is_some_and(|globset| {
            action
                .paths
                .iter()
                .any(|path| globset.is_match(path.as_str()))
        });
        exact_match || glob_match
    }
}

impl RuleRoute {
    fn decision_route(&self, runtime: &HookRuntime) -> DecisionRoute {
        let provider = runtime
            .providers
            .iter()
            .find(|provider| provider.provider_id == self.provider_id);
        DecisionRoute {
            language_id: self
                .language_id
                .clone()
                .or_else(|| provider.map(|provider| provider.language_id.clone()))
                .unwrap_or_default(),
            provider_id: self.provider_id.clone(),
            binary: self
                .binary
                .clone()
                .or_else(|| provider.map(|provider| provider.binary.clone()))
                .unwrap_or_default(),
            kind: self.kind,
            argv: self.argv.clone(),
            stdin_mode: self.stdin_mode,
        }
    }
}

fn compile_config(config: HookClientConfigFile) -> Result<ClientHookConfig, String> {
    let semantic_ast_patch_enabled = config
        .experimental
        .get("semanticAstPatch")
        .and_then(|feature| feature.get("enabled"))
        .copied()
        .unwrap_or(true);
    let mut rules = config
        .rules
        .into_iter()
        .filter(|rule| rule.enabled)
        .map(CompiledHookRule::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    // `sort_by_key` is stable, so equal-priority rules keep config file order.
    rules.sort_by_key(|rule| std::cmp::Reverse(rule.priority));
    Ok(ClientHookConfig {
        rules,
        semantic_ast_patch_disabled: !semantic_ast_patch_enabled,
    })
}

impl TryFrom<HookClientRuleConfig> for CompiledHookRule {
    type Error = String;

    fn try_from(config: HookClientRuleConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            id: config.id,
            priority: config.priority,
            decision: config.decision,
            reason_kind: config
                .reason_kind
                .map(ReasonKind::from)
                .unwrap_or(ReasonKind::None),
            message: config.message,
            language_ids: config.language_ids,
            event: config.event,
            platform: config.platform,
            match_config: RuleMatch::try_from(config.match_config)?,
            routes: config
                .routes
                .into_iter()
                .map(RuleRoute::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<HookClientRuleMatchConfig> for RuleMatch {
    type Error = String;

    fn try_from(config: HookClientRuleMatchConfig) -> Result<Self, Self::Error> {
        let mut tool_any = config.tool_any;
        if let Some(tool) = config.tool {
            tool_any.push(tool);
        }
        Ok(Self {
            tool_any,
            command_any: config.command_any,
            command_contains_any: config.command_contains_any,
            path_any: config.path_any,
            path_glob_any: compile_globs(config.path_glob_any)?,
        })
    }
}

impl TryFrom<HookClientRuleRouteConfig> for RuleRoute {
    type Error = String;

    fn try_from(config: HookClientRuleRouteConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            provider_id: config.provider_id,
            language_id: config.language_id,
            binary: config.binary,
            kind: DecisionRouteKind::from(config.kind),
            argv: config.argv,
            stdin_mode: config.stdin_mode.map(StdinMode::from),
        })
    }
}

impl From<HookClientConfigReasonKind> for ReasonKind {
    fn from(kind: HookClientConfigReasonKind) -> Self {
        match kind {
            HookClientConfigReasonKind::None => Self::None,
            HookClientConfigReasonKind::DirectSourceRead => Self::DirectSourceRead,
            HookClientConfigReasonKind::BulkSourceDump => Self::BulkSourceDump,
            HookClientConfigReasonKind::RawBroadSearch => Self::RawBroadSearch,
            HookClientConfigReasonKind::AgentSearchJson => Self::AgentSearchJson,
            HookClientConfigReasonKind::SubagentReceiptRequired => Self::SubagentReceiptRequired,
        }
    }
}

impl From<HookClientConfigRouteKind> for DecisionRouteKind {
    fn from(kind: HookClientConfigRouteKind) -> Self {
        match kind {
            HookClientConfigRouteKind::Prime => Self::Prime,
            HookClientConfigRouteKind::Owner => Self::Owner,
            HookClientConfigRouteKind::Query => Self::Query,
            HookClientConfigRouteKind::Fzf => Self::Fzf,
            HookClientConfigRouteKind::Read => Self::Read,
            HookClientConfigRouteKind::Deps => Self::Deps,
            HookClientConfigRouteKind::Api => Self::Api,
            HookClientConfigRouteKind::Ingest => Self::Ingest,
            HookClientConfigRouteKind::Tests => Self::Tests,
            HookClientConfigRouteKind::CheckChanged => Self::CheckChanged,
        }
    }
}

impl From<HookClientConfigStdinMode> for StdinMode {
    fn from(mode: HookClientConfigStdinMode) -> Self {
        match mode {
            HookClientConfigStdinMode::None => Self::None,
            HookClientConfigStdinMode::PipeCandidates => Self::PipeCandidates,
            HookClientConfigStdinMode::PipeDiff => Self::PipeDiff,
            HookClientConfigStdinMode::Unknown => Self::Unknown,
        }
    }
}

fn compile_globs(patterns: Vec<String>) -> Result<Option<GlobSet>, String> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(&pattern)
            .literal_separator(true)
            .build()
            .map_err(|error| format!("invalid pathGlobAny pattern `{pattern}`: {error}"))?;
        builder.add(glob);
    }
    builder
        .build()
        .map(Some)
        .map_err(|error| format!("failed to compile pathGlobAny patterns: {error}"))
}

fn canonical_event(value: &str) -> String {
    value.to_ascii_lowercase().replace('_', "-")
}
