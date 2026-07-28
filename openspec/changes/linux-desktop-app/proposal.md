# Proposal: Linux Desktop App

## Intent

Provide Ubuntu 24.04 GNOME users a native CodexBar tray while retaining the Swift CLI/backend and provider isolation. The first implementation is a Rust desktop client in this monorepo, with versioned HTTP JSON as the language boundary.

## Scope

### In Scope
- Add `linux-tray/` using `gtk4-rs`, `libadwaita-rs`, and GIO/zbus for SNI/D-Bus.
- Extend schema v1 additively with provider filtering, scoped refresh, and optional backend-derived dashboard actions.
- Supervise an owned loopback Swift CLI child without exposing its token.
- Present independently scoped backend, provider, configuration, and tray health with retained provider data.

### Out of Scope
- Reimplementing provider probes in Rust or porting the macOS UI.
- Browser cookies, libsecret, remote serving, silent CLI installation/download, or foreign-process adoption.
- `.deb` packaging, auto-update, and community publication before local validation.

## Capabilities

### New Capabilities
- `linux-desktop-app`: Rust GNOME tray, managed Swift backend, provider state/actions, diagnostics, and safe configuration.

### Modified Capabilities
- None; no existing OpenSpec capability is defined.

## Approach

Decode exact v1 wire DTOs, then derive domain freshness and UI state through a tray adapter. Keep wire `staleAfterSeconds` distinct from domain `freshnessThreshold` and backend stale-fallback retention. Extend `GET /dashboard/v1/snapshot` with `provider`, `refresh=1`, and optional provider-scoped `actions.dashboardURL` sourced from `ProviderDescriptor.dashboardURL`. Rust validates HTTPS links but never constructs them. Backend, provider, invalid-config, and SNI failures remain independently scoped.

## Affected Areas

| Area | Impact | Description |
|---|---|---|
| `linux-tray/` | New | Rust host, state, UI, SNI, config, supervision, logs |
| `Sources/CodexBarCLI/` | Modified | Filtered snapshot/refresh, descriptor-derived actions, secure readiness |
| Linux tests/docs/CI | Modified | Contract, lifecycle, D-Bus, config, and local QA |

## Risks

| Risk | Mitigation |
|---|---|
| GNOME lacks watcher/extension | Report unsupported/degraded and open a normal window |
| Secret or process-ownership leak | Pipe/fd token, redaction, instance matching, bounded termination |
| Failure scope erases valid data | Explicit backend/provider/config/tray transitions preserve `lastGood` |
| Untrusted or drifting links | Optional additive fields, HTTPS validation, exact DTO fixtures |

## Rollback Plan

Remove `linux-tray/` and Linux CI/docs; independently revert additive snapshot query and readiness changes. Existing Swift CLI and macOS products remain usable.

## Dependencies

- Ubuntu 24.04 GTK4/libadwaita, session D-Bus, and GNOME AppIndicator extension.
- A discoverable compatible `CodexBarCLI`; no automatic installation.

## Success Criteria

- [ ] Supported sessions expose accurate scoped states and trusted dashboard actions; absent actions remain compatible.
- [ ] Filtered refresh, distinct freshness policies, four failure scopes, and configuration rollback are verifiable.
- [ ] Child lifecycle and diagnostics expose no token and never control a foreign process.
