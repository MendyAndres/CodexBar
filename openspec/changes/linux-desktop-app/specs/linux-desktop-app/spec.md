# Linux Desktop App Specification

## Requirements

### Requirement: Versioned provider snapshot contract

The backend MUST support `GET /dashboard/v1/snapshot`, provider filtering, and scoped refresh. Invalid providers MUST return 400. Schema v1 MUST tolerate unknown additive and absent optional fields. Tokens MUST NOT appear in queries.

#### Scenario: Scoped forced refresh
- GIVEN two providers have cached v1 snapshots
- WHEN one valid provider is requested with `refresh=1`
- THEN only its cache is recomputed

#### Scenario: Invalid provider
- GIVEN an unknown provider ID
- WHEN its snapshot is requested
- THEN the backend returns 400 without refreshing any provider

### Requirement: Trusted provider actions

Schema v1 MUST support optional provider `actions.dashboardURL`, derived from `ProviderDescriptor.dashboardURL`. Rust MUST NOT construct URLs. Open dashboard MUST require an `https` `ValidatedUrl`; `OpenLink(ValidatedUrl)` MUST NOT originate from error, config, or identity.

#### Scenario: Trusted dashboard link
- GIVEN a provider payload contains a valid HTTPS `actions.dashboardURL`
- WHEN Rust projects DTO state into UI state
- THEN Open dashboard emits `OpenLink` with that validated URL

#### Scenario: Missing, unknown, or rejected action
- GIVEN actions are absent, fields are unknown, or the URL is invalid/non-HTTPS
- WHEN the v1 payload is decoded
- THEN decoding succeeds and Open dashboard is unavailable

### Requirement: Layered provider state

Wire `staleAfterSeconds` MUST remain Rust `stale_after_seconds`; domain/UI MUST derive `freshnessThreshold: Duration` as `freshness_threshold`. Backend fallback retention MUST remain separate. DTO, domain, and UI MUST remain distinct; UTC timestamps cross the wire, monotonic `Instant` does not.

#### Scenario: Backend failure
- GIVEN every provider has `lastGood` data
- WHEN backend connection fails
- THEN all lose connection while each `lastGood` ages stale then unavailable

#### Scenario: Provider failure
- GIVEN multiple providers have valid state
- WHEN one provider fails
- THEN only that provider becomes degraded

#### Scenario: SNI failure
- GIVEN provider data and fallback retention are active
- WHEN SNI publication fails
- THEN only TrayHealth changes; provider data and fallback windows remain

### Requirement: Tray and user actions

The client MUST implement SNI, DBusMenu, watcher registration, and window fallback. UI MUST expose health, refresh, trusted actions, diagnostics, retry, and quit.

#### Scenario: Watcher unavailable
- GIVEN no usable StatusNotifier watcher exists
- WHEN the client starts
- THEN tray state is unsupported/degraded and the normal window remains usable

### Requirement: Safe configuration

Configuration MUST use a typed allowlist, masked secrets, validation, atomic mode-0600 writes, and rollback. It MUST NOT expose cookies or libsecret.

#### Scenario: Invalid save rolls back
- GIVEN existing valid configuration
- WHEN an edited operation/provider fails validation
- THEN only that edit reports failure and no configuration mutation is persisted

### Requirement: Polling and refresh resilience

Polling MUST be independent of backend TTL, retain provider `lastGood`, coalesce refreshes, and use bounded jittered backoff.

#### Scenario: Concurrent refresh
- GIVEN a provider refresh is active
- WHEN another refresh for it arrives
- THEN work is coalesced and failure preserves `lastGood`

### Requirement: Owned backend and privacy

CLI discovery MUST use override, sibling, XDG, then `PATH`, without installation. Spawn MUST use direct argv, port 0, matched readiness, and an in-memory fd token. Supervision MUST NOT control foreign processes and MUST bound crashes, output, termination, and reaping. Logs MUST be redacted.

#### Scenario: Foreign endpoint or secret boundary
- GIVEN readiness has the wrong instance ID or output contains a token
- WHEN supervision and diagnostics process it
- THEN no foreign PID is controlled and no token is persisted, logged, or exposed via argv/environment

### Requirement: Tray adapter boundary and delivery scope

The adapter MUST expose start/apply/events/stop and emit only `Refresh(provider)`, `OpenLink(ValidatedUrl)`, or `Quit`. It MUST NOT own providers, config, retries, validation, or processes. Delivery MUST remain local-first.

#### Scenario: Adapter event
- GIVEN a provider refresh menu action
- WHEN the adapter emits the event
- THEN application state handles refresh outside the adapter
