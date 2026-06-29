# Change: runtime-config

## Why

The service runs in a variety of environments (local dev, CI, staging,
production behind a reverse proxy). Each environment has different
network addresses, source-image size limits, and remote-source
policies. The runtime-config step is what lets one binary serve all
of them without rebuilds.

## What Changes

- Read all configuration from environment variables prefixed with
  `THUMBOR_`. No other prefix is read.
- Allow the bind address to be overridden by `THUMBOR_BIND`
  (default `0.0.0.0:8080`).
- Allow the source-image size cap to be overridden by
  `THUMBOR_MAX_SOURCE_BYTES` (default 26214400 / 25 MiB).
- Allow the remote-fetch timeout to be overridden by
  `THUMBOR_FETCH_TIMEOUT_MS` (default 10000).
- Allow the text-watermark font path to be set by
  `THUMBOR_WATERMARK_FONT` (default unset).
- Allow remote sources to be disabled by `THUMBOR_ALLOW_REMOTE`
  (default `true`).
- Allow a local-source root to be set by
  `THUMBOR_LOCAL_SOURCE_ROOT` (default unset).
- When a `THUMBOR_*` value is unparseable, log a `tracing::warn!`
  and fall back to the documented default. The service MUST NOT
  refuse to start because of a malformed configuration value.

## Capabilities

- `runtime-config`: Single-prefix, configurable bind address, invalid
  values fall back to defaults, documented variables.

## Impact

- **Code:** new `src/config.rs` with the `Config` struct,
  `Config::default`, and `Config::from_env`. Called by `main.rs`
  at startup.
- **Dependencies:** none — only the standard library's `std::env::var`.
- **Operational surface:** the operator's contract is the set of
  `THUMBOR_*` variables, all listed in `AGENTS.md §2`. Adding a
  new variable means a new field on `Config` and a new branch in
  `from_env`, plus a row in `AGENTS.md` and a row in the
  `runtime-config` spec.

## Non-goals

- File-based configuration (e.g. YAML, TOML, JSON). Env vars only
  for now.
- Live reload of configuration without a process restart.
- Per-request overrides (e.g. an admin endpoint that sets
  `THUMBOR_ALLOW_REMOTE` for the next request).
- Secret values (tokens, API keys). The service has no auth at this
  layer, and putting secrets in env vars is *not* a secret manager.

## Affected domains

- **runtime-config** (created) — the only spec modified or created
  by this change.
