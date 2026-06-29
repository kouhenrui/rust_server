# Change: cache-backend

## Why

The service needs an optional cache layer for future image/metadata caching
without coupling handlers to a specific backend.

## What Changes

- Pluggable `Cache` trait with `disabled`, `memory`, and `redis` backends.
- `THUMBOR_CACHE_*` environment variables.
- Connected during `AppState::connect`.

## Capabilities

- `cache-backend` — configuration and trait abstraction.

## Impact

- **Code:** `src/cache/`, `src/state.rs`
- **Dependencies:** `redis` crate

## Non-goals

- Automatic caching of `/img` responses (trait is ready, handlers not wired)
- Cluster-aware Redis client configuration beyond single endpoint

## Affected domains

- **cache-backend** (created)
- **runtime-config** (env vars documented in cache spec)
