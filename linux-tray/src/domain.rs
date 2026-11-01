use crate::dto::DashboardSnapshotDto;
use std::time::Duration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedUrl(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkOrigin {
    DashboardAction,
    Error,
    Config,
    Identity,
}

#[derive(Debug)]
pub enum UrlValidationError {
    UntrustedOrigin,
}

impl ValidatedUrl {
    pub fn parse(origin: LinkOrigin, value: &str) -> Result<Self, UrlValidationError> {
        let host = value
            .strip_prefix("https://")
            .filter(|host| !host.is_empty());
        if origin == LinkOrigin::DashboardAction && host.is_some() {
            Ok(Self(value.into()))
        } else {
            Err(UrlValidationError::UntrustedOrigin)
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub enum TrayEvent {
    OpenLink(ValidatedUrl),
    Refresh(String),
    Quit,
}

pub struct DomainSnapshot {
    pub freshness_threshold: Duration,
    pub providers: Vec<DomainProvider>,
}

pub struct DomainProvider {
    pub id: String,
    pub name: String,
    pub dashboard_url: Option<ValidatedUrl>,
}

impl TryFrom<DashboardSnapshotDto> for DomainSnapshot {
    type Error = ();
    fn try_from(dto: DashboardSnapshotDto) -> Result<Self, Self::Error> {
        if dto.schema_version != 1 {
            return Err(());
        }
        Ok(Self {
            freshness_threshold: Duration::from_secs(dto.stale_after_seconds),
            providers: dto
                .providers
                .into_iter()
                .map(|provider| DomainProvider {
                    dashboard_url: provider
                        .actions
                        .and_then(|actions| actions.dashboard_url)
                        .and_then(|url| {
                            ValidatedUrl::parse(LinkOrigin::DashboardAction, &url).ok()
                        }),
                    id: provider.id,
                    name: provider.name,
                })
                .collect(),
        })
    }
}
