use std::time::{Duration, SystemTime};

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderState {
    pub id: String,
    pub last_good: Option<SystemTime>,
    pub disconnected: bool,
    pub stale: bool,
    pub degraded: bool,
}

impl ProviderState {
    pub fn connected(id: &str, last_good: SystemTime) -> Self {
        Self {
            id: id.into(),
            last_good: Some(last_good),
            disconnected: false,
            stale: false,
            degraded: false,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TrayHealth {
    Ready,
    Degraded,
}

pub fn apply_backend_failure(
    providers: &mut [ProviderState],
    now: SystemTime,
    freshness: Duration,
) {
    for provider in providers {
        provider.disconnected = true;
        provider.stale = provider
            .last_good
            .is_none_or(|last_good| now.duration_since(last_good).unwrap_or_default() >= freshness);
    }
}

pub fn apply_provider_failure(providers: &mut [ProviderState], id: &str) {
    if let Some(provider) = providers.iter_mut().find(|provider| provider.id == id) {
        provider.degraded = true;
    }
}

pub fn apply_sni_failure(tray: &mut TrayHealth) {
    *tray = TrayHealth::Degraded;
}

pub fn apply_edit(providers: &mut [ProviderState], id: &str, replacement: &str) -> bool {
    if replacement.is_empty() {
        return false;
    }
    if let Some(provider) = providers.iter_mut().find(|provider| provider.id == id) {
        provider.id = replacement.into();
        true
    } else {
        false
    }
}
