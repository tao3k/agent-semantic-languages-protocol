use agent_semantic_hook::RuntimeProviderHealthStatus;

use crate::{LanguageId, ProviderExecution, ProviderId, ResolvedProvider, RuntimeProfileStatus};

#[test]
fn runtime_profile_status_preserves_receipt_labels() {
    assert_eq!(RuntimeProfileStatus::Available.as_str(), "available");
    assert_eq!(RuntimeProfileStatus::Missing.as_str(), "missing");
    assert_eq!(RuntimeProfileStatus::Unexecutable.as_str(), "unexecutable");
}

#[test]
fn runtime_profile_status_maps_from_hook_health_status() {
    assert_eq!(
        RuntimeProfileStatus::from(RuntimeProviderHealthStatus::Available),
        RuntimeProfileStatus::Available
    );
    assert_eq!(
        RuntimeProfileStatus::from(RuntimeProviderHealthStatus::Missing),
        RuntimeProfileStatus::Missing
    );
    assert_eq!(
        RuntimeProfileStatus::from(RuntimeProviderHealthStatus::Unexecutable),
        RuntimeProfileStatus::Unexecutable
    );
}

#[test]
fn activation_provider_prefix_takes_precedence_over_runtime_profile_argv() {
    let provider = ResolvedProvider {
        language_id: LanguageId::from("rust"),
        provider_id: ProviderId::from("rs-harness"),
        binary: "rs-harness".to_string(),
        execution: ProviderExecution::ExternalProcess,
        provider_command_prefix: vec!["./.bin/rs-harness".to_string()],
        runtime_command_argv: Some(vec!["/opt/homebrew/bin/rs-harness".to_string()]),
        runtime_profile_status: Some(RuntimeProfileStatus::Available),
        package_roots: vec![".".to_string()],
    };

    assert_eq!(
        provider.command_prefix(),
        vec!["./.bin/rs-harness".to_string()]
    );
    assert_eq!(provider.runtime_command_prefix(), None);
}
