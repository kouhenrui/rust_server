# Tasks: image-source

## 1. Source loading module

- [x] 1.1 Create `src/source.rs` with `load_source`, `fetch_remote`,
  `read_local`, `decode`.
- [x] 1.2 `load_source` branches on `http://` / `https://` / `file://`
  prefix; falls through to the local-root + literal-path resolution.
- [x] 1.3 `fetch_remote` calls `reqwest::Client::get(url)`, checks the
  status, checks `Content-Length` if present, and finally collects
  `bytes()` and re-checks the actual length.

## 2. Size cap and timeout

- [x] 2.1 Both `fetch_remote` and `read_local` enforce
  `THUMBOR_MAX_SOURCE_BYTES` and return `AppError::SourceTooLarge` on
  overrun.
- [x] 2.2 `AppState::new` builds the `reqwest::Client` with
  `THUMBOR_FETCH_TIMEOUT_MS`. A timeout surfaces as
  `AppError::Upstream` → 502 `upstream_failed`.

## 3. Remote disable

- [x] 3.1 `load_source` rejects `http(s)://` sources with
  `AppError::RemoteDisabled` (502) when
  `state.config.allow_remote_sources == false`, before any network I/O.

## 4. Format detection

- [x] 4.1 `AppState::sniff_format` uses `image::guess_format` (the
  image 0.25+ API; the older `ImageFormat::from_bytes` does not
  exist).
- [x] 4.2 `source::decode` allows PNG, JPEG, WebP, BMP, GIF and
  rejects everything else with `AppError::UnsupportedFormat`.

## 5. Tests

- [x] 5.1 Unit test: `rejects_remote_when_disabled` — when
  `THUMBOR_ALLOW_REMOTE=false`, `load_source` returns
  `AppError::RemoteDisabled` for an `https://` URL.
- [x] 5.2 Unit test: `missing_local_source_404s` — when a relative
  path doesn't exist on disk, `load_source` returns
  `AppError::SourceNotFound`.

## 6. Verification

- [x] 6.1 `cargo check --all-targets` exits 0.
- [x] 6.2 `cargo test --lib` passes (covers the two unit tests above).
