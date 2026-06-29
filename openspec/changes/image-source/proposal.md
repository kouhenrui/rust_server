# Change: image-source

## Why

The image-processing pipeline needs source bytes before it can do
anything. The way those bytes are obtained — `http://`, `file://`, or a
relative path under a configured root — is the contract that all
upstream callers depend on, and the operational surface that lets
deployments lock the service down (disable remote, cap size, set a
timeout).

## What Changes

- Support `http://` and `https://` URLs as the `src` value, fetched
  with a configurable timeout.
- Support `file://` URIs as the `src` value.
- Support relative paths joined onto `THUMBOR_LOCAL_SOURCE_ROOT`
  (falling back to the literal path when the env var is unset).
- Reject `http(s)://` sources when `THUMBOR_ALLOW_REMOTE=false` with
  status 502 and error code `remote_disabled`.
- Reject any source larger than `THUMBOR_MAX_SOURCE_BYTES` with status
  413 and error code `source_too_large`.
- Bound remote fetches with `THUMBOR_FETCH_TIMEOUT_MS`; on timeout,
  return status 502 and error code `upstream_failed`.

## Capabilities

- `image-source`: HTTP(S) and `file://` and relative-path source
  loading, with the remote-disable, size-cap, and timeout
  constraints.

## Impact

- **Code:** new `src/source.rs` with `load_source`, `fetch_remote`,
  `read_local`, `decode`. Called by the `/img` handler.
- **Dependencies:** `reqwest 0.12` (rustls-tls only — no OpenSSL on
  the link line), `image 0.25`.
- **Config:** `THUMBOR_ALLOW_REMOTE`, `THUMBOR_MAX_SOURCE_BYTES`,
  `THUMBOR_FETCH_TIMEOUT_MS`, `THUMBOR_LOCAL_SOURCE_ROOT` — all
  documented in the `runtime-config` change.
- **Security:** an attacker pointing `src` at a private IP can still
  trick the service into making outbound requests when remote sources
  are enabled. The `THUMBOR_ALLOW_REMOTE` switch is the operator's
  lever; there is no IP allowlist at this layer.

## Non-goals

- Source-format negotiation (the image bytes are taken as-is; the
  decoder sniffs the magic number).
- Authenticated remote sources (no S3 / GCS pre-signed URLs at this
  layer; could be added as a future `s3://` or `gs://` scheme).
- Streaming / chunked source reads (the whole image is loaded into
  memory; this is bounded by `THUMBOR_MAX_SOURCE_BYTES`).
- Source-side rate limiting or per-source-token quotas.

## Affected domains

- **image-source** (created) — the only spec modified or created by
  this change.
