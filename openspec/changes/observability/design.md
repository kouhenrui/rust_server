# Design: observability

## Decisions

### Encapsulated macros with call-site metadata

`info!`, `warn!`, and `error!` attach `module`, `file`, and `line` so
operators can jump to source without enabling full `RUST_BACKTRACE`.

### Middleware buffers request body for logging

POST protobuf bodies are summarized (field values), not dumped raw. Response
bodies are never logged.

### nanoid for trace ids

Client-supplied `X-Trace-Id` is honored when non-empty; otherwise nanoid
generates an id.

## File map

- `src/logger/` — subscriber bootstrap and macros
- `src/middleware/middleware.rs` — HTTP access middleware
- `src/router.rs` — applies middleware via `from_fn`
