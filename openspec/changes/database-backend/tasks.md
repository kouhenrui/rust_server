# Tasks: database-backend

## 1. Database module

- [x] 1.1 Create `src/db/` with `config.rs`, `client.rs`, `sql.rs`, `mongo.rs`.
- [x] 1.2 Define `DbProvider` trait with `ping` and `backend_name`.
- [x] 1.3 `DbClient::connect` dispatches on `DbBackendConfig`.

## 2. Configuration

- [x] 2.1 `DbBackendConfig::from_env()` reads `THUMBOR_DB_BACKEND` (default sqlite).
- [x] 2.2 Support `THUMBOR_DB_URL` or discrete host/port/name/auth/path fields.
- [x] 2.3 Invalid `THUMBOR_DB_PORT` warns and keeps default.

## 3. Startup wiring

- [x] 3.1 `AppState::connect` requires successful `Db::connect`.
- [x] 3.2 Log `database ready` with backend name.
- [x] 3.3 Call `entity::migrate` for SQL backends.

## 4. Entity integration

- [x] 4.1 `src/entity/` module: models, repositories, schema, sql_backend.
- [x] 4.2 Tables: `accounts`, `casbin_rule`.
- [x] 4.3 `entity/test_util.rs` for unit test pools.

## 5. Tests

- [x] 5.1 Unit test: disabled backend rejected, sqlite pool exposes pool.
  (`src/db/client.rs`)
- [x] 5.2 Integration tests use `tests/common/mod.rs` with unique memory DB names.
- [x] 5.3 `AppState::test` helper for lib tests (`#[cfg(test)]` in `state.rs`).

## 6. Verification

- [x] 6.1 `cargo test` passes.
