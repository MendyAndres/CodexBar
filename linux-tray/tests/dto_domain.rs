use codexbar_linux_tray::{
    DashboardSnapshotDto, DomainSnapshot, LinkOrigin, TrayEvent, ValidatedUrl,
};
use std::time::Duration;

#[test]
fn converts_wire_staleness_and_accepts_dashboard_https_action() {
    let dto: DashboardSnapshotDto = serde_json::from_str(
        r#"{"schemaVersion":1,"generatedAt":"2026-08-02T00:00:00Z","staleAfterSeconds":30,"providers":[{"id":"codex","name":"Codex","actions":{"dashboardURL":"https://chatgpt.com/codex"}}]}"#,
    )
    .unwrap();
    let snapshot = DomainSnapshot::try_from(dto).unwrap();
    assert_eq!(snapshot.freshness_threshold, Duration::from_secs(30));
    assert_eq!(
        snapshot.providers[0]
            .dashboard_url
            .as_ref()
            .unwrap()
            .as_str(),
        "https://chatgpt.com/codex"
    );
}

#[test]
fn tolerates_unknown_fields_but_rejects_untrusted_or_non_https_links() {
    let dto: DashboardSnapshotDto = serde_json::from_str(
        r#"{"schemaVersion":1,"generatedAt":"2026-08-02T00:00:00Z","staleAfterSeconds":30,"future":true,"providers":[{"id":"codex","name":"Codex","actions":{"dashboardURL":"http://example.test","future":true}}]}"#,
    )
    .unwrap();
    assert!(
        DomainSnapshot::try_from(dto).unwrap().providers[0]
            .dashboard_url
            .is_none()
    );
    for origin in [LinkOrigin::Error, LinkOrigin::Config, LinkOrigin::Identity] {
        assert!(ValidatedUrl::parse(origin, "https://example.test").is_err());
    }
}

#[test]
fn open_link_requires_a_validated_url() {
    let url = ValidatedUrl::parse(LinkOrigin::DashboardAction, "https://example.test").unwrap();
    let event = TrayEvent::OpenLink(url.clone());
    assert!(matches!(event, TrayEvent::OpenLink(value) if value == url));
}
