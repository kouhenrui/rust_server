# Tasks: runtime-config

## 1. Config module

- [x] 1.1 Create `src/config.rs` with the `Config` struct
  (`bind_addr`, `max_source_bytes`, `fetch_timeout`, `watermark_font`,
  `allow_remote_sources`, `local_source_root`, `log_level`, `jwt_secret`,
  `jwt_expire_secs`).
- [x] 1.2 `Config::default()` returns the documented defaults
  (`0.0.0.0:8080`, 25 MiB, 10s, `None`, `true`, `None`).
- [x] 1.3 `Config::from_env()` reads each `THUMBOR_*` variable,
  parses it, and replaces the corresponding field. On parse
  failure, log `crate::warn!` and leave the default.
- [x] 1.4 `Config::load_dotenv()` loads `.env` via `dotenvy` before logger init.

## 2. Field-by-field parsing

- [x] 2.1 `THUMBOR_BIND` — `v.parse::<SocketAddr>()`. On failure, warn.
- [x] 2.2 `THUMBOR_MAX_SOURCE_BYTES` — `v.parse::<usize>()`. On failure, warn.
- [x] 2.3 `THUMBOR_FETCH_TIMEOUT_MS` — `v.parse::<u64>()`,
  `Duration::from_millis`. On failure, warn.
- [x] 2.4 `THUMBOR_WATERMARK_FONT` — set `watermark_font =
  Some(PathBuf::from(v))`. No parsing required.
- [x] 2.5 `THUMBOR_ALLOW_REMOTE` — match `1` / `true` / `yes`
  (case-insensitive) as `true`, anything else as `false`.
- [x] 2.6 `THUMBOR_LOCAL_SOURCE_ROOT` — set `local_source_root =
  Some(PathBuf::from(v))`. No parsing required.
- [x] 2.7 `THUMBOR_LOG_LEVEL` — set `log_level` string (used by `LoggerConfig`
  when `RUST_LOG` is unset).
- [x] 2.8 `THUMBOR_DOTENV_PATH` — custom dotenv file path in `load_dotenv`.
- [x] 2.9 `THUMBOR_JWT_SECRET` — set `jwt_secret` when non-empty.
- [x] 2.10 `THUMBOR_JWT_EXPIRE_SECS` — `parse_or_warn` into `jwt_expire_secs`.

## 3. Wire it into the binary

- [x] 3.1 `src/main.rs` calls `Config::load_dotenv()`, `logger::init()`,
  `Config::from_env()`, `AppState::connect`, and uses `config.bind_addr` for
  the axum listener with `with_graceful_shutdown` (SIGINT + SIGTERM).
- [x] 3.2 `AppState::connect` uses `HttpClient::build(config.fetch_timeout)`.
- [x] 3.3 `CorsLayer::permissive()` applied in `main.rs` (production config TBD).

## 4. Documentation

- [x] 4.1 Variable tables in `AGENTS.md`, `.env.example`, and
  `openspec/specs/runtime-config/spec.md`.

## 5. Verification

- [x] 5.1 `cargo check --all-targets` exits 0.
- [x] 5.2 `cargo test --lib` passes (includes `THUMBOR_ALLOW_REMOTE=false`).
