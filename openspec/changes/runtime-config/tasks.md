# Tasks: runtime-config

## 1. Config module

- [x] 1.1 Create `src/config.rs` with the `Config` struct
  (`bind_addr`, `max_source_bytes`, `fetch_timeout`, `watermark_font`,
  `allow_remote_sources`, `local_source_root`).
- [x] 1.2 `Config::default()` returns the documented defaults
  (`0.0.0.0:8080`, 25 MiB, 10s, `None`, `true`, `None`).
- [x] 1.3 `Config::from_env()` reads each `THUMBOR_*` variable,
  parses it, and replaces the corresponding field. On parse
  failure, log `tracing::warn!` and leave the default.

## 2. Field-by-field parsing

- [x] 2.1 `THUMBOR_BIND` — `v.parse::<SocketAddr>()`. On failure,
  warn.
- [x] 2.2 `THUMBOR_MAX_SOURCE_BYTES` — `v.parse::<usize>()`. On
  failure, warn.
- [x] 2.3 `THUMBOR_FETCH_TIMEOUT_MS` — `v.parse::<u64>()`,
  `Duration::from_millis`. On failure, warn.
- [x] 2.4 `THUMBOR_WATERMARK_FONT` — set `watermark_font =
  Some(PathBuf::from(v))`. No parsing required.
- [x] 2.5 `THUMBOR_ALLOW_REMOTE` — match `1` / `true` / `yes`
  (case-insensitive) as `true`, anything else as `false`. No warn
  for unexpected values (it is a toggle, not a parsed value).
- [x] 2.6 `THUMBOR_LOCAL_SOURCE_ROOT` — set `local_source_root =
  Some(PathBuf::from(v))`. No parsing required.

## 3. Wire it into the binary

- [x] 3.1 `src/main.rs` calls `Config::from_env()` and uses
  `config.bind_addr` for the axum listener and `config.fetch_timeout`
  for the `reqwest::Client`.

## 4. Documentation

- [x] 4.1 The full table of `THUMBOR_*` variables with their
  defaults and meaning is in `AGENTS.md §2` (cross-referenced from
  the spec).

## 5. Verification

- [x] 5.1 `cargo check --all-targets` exits 0.
- [x] 5.2 The unit tests in `src/source.rs` exercise the
  `THUMBOR_ALLOW_REMOTE=false` path (rejects remote sources).
