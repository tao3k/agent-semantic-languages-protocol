use agent_semantic_client_core::{ClientMethod, ClientRequest};

use crate::provider_method::should_try_search_packet_first;

#[test]
fn search_packet_first_skips_workspace_seed_discovery() {
    let request = ClientRequest::new(ClientMethod::Search, ".").with_forwarded_args(vec![
        "workspace".to_string(),
        "--view".to_string(),
        "seeds".to_string(),
    ]);

    assert!(!should_try_search_packet_first(&request));
}

#[test]
fn search_packet_first_still_handles_seed_fzf() {
    let request = ClientRequest::new(ClientMethod::Search, ".").with_forwarded_args(vec![
        "fzf".to_string(),
        "workspace".to_string(),
        "--view".to_string(),
        "seeds".to_string(),
    ]);

    assert!(should_try_search_packet_first(&request));
}
