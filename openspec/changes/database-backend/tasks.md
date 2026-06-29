# Tasks: database-backend

## 1. Database module

- [x] 1.1 Create `src/db/` with `config.rs`, `db.rs`, `sql.rs`, `mongo.rs`.
- [x] 1.2 Define `DbProvider` trait with `ping` and `backend_name`.
- [x] 1.3 `DbClient::connect` dispatches on `DbBackendConfig`.

## 2. Configuration

- [x] 2.1 `DbBackendConfig::from_env()` reads `THUMBOR_DB_BACKEND` (default sqlite).
- [x] 2.2 Support `THUMBOR_DB_URL` or discrete host/port/name/auth/path fields.
- [x] 2.3 Invalid `THUMBOR_DB_PORT` warns and keeps default.

## 3. Startup wiring

- [x] 3.1 `AppState::connect` requires successful `Db::connect`.
- [x] 3.2 Log `database ready` with backend name.

## 4. Tests

- [x] 4.1 Unit test: disabled backend rejected, sqlite pool exposes pool.
  (`src/db/db.rs`)
- [x] 4.2 Integration tests use `sqlite::memory:`. (`tests/integration.rs`)

## 5. Verification

- [x] 5.1 `cargo test` passes.
