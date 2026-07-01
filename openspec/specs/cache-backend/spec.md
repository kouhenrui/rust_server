# Cache Backend

## Purpose

Define the pluggable cache layer used at application startup.

## Requirements

### Requirement: Backend selection

The cache backend MUST be selected by `THUMBOR_CACHE_BACKEND`:

| value | behavior |
|---|---|
| `disabled` / `none` / `off` / unset | no-op cache |
| `memory` | in-process LRU cache |
| `redis` | Redis via `redis` crate |

Unknown values MUST log a warning and fall back to disabled.

#### Scenario: default is disabled

- GIVEN `THUMBOR_CACHE_BACKEND` is unset
- WHEN `CacheBackendConfig::from_env()` runs
- THEN the backend is disabled and `Cache::connect` returns a no-op provider

#### Scenario: memory backend

- GIVEN `THUMBOR_CACHE_BACKEND=memory`
- WHEN the service starts
- THEN `AppState::connect` logs `cache ready` with backend `memory`

### Requirement: Memory cache configuration

When `memory` is selected, the service MUST read:

| variable | default | purpose |
|---|---|---|
| `THUMBOR_CACHE_MEMORY_MAX_ENTRIES` | `1024` | LRU capacity |
| `THUMBOR_CACHE_MEMORY_TTL_SECS` | `3600` | default TTL; `0` means no expiry |

Invalid values MUST warn and keep the default.

### Requirement: Redis cache configuration

When `redis` is selected, the service MUST support either `THUMBOR_REDIS_URL`
or discrete `THUMBOR_REDIS_HOST`, `THUMBOR_REDIS_PORT`, `THUMBOR_REDIS_DB`,
`THUMBOR_REDIS_USERNAME`, `THUMBOR_REDIS_PASSWORD`.

#### Scenario: redis connection log

- GIVEN `THUMBOR_CACHE_BACKEND=redis` and valid Redis settings
- WHEN `RedisCache::connect` runs
- THEN a connection log is emitted with credentials redacted

### Requirement: Cache trait

All backends MUST implement `cache::Cache` with `get`, `set`, `delete`, and
`ping`. The cache is connected during `AppState::connect` and held on
`AppState`.

**Note:** As of current code, the `/img` pipeline does not yet read or write
the cache; see `changes/cache-backend/tasks.md` §6.

#### Scenario: disabled ping succeeds

- GIVEN the disabled backend
- WHEN `ping` is called
- THEN it returns `Ok(())` without network I/O
