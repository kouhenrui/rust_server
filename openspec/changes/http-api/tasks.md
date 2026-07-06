# Tasks: http-api

## 1. Error type

- [x] 1.1 Define `AppError` enum with variants for bad request, source
  not found, source too large, unsupported format, decode failed,
  remote disabled, watermark font missing, invalid filter, upstream,
  unauthorized, invalid_token, internal. (`src/error.rs`)
- [x] 1.2 Implement `From<reqwest::Error>`, `From<image::ImageError>`,
  `From<std::io::Error>` so handlers can `?` freely. (`src/error.rs`)
- [x] 1.3 Map errors to unified envelope via `src/response.rs` (`api_error`,
  `ImageOutcome`) instead of raw `IntoResponse` JSON on `AppError`.
- [x] 1.4 Implement `AppError::status()` and `AppError::code()` for stable
  HTTP status and `err.kind` strings. (`src/error.rs`)
- [x] 1.5 Add `AppResultExt` / `AppResultMapExt` for handler ergonomics.
  (`src/error.rs`)

## 2. HTTP routing

- [x] 2.1 Build a `Router` in `src/router.rs` with `/health`, `/login`,
  `/me`, `GET /img`, and `POST /img`. (`src/router.rs`)
- [x] 2.2 Implement `health` in `src/controller/health.rs` with
  `check_health` (cache/db ping) via `ok!`.
- [x] 2.3 Implement `login` and `me` in `src/controller/auth.rs`.
- [x] 2.4 Implement `img_get` and `img_post` in `src/controller/img.rs`.
- [x] 2.4 JSON success responses use `Content-Type: application/json` with
  base64-encoded `data.image`.
- [x] 2.5 `process_image` orchestrates load → transform → filter → watermark →
  encode (see `changes/image-pipeline/tasks.md`).
- [x] 2.6 `router` wraps state in `Arc<AppState>` and applies
  `authorize_middleware` + `logging_middleware`.

## 3. Wire it up

- [x] 3.1 `src/main.rs` calls `router::router(state)` and serves it with
  `axum::serve`, `CorsLayer::permissive()`, and graceful shutdown.
- [x] 3.2 `src/lib.rs` exposes `router`, `controller`, `response`, `middleware`,
  `params`, `auth`, `error`, `http_client`, `util`.
- [x] 3.3 Integration tests in `tests/integration.rs` + `tests/common/` cover
  health, login, `/me`, GET/POST `/img`, cache, and error envelopes.

## 4. Verification

- [x] 4.1 `cargo check --all-targets` exits 0.
- [x] 4.2 `cargo test` passes (unit tests + `tests/integration.rs`).

## 5. Production readiness (backlog)

- [x] 5.1 Replace `CorsLayer::permissive()` with configurable `THUMBOR_CORS_ORIGINS`
  (empty = permissive fallback). (`src/main.rs`)
- [ ] 5.2 Optional: protect `/img` with JWT when auth is required.
