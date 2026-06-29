# Design: image-source

## Context

This change owns the bytes-acquisition step of the processing pipeline
(`load_source → decode → transform → filter → watermark → encode`).
Everything else in the pipeline operates on an in-memory
`DynamicImage`; this step is the only one that touches the network
or the filesystem.

## Goals

- Three source kinds: `http(s)://`, `file://`, relative path.
- A hard size cap (configurable) for both remote and local reads.
- A fetch timeout for remote reads (configurable).
- An operator toggle to disable remote entirely.
- Format detection by magic-number sniffing, not by URL extension or
  `Content-Type`.

## Non-goals (recap)

No auth, no S3, no streaming. See `proposal.md`.

## Decisions

### Branch on prefix in `load_source`, then dispatch

`src/source.rs::load_source(state, src)` first looks for an `http://`
or `https://` prefix and short-circuits to `fetch_remote`. Then
`file://` → `read_local`. Otherwise it resolves the path against the
configured local root.

**Why:** the prefix is a free, unambiguous signal of which I/O channel
to use. We don't try to parse the URL as a `url::Url` (no dependency
on the `url` crate) — `strip_prefix` is enough for the two schemes we
care about and lets us keep the dependency footprint small.

**Trade-off:** a path that happens to look like a URL but isn't
prefixed correctly falls through to the local-path branch. That is the
correct behavior — local paths can contain colons, and we shouldn't
silently re-route them to the network.

### `read_local` stats before reading

`read_local` calls `std::fs::metadata(path)` first and rejects if the
file's stat size exceeds the cap, then calls `std::fs::read(path)`.

**Why:** `read_to_end` only reports an error after reading. With a
25 MiB cap, a 1 GiB file would be 40x over the cap; statting first
turns that into a fast-path rejection. The trade-off is that a
size-mismatch (e.g. file is truncated between stat and read) can still
slip through, but that's a small window and the second check on the
actual `bytes.len()` in `fetch_remote` covers the remote case
symmetrically.

### `fetch_remote` checks `Content-Length` and then the actual body

Remote fetches look at the `Content-Length` header first (if present)
to short-circuit on a too-large source without downloading it, then
double-check after `bytes()` is collected. This handles servers that
omit `Content-Length` or lie about it (both happen in practice with
CDNs and reverse proxies).

### Disable-remote is checked **before** the HTTP call

`load_source` checks `state.config.allow_remote_sources` and returns
`AppError::RemoteDisabled` before any network I/O. This is a hot path
for security review: an operator that sets `THUMBOR_ALLOW_REMOTE=false`
should be able to verify in the code that no request is ever sent to
the network.

### Format detection by `image::guess_format`, not by extension

`decode` (also in `src/source.rs`) calls
`image::guess_format(bytes)` to detect the format. The handler does
**not** trust the URL's extension or the server's `Content-Type` to
choose the decoder.

**Why:** extensions are routinely wrong (`cat.jpg` that is actually a
PNG, served by an upstream that re-encodes without renaming). The
magic-number sniff is the only reliable signal. We filter the
detected format to the allowlist (PNG, JPEG, WebP, BMP, GIF) and
return `UnsupportedFormat` for anything else, so the downstream
encoder never has to deal with a format it can't handle.

## File map

- `src/source.rs` — `load_source`, `fetch_remote`, `read_local`,
  `decode`. Added in this change.
- `src/state.rs::AppState::sniff_format` — calls `image::guess_format`.
  Added in the same change because the dependency direction is one-way
  (`source` calls `state`).

## Open questions

- Should we cache the source bytes in memory keyed by URL? Not in this
  change — the 24h `Cache-Control` already handles the common case via
  HTTP caches.
