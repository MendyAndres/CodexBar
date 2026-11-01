mod backend;
mod discovery;
mod domain;
mod dto;
mod state;

pub use backend::{Readiness, Supervisor};
pub use discovery::{CliDiscovery, SupervisorError};
pub use domain::{DomainSnapshot, LinkOrigin, TrayEvent, ValidatedUrl};
pub use dto::DashboardSnapshotDto;
pub use state::{
    ProviderState, TrayHealth, apply_backend_failure, apply_edit, apply_provider_failure,
    apply_sni_failure,
};
