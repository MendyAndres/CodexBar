# Tasks: Linux Desktop App

## Review Workload Forecast

| Field | Value |
|---|---|
| Estimated changed lines | 1,200–1,800 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | Contract → Rust core/security → tray/UI/QA |
| Delivery strategy | ask-on-risk |
| Chain strategy | stacked-to-main |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|---|---|---|---|---|---|
| 1 | Swift snapshot contract and readiness | PR 1 → `main` | `swift test --filter DashboardSnapshot` | N/A: HTTP fixtures only | Swift CLI files and additive routes |
| 2 | Rust workspace, DTO/state/backend security | PR 2 → `main` after PR 1 | `cargo test --manifest-path linux-tray/Cargo.toml` | N/A: fake loopback child/fixtures | `linux-tray/src/{dto,domain,state,backend,discovery}.rs` |
| 3 | Tray/UI/config/diagnostics | PR 3 → `main` after PR 2 | `cargo test --manifest-path linux-tray/Cargo.toml` | N/A: GNOME session required; use fake D-Bus tests | remaining `linux-tray/src` and docs/CI |

## Phase 1: Swift Contract and RED Tests

- [x] 1.1 RED: add XCTest fixtures for provider filtering, `refresh=1`, invalid-provider 400, additive/absent fields, malformed required schema, and token-free queries in `Sources/CodexBarCLI/{CLIServeCommand,DashboardSnapshotBuilder}.swift` tests.
- [x] 1.2 GREEN: implement scoped snapshot/cache refresh and `ProviderDescriptor.dashboardURL`-derived optional actions; preserve schema-v1 compatibility. Run `swift test --filter DashboardSnapshot`.
- [x] 1.3 RED then GREEN: add Swift server tests for port `0`, bounded fd-token input, token-free readiness with actual port/instance ID, and rejection of argv/environment tokens; leave child ownership, output capture, termination, and reaping to Rust task 2.3.

## Phase 2: Rust Workspace, DTO, State, Supervisor

- [ ] 2.1 Create `linux-tray/Cargo.toml`, workspace modules, and test harness; add RED tests for freshness conversion, HTTPS-only `ValidatedUrl`, rejected error/config/identity links, and `OpenLink` type safety.
- [ ] 2.2 Implement `src/dto.rs`, `domain.rs`, and `state.rs`; add RED reducer tests for backend-wide aging, provider-only degradation, invalid-edit non-persistence, and SNI-only health mutation, then make them pass.
- [ ] 2.3 Add RED security tests for metacharacter executable paths, foreign listener/instance mismatch, crash loops, output floods, ignored TERM, and orphan prevention; implement `src/discovery.rs` and `src/backend.rs` with bounded ownership/lifecycle.
- [ ] 2.4 Run `cargo test --manifest-path linux-tray/Cargo.toml` and `cargo clippy --manifest-path linux-tray/Cargo.toml --all-targets -- -D warnings`; refactor without changing contracts.

## Phase 3: Config, Polling, SNI/DBusMenu, UI

- [ ] 3.1 RED then GREEN: test typed allowlist, masked secrets, atomic 0600 write, rollback, independent polling/coalescing, jittered backoff, and retained `lastGood`; implement `src/config.rs` and polling integration.
- [ ] 3.2 RED then GREEN: fake-D-Bus tests for SNI/watcher absence and DBusMenu events; implement `src/tray.rs` with `TrayAdapter` boundaries and `src/ui.rs` with normal-window fallback.
- [ ] 3.3 Implement `src/logging.rs` with tracing redaction, bounded rotation, and diagnostics; test token/secret redaction and provider isolation.

## Phase 4: CI, Documentation, QA

- [ ] 4.1 Add Linux Cargo jobs and fixture-only Swift checks to CI; document local Ubuntu 24.04 GNOME QA in `docs/` without live providers, cookies, Keychain, downloads, or GUI automation claims.
- [ ] 4.2 Run `make test`, `make check`, focused Swift/Cargo suites, and manual local GNOME/SNI fallback QA; record results and rollback boundaries.
