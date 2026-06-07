fn main() {
    let config = rust_lang_project_harness::RustHarnessConfig {
        cargo_check_advice_allow_explanation: Some(
            "runtime state materialization keeps filesystem side effects in a focused crate while config remains the read-only layout source"
                .to_string(),
        ),
        ..Default::default()
    };
    rust_lang_project_harness::assert_rust_project_harness_cargo_check_clean_from_env_with_config(
        &config,
    );
}
