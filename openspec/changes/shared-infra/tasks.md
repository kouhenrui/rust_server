# Tasks: shared-infra

## 1. Util module

- [x] 1.1 Create `src/util/mod.rs` with `parse_or_warn`. (`src/util/mod.rs`)
- [x] 1.2 Add `redact_url`; migrate `db/sql.rs`, `db/mongo.rs`, `cache/config.rs`.
- [x] 1.3 Unit tests for parse and redact.

## 2. HTTP client

- [x] 2.1 Create `src/http_client.rs` with `HttpClient::build` / `fetch`.
- [x] 2.2 Migrate `src/source.rs` remote loading to `state.http.fetch`.
- [x] 2.3 Wire in `AppState::connect` (replaces raw `reqwest::Client`).

## 3. Macros and ergonomics

- [x] 3.1 Add `span!`, `ok!`, `err!` to `src/logger/macros.rs`.
- [x] 3.2 Add `AppResultExt` / `AppResultMapExt` in `src/error.rs`.
- [x] 3.3 Implement `TraceId` `FromRequestParts` in `src/middleware/middleware.rs`.
- [x] 3.4 Use `crate::span!` in logging middleware.

## 4. Health dependencies

- [x] 4.1 Add `ComponentHealth` and extend `HealthData` in `src/response.rs`.
- [x] 4.2 Implement `AppState::check_health` with cache/db `ping`.
- [x] 4.3 Update `controller/health.rs` to use `ok!` and `State`.
- [x] 4.4 Update integration test for `data.cache` / `data.database`.

## 5. AppState wiring

- [x] 5.1 `AppState::connect` wires http, fonts, cache, db, jwt. (`src/state.rs`)
- [x] 5.2 `FontCache` lazy font loading on `AppState.fonts`.

## 6. Config migration

- [x] 6.1 Use `parse_or_warn` in `config.rs`, `cache/config.rs`, `db/config.rs`.

## 7. Verification

- [x] 7.1 `cargo test` passes (util + health integration).
