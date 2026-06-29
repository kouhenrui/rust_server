# Tasks: observability

## 1. Logger module

- [x] 1.1 Create `src/logger/` with `config.rs`, `formatter.rs`, `init.rs`,
  `layer.rs`, `macros.rs`. (`src/logger/`)
- [x] 1.2 `logger::init()` registers global subscriber from `LoggerConfig`.
- [x] 1.3 Export `trace!`, `debug!`, `info!`, `warn!`, `error!` at crate root.

## 2. HTTP middleware

- [x] 2.1 Implement `logging_middleware` in `src/middleware/middleware.rs`.
- [x] 2.2 Summarize GET query strings and POST protobuf `ImageRequest` fields.
- [x] 2.3 Log `http request received` and `http request completed` via `crate::info!`.
- [x] 2.4 Mount middleware in `src/router.rs`.

## 3. Migrate call sites

- [x] 3.1 Replace direct `tracing::info!` / `warn!` with crate macros in
  `main.rs`, `config.rs`, `state.rs`, `cache/`, `db/`.

## 4. Verification

- [x] 4.1 `cargo test` passes including middleware unit tests.
