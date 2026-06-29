# Design: http-api

## Context

This change adds the only public HTTP surface of the service. The design
needs to fit the rest of the crate, which is structured around a single
processing pipeline (load → decode → transform → filter → watermark →
encode) and a unified `AppError` type.

## Goals

- One endpoint for the whole transformation (`GET /img`) so callers
  don't have to learn multiple routes.
- Errors are machine-dispatchable via a stable string code, so a client
  can write `switch (err.code)` and react without parsing the message.
- Browser cache headers set such that downstream CDNs reuse the work
  without us having to operate our own cache.
- Liveness is decoupled from the image pipeline so health checks never
  touch the network, the disk, or the font cache.

## Non-goals (recap)

No gRPC, no path versioning, no auth, no streaming, no body. See
`proposal.md`.

## Decisions

### Single `AppError` type implementing `IntoResponse`

`src/error.rs` defines one `AppError` enum whose variants map to the
HTTP status code (4xx for caller problems, 502 for upstream / dependency
problems, 500 for internal) and to a stable string `code`. The axum
`IntoResponse` impl is the single place that turns a `Result<_, AppError>`
into a `Response`. Handlers `?`-propagate freely; there is no error
shape branching per handler.

**Why:** keeping the error envelope in one place means a new error
variant is a single match arm in `error.rs` plus a single `code()` arm,
and clients can rely on the body shape being stable. The trade-off is
that "structured per-error context" (e.g. extra fields on the JSON body
for a specific variant) is harder — for now the message string carries
the context.

### Query-string parameters, not JSON body

`/img` is a `GET` with parameters in the URL. A `GET` is safely
cacheable by HTTP intermediaries; a `POST` with a JSON body is not
(without extra headers). For a CDN-fronted image service, cacheability
of the request itself is a key requirement, and that rules out `POST`
on the canonical path.

**Why not a single `POST` with the params in the body:** the same
parameters in a URL let CDNs and browsers cache the *response* against
the *request URL*; that's the whole point of `Cache-Control: max-age=86400`.

### `GET /health` returns a fixed `"ok"` body

`src/handler.rs::health` is a one-line handler. There is no JSON, no
header, no auth — it is a literal "is the process running" signal.

**Why:** k8s liveness probes, AWS ALB target health checks, and the
`curl https://service/health` ops check all want the same minimal
signal. Returning JSON for `/health` would force every probe config
to know our envelope shape, which is unnecessary.

### `Cache-Control: public, max-age=86400`

24 hours is the chosen cache window. Trade-offs:

- **Long enough** to make CDNs and browser caches useful — without a
  cache, every image transform runs every time, defeating the point
  of the service.
- **Short enough** that a source-image swap propagates within a day
  without operator action.
- **Public** because the response varies only by the request URL,
  which is itself content-addressable in practice.

**Why not `immutable` or `must-revalidate`:** we cannot guarantee
immutability (the operator may swap a source file), and we don't need
revalidation since the URL is fully derivable from request parameters.

### `Box<dyn ImageEncoder>` is rejected in favor of a `match`

`handler::encode` is a `match` on the three output formats, not a
trait-object dispatch. Reason: `dyn ImageEncoder` is unsized and the
encoder holds a `&mut Vec<u8>` that cannot move out of the function on
drop. Trait-objecting it would force `unsafe` or `Pin`, neither of
which is justified by the size of the code.

**Trade-off:** adding a new output format means a new `match` arm in
`encode`. We accept that in exchange for simpler memory safety.

## File map

- `src/handler.rs` — `router`, `health`, `img`, `encode`,
  `response_with_image`. Added in this change.
- `src/error.rs` — `AppError`, `IntoResponse`, status / code mapping.
  Added in this change.
- `src/main.rs` — calls `handler::router(state)`. Pre-existing, modified
  to wire the router.

## Open questions

- Should the `POST /img` (protobuf) variant be in this change or
  separated? It lives in a separate change so the GET path is reviewable
  in isolation.
