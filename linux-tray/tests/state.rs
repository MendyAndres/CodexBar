use codexbar_linux_tray::{
    ProviderState, TrayHealth, apply_backend_failure, apply_edit, apply_provider_failure,
    apply_sni_failure,
};
use std::time::{Duration, SystemTime};

fn provider(id: &str) -> ProviderState {
    ProviderState::connected(id, SystemTime::UNIX_EPOCH)
}

#[test]
fn backend_disconnect_ages_every_last_good_without_provider_degradation() {
    let mut providers = vec![provider("codex"), provider("claude")];
    apply_backend_failure(
        &mut providers,
        SystemTime::UNIX_EPOCH + Duration::from_secs(31),
        Duration::from_secs(30),
    );
    assert!(
        providers
            .iter()
            .all(|provider| provider.disconnected && provider.stale && !provider.degraded)
    );
}

#[test]
fn provider_failure_and_sni_failure_stay_scoped() {
    let mut providers = vec![provider("codex"), provider("claude")];
    apply_provider_failure(&mut providers, "codex");
    assert!(providers[0].degraded);
    assert!(!providers[1].degraded);
    let mut tray = TrayHealth::Ready;
    apply_sni_failure(&mut tray);
    assert_eq!(tray, TrayHealth::Degraded);
    assert_eq!(providers[1].last_good, Some(SystemTime::UNIX_EPOCH));
}

#[test]
fn invalid_edit_does_not_persist_provider_mutation() {
    let mut providers = vec![provider("codex")];
    assert!(!apply_edit(&mut providers, "codex", ""));
    assert_eq!(providers[0].id, "codex");
    assert!(apply_edit(&mut providers, "codex", "codex-next"));
    assert_eq!(providers[0].id, "codex-next");
}
