# Design: Linux Desktop App

## Technical Approach

Create a Rust workspace at `linux-tray/` using `gtk4-rs`, `libadwaita-rs`, and GIO/zbus. Keep provider execution and dashboard generation in the Swift CLI; cross the language boundary only through authenticated loopback HTTP JSON. Rust owns desktop state and its explicitly spawned backend process, not provider logic.

## Architecture Decisions

| Topic | Decision and rationale |
|---|---|
| Host | Rust from inception; its maintained GNOME bindings and D-Bus ecosystem avoid a Swift GTK/shim stack. |
| Contract | Extend v1 additively with optional `provider`, `refresh=1`, and provider `actions.dashboardURL`; absent/unknown additive fields remain compatible. Swift derives actions from `ProviderDescriptor.dashboardURL`; Rust never constructs URLs. |
| State | Exact DTOs retain `stale_after_seconds`; conversion derives domain `freshness_threshold: Duration`; UI consumes domain state. Backend stale-fallback retention remains a separate internal policy. |
| Links | Parse backend actions into HTTPS-only `ValidatedUrl`. Only this type reaches `TrayEvent::OpenLink`; error, config, and identity strings cannot become links. |
| Tray | Implement SNI `org.kde.StatusNotifierItem`, `com.canonical.dbusmenu`, watcher registration, and session bus. GNOME AppIndicator is an environment requirement; no watcher yields unsupported/degraded plus normal-window fallback. |
| Process auth | Direct argv, loopback port 0, random instance ID, readiness match, and random token through inherited pipe/fd only. This prevents argv/environment/persistence disclosure and foreign-process ownership. |

## Data Flow and Concurrency

```text
GTK/SNI event -> AppState reducer -> Dashboard client -> owned Swift CLI
                       ^                 |
                       +-- domain state <- DashboardSnapshotDto
```

Polling is independent of backend TTL. Provider refreshes are coalesced; retries use bounded exponential backoff plus jitter. Failure reducers are explicit: backend disconnect marks every provider disconnected while each `lastGood` ages stale/unavailable; one provider failure degrades only it; invalid config affects only the edited operation/provider and persists nothing; SNI failure changes only `TrayHealth`, preserving provider data and fallback retention.

## Components and Files

| Path | Responsibility |
|---|---|
| `linux-tray/src/{dto,domain,state}.rs` | Exact JSON/actions DTOs, freshness conversion, scoped reducers |
| `linux-tray/src/{backend,discovery}.rs` | HTTP client, CLI search, child ownership/readiness |
| `linux-tray/src/{tray,ui}.rs` | SNI/DBusMenu adapter and libadwaita window |
| `linux-tray/src/{config,logging}.rs` | Typed atomic config and redacted `tracing` |
| `Sources/CodexBarCLI/{CLIServeCommand,DashboardSnapshotBuilder}.swift` | Scoped snapshot/cache/actions and fd-token readiness |

CLI discovery order is explicit override, sibling executable, known XDG paths, then `PATH`; failure is actionable and never downloads. Shutdown is TERM -> timeout -> KILL -> wait/reap. Crash restarts and captured output are bounded.

## Interfaces / Contracts

`TrayAdapter` provides `start`, `apply`, event stream, and `stop`; events are `Refresh(provider)`, `OpenLink(ValidatedUrl)`, and `Quit`. The adapter never performs provider work, URL construction, config writes, retry policy, or process ownership.

`DashboardSnapshotDto` covers current fields plus optional `actions.dashboardURL`; serde ignores unknown additive fields and defaults absent actions. Its `stale_after_seconds` wire value converts once to domain `freshness_threshold`. Domain types do not expose backend fallback retention.

Config uses a typed allowlist over `~/.codexbar/config.toml`. Save validates, writes a same-directory temp file at 0600, atomically renames, and rolls back on failure. Secrets are masked; cookies and libsecret are absent.

Logging uses `tracing` to stderr/journald by default. File logging is opt-in at `$XDG_STATE_HOME/codexbar/linux-tray.log` or `~/.local/state/codexbar/linux-tray.log`, rotating 3x5 MiB. Diagnostics uses the same redaction.

## Threat Matrix

| Boundary | Applicability | Safe/failure behavior | Planned RED test |
|---|---|---|---|
| Documentation-like paths | N/A: no executable classification | None | None |
| Git repository selection | N/A: no VCS | None | None |
| Commit state | N/A: no VCS | None | None |
| Push state | N/A: no VCS | None | None |
| PR commands | N/A: no PR automation | None | None |
| Executable/argv | Applicable | Resolved executable is one argv value; missing/non-executable fails actionably | Metacharacter path is never shell-interpreted |
| Endpoint ownership | Applicable | Loopback/port 0 and instance match; mismatch never adopts/signals PID | Foreign listener/wrong instance is untouched |
| Token transport | Applicable | Pipe/fd and memory only; redact all output | Token absent from argv, env, files, logs, diagnostics |
| Child lifecycle | Applicable | Bounded restart/output; TERM/KILL/wait/reap | Crash loop, output flood, ignored TERM leave no orphan |

## Testing Strategy

RED unit tests cover DTO-to-domain freshness naming/conversion; absent and unknown additive fields; valid HTTPS action acceptance; non-HTTPS, malformed, and error/config/identity URL rejection; and `OpenLink` type safety. Reducer tests independently prove backend-wide disconnect with timed stale/unavailable evolution, one-provider degradation, invalid-edit non-persistence, and SNI-only TrayHealth mutation with fallback retention preserved. Integration fixtures cover descriptor-derived actions, schema rejection for incompatible/malformed required fields, scoped refresh/400, fd token/readiness, process ownership, and fake D-Bus fallback. No live providers, cookies, secrets, or downloads.

## Migration / Rollout

Local source build and Ubuntu 24.04 GNOME QA first. `.deb` and community publication remain deferred. Rollback removes `linux-tray/` and independently reverts additive Swift endpoints.

## Open Questions

None; implementation details must preserve these contracts.
