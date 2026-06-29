# Tasks: http-api

## 1. Error type

- [x] 1.1 Define `AppError` enum with variants for bad request, source
  not found, source too large, unsupported format, decode failed,
  remote disabled, watermark font missing, invalid filter, upstream
  failed, internal. (`src/error.rs`)
- [x] 1.2 Implement `From<reqwest::Error>`, `From<image::ImageError>`,
  `From<std::io::Error>` so handlers can `?` freely. (`src/error.rs`)
- [x] 1.3 Implement `IntoResponse` to render the JSON envelope
  `{"error":{"code","message"}}` with the right HTTP status. (`src/error.rs`)

## 2. HTTP routing

- [x] 2.1 Build a `Router` in `src/handler.rs::router(state)` with
  `/health` and `/img` routes. (`src/handler.rs`)
- [x] 2.2 Implement `health` handler returning `&'static str` `"ok"`.
  (`src/handler.rs`)
- [x] 2.3 Implement the `/img` handler that parses query parameters,
  runs the processing pipeline, and writes the response. (`src/handler.rs`)
- [x] 2.4 Set `Content-Type` and `Cache-Control: public, max-age=86400`
  on successful image responses. (`src/handler.rs`)

## 3. Wire it up

- [x] 3.1 `src/main.rs` calls `handler::router(state)` and serves it
  with `axum::serve`, plus `TraceLayer` and `CorsLayer::permissive()`.
- [x] 3.2 `src/lib.rs` exposes `pub mod handler; pub mod error;`.

## 4. Verification

- [x] 4.1 `cargo check --all-targets` exits 0.
- [x] 4.2 `cargo test --lib` passes (covers the error and handler paths
  through the parsing and processing logic).
