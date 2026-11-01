use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSnapshotDto {
    pub schema_version: u32,
    pub generated_at: String,
    pub stale_after_seconds: u64,
    pub providers: Vec<ProviderDto>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderDto {
    pub id: String,
    pub name: String,
    pub actions: Option<ProviderActionsDto>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderActionsDto {
    #[serde(rename = "dashboardURL")]
    pub dashboard_url: Option<String>,
}
