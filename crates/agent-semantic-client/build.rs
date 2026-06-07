use rust_lang_project_harness::{
    assert_rust_project_harness_cargo_check_clean_from_env_with_config,
    assert_rust_project_harness_performance_verification_from_env, default_rust_harness_config,
};

fn main() {
    let config = default_rust_harness_config()
        .with_cargo_check_advice_allow_explanation(
            "agent-semantic-client keeps advisory findings visible while the build gate blocks warning and error policy drift",
        )
        .with_criterion_performance_verification()
        .with_latency_sensitive_performance_owner(
            "src/provider_method.rs",
            "provider method dispatch owns cache-hit, packet-first, and provider-exec latency",
        )
        .with_latency_sensitive_performance_owner(
            "src/cache_replay/search_packet.rs",
            "search packet replay renders compact stdout on the cache hot path",
        )
        .with_latency_sensitive_performance_owner(
            "src/search_history.rs",
            "search history audit uses sqlite-backed artifact timelines and graph-turbo dispatch",
        );
    assert_rust_project_harness_cargo_check_clean_from_env_with_config(&config);
    assert_rust_project_harness_performance_verification_from_env(&config, "client");
}
