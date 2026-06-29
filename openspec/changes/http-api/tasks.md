# Tasks: http-api

## 1. Error type

- [x] 1.1 Define `AppError` enum with variants for bad request, source
  not found, source too large, unsupported format, decode failed,
  remote disabled, watermark font missing, invalid filter, upstream
  failed, internal. (`src/error.rs`)
- [x] 1.2 Implement `From<reqwest::Error>`, `From<image::ImageError>`,
  `From<std::io::Error>` so handlers can `?` freely. (`src/error.rs`)
- [x] 1.3 Map errors to unified envelope via `src/response.rs` (`api_error`,
  `ImageOutcome`) instead of raw `IntoResponse` JSON on `AppError`.

## 2. HTTP routing

- [x] 2.1 Build a `Router` in `src/router.rs` with `/health`, `GET /img`,
  and `POST /img`. (`src/router.rs`)
- [x] 2.2 Implement `health` in `src/controller/health.rs` returning the
  unified JSON envelope with `data.status = "ok"`.
- [x] 2.3 Implement `img_get` and `img_post` in `src/controller/img.rs`.
- [x] 2.4 JSON success responses use `Content-Type: application/json` with
  base64-encoded `data.image`.

## 3. Wire it up

- [x] 3.1 `src/main.rs` calls `router::router(state)` and serves it with
  `axum::serve`, `CorsLayer::permissive()`, and graceful shutdown.
- [x] 3.2 `src/lib.rs` exposes `router`, `controller`, `response`, `middleware`.

## 4. Verification

- [x] 4.1 `cargo check --all-targets` exits 0.
- [x] 4.2 `cargo test` passes (unit tests + `tests/integration.rs`).
