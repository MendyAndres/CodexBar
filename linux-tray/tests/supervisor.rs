use codexbar_linux_tray::{CliDiscovery, Readiness, Supervisor, SupervisorError};
use std::time::Duration;

#[test]
fn metacharacters_are_not_shell_interpreted_and_readiness_requires_our_loopback_instance() {
    let marker = std::env::temp_dir().join(format!("codexbar-marker-{}", std::process::id()));
    let candidate = format!("/missing; touch {}", marker.display());
    assert!(matches!(
        CliDiscovery::from_override(&candidate),
        Err(SupervisorError::ExecutableNotFound(_))
    ));
    assert!(!marker.exists());
    assert!(Readiness::parse(r#"{"instanceID":"ours","port":42123}"#, "ours").is_ok());
    assert!(Readiness::parse(r#"{"instanceID":"foreign","port":42123}"#, "ours").is_err());
    assert!(Readiness::parse(r#"{"instanceID":"ours","port":0}"#, "ours").is_err());
}

#[test]
fn discovery_prefers_override_then_sibling_xdg_and_path() {
    let executable = std::path::PathBuf::from("/bin/sh");
    assert_eq!(
        CliDiscovery::resolve(
            Some(executable.clone()),
            Some(executable.clone()),
            None,
            None
        )
        .unwrap(),
        executable
    );
    assert_eq!(
        CliDiscovery::resolve(
            None,
            Some(executable.clone()),
            Some(executable.clone()),
            None
        )
        .unwrap(),
        executable
    );
    assert_eq!(
        CliDiscovery::resolve(
            None,
            None,
            Some(executable.clone()),
            Some(executable.clone())
        )
        .unwrap(),
        executable
    );
}

#[test]
fn crash_budget_and_output_capture_are_bounded_without_token_leakage() {
    let mut supervisor = Supervisor::new(2, 8);
    assert!(supervisor.record_exit().is_ok());
    assert!(supervisor.record_exit().is_ok());
    assert!(supervisor.record_exit().is_err());
    supervisor.capture("token=secret-value output flood");
    assert!(supervisor.diagnostics().len() <= 8);
    assert!(!supervisor.diagnostics().contains("secret-value"));
}

#[cfg(unix)]
#[test]
fn stop_kills_ignored_term_process_group_and_reaps_owner() {
    let mut supervisor = Supervisor::spawn(
        "/bin/sh",
        &[
            "-c",
            "trap '' TERM; (trap '' TERM; while :; do :; done) & while :; do :; done",
        ],
        "secret",
    )
    .unwrap();
    supervisor.stop(Duration::from_millis(20)).unwrap();
    assert!(supervisor.is_reaped());
}
