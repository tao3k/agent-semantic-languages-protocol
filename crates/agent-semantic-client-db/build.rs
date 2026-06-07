use rust_lang_project_harness::{
    assert_rust_project_harness_cargo_check_clean_from_env_with_config,
    assert_rust_project_harness_performance_verification_from_env, default_rust_harness_config,
};

fn main() {
    let config = default_rust_harness_config()
        .with_cargo_check_advice_allow_explanation(
            "agent-semantic-client-db keeps advisory findings visible while the build gate blocks warning and error policy drift",
        )
        .with_criterion_performance_verification()
        .with_latency_sensitive_performance_owner(
            "src/db.rs",
            "sqlite cache lookups and artifact timeline writes sit on provider replay hot paths",
        )
        .with_latency_sensitive_performance_owner(
            "src/pragmas.rs",
            "sqlite runtime pragmas control cache query latency under repeated agent searches",
        );
    assert_rust_project_harness_cargo_check_clean_from_env_with_config(&config);
    assert_rust_project_harness_performance_verification_from_env(&config, "client db");
}
